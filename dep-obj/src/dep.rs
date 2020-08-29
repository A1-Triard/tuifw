use alloc::alloc::{alloc, dealloc, Layout};
use alloc::vec::Vec;
use core::cmp::max;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem::{replace, align_of, size_of, transmute};
use core::ptr::{self, NonNull, null_mut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct DepObjLock(AtomicBool);

impl DepObjLock {
    pub const fn new() -> Self { DepObjLock(AtomicBool::new(false)) }
}

impl Default for DepObjLock {
    fn default() -> Self { DepObjLock::new() }
}

pub trait DepObjRaw {
    fn lock() -> &'static DepObjLock where Self: Sized;
}

pub trait DepObj: DepObjRaw {
    fn dep_props(&self) -> &DepObjProps<Self> where Self: Sized;
    fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> where Self: Sized;
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct DepTypeBuilder<Owner: DepObj + ?Sized> {
    align: usize,
    size: usize,
    default: Vec<(isize, unsafe fn(usize, *mut u8), usize)>,
    phantom: PhantomData<Owner>,
}

unsafe impl<Owner: DepObj + ?Sized> Send for DepTypeBuilder<Owner> { }
unsafe impl<Owner: DepObj + ?Sized> Sync for DepTypeBuilder<Owner> { }
impl<Owner: DepObj + ?Sized> Unpin for DepTypeBuilder<Owner> { }

unsafe fn store_default<T>(fn_ptr: usize, storage: *mut u8) {
    let fn_ptr: fn() -> T = transmute(fn_ptr);
    ptr::write(storage as *mut T, fn_ptr());
}

impl<Owner: DepObj> DepTypeBuilder<Owner> {
    pub fn new() -> Option<DepTypeBuilder<Owner>> {
        let lock = Owner::lock();
        if lock.0.compare_and_swap(false, true, Ordering::Relaxed) {
            None
        } else {
            Some(DepTypeBuilder { size: 0, align: 1, default: Vec::new(), phantom: PhantomData })
        }
    }
}

impl<Owner: DepObj + ?Sized> DepTypeBuilder<Owner> {
    pub fn prop<T>(&mut self, default: fn() -> T) -> DepProp<Owner, T> {
        let align = align_of::<T>();
        self.align = max(self.align, align);
        let padding = (align - self.size % align) % align;
        self.size = self.size.checked_add(padding).expect("out of memory");
        let offset = self.size.try_into().expect("out of memory");
        self.size = self.size.checked_add(size_of::<T>()).expect("out of memory");
        self.default.push((offset, store_default::<T>, default as usize));
        DepProp { offset, phantom: PhantomData }
    }

    pub fn build(self) -> DepTypeToken<Owner> {
        DepTypeToken {
            layout: Layout::from_size_align(self.size, self.align).expect("out of memory"),
            default: self.default,
            phantom: PhantomData
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound=""), Copy(bound=""), Clone(bound=""), Eq(bound=""), PartialEq(bound=""))]
#[derivative(Hash(bound=""), Ord(bound=""), PartialOrd(bound=""))]
pub struct DepProp<Owner: DepObj + ?Sized, T> {
    offset: isize,
    phantom: PhantomData<(*const Owner, T)>,
}

unsafe impl<Owner: DepObj + ?Sized, T> Send for DepProp<Owner, T> { }
unsafe impl<Owner: DepObj + ?Sized, T> Sync for DepProp<Owner, T> { }
impl<Owner: DepObj + ?Sized, T> Unpin for DepProp<Owner, T> { }

impl<Owner: DepObj + ?Sized, T> DepProp<Owner, T> {
    pub fn get(self, obj_props: &DepObjProps<Owner>) -> &T {
        unsafe { &*(obj_props.storage.offset(self.offset) as *const T) }
    }

    pub fn get_mut(self, obj_props: &mut DepObjProps<Owner>) -> &mut T {
        unsafe { &mut *(obj_props.storage.offset(self.offset) as *mut T) }
    }

    pub fn set(self, obj_props: &mut DepObjProps<Owner>, value: T) -> T {
        replace(self.get_mut(obj_props), value)
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct DepTypeToken<Owner: DepObj + ?Sized> {
    layout: Layout,
    default: Vec<(isize, unsafe fn(usize, *mut u8), usize)>,
    phantom: PhantomData<Owner>,
}

unsafe impl<Owner: DepObj + ?Sized> Send for DepTypeToken<Owner> { }
unsafe impl<Owner: DepObj + ?Sized> Sync for DepTypeToken<Owner> { }
impl<Owner: DepObj + ?Sized> Unpin for DepTypeToken<Owner> { }

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct DepObjProps<Owner: DepObj + ?Sized> {
    layout: Layout,
    storage: *mut u8,
    phantom: PhantomData<Owner>,
}

unsafe impl<Owner: DepObj + ?Sized> Send for DepObjProps<Owner> { }
unsafe impl<Owner: DepObj + ?Sized> Sync for DepObjProps<Owner> { }
impl<Owner: DepObj + ?Sized> Unpin for DepObjProps<Owner> { }

impl<Owner: DepObj + ?Sized> DepObjProps<Owner> {
    pub fn new(type_token: &DepTypeToken<Owner>) -> DepObjProps<Owner> {
        let storage = if type_token.layout.size() == 0 {
            null_mut()
        } else {
            NonNull::new(unsafe { alloc(type_token.layout) }).expect("out of memory").as_ptr()
        };
        for &(offset, store, fn_ptr) in &type_token.default {
            unsafe { store(fn_ptr, storage.offset(offset)) };
        }
        DepObjProps {
            layout: type_token.layout,
            storage,
            phantom: PhantomData
        }
    }
}

impl<Owner: DepObj + ?Sized> Drop for DepObjProps<Owner> {
    fn drop(&mut self) {
        if !self.storage.is_null() {
            unsafe { dealloc(self.storage, self.layout) };
            self.storage = null_mut();
        }
    }
}

#[macro_export]
macro_rules! DepObjRaw {
    (()
        $(pub $(($($vis:tt)+))?)? enum $name:ident
        $($tail:tt)+ ) => {
        DepObjRaw! {
            @impl $name
        }
    };
    (()
        $(pub $(($($vis:tt)+))?)? struct $name:ident
        $($tail:tt)+ ) => {
        DepObjRaw! {
            @impl $name
        }
    };
    (@impl $name:ident) => {
        impl $crate::dep::DepObjRaw for $name {
            fn lock() -> &'static $crate::dep::DepObjLock {
                static LOCK: $crate::dep::DepObjLock = $crate::dep::DepObjLock::new();
                &LOCK
            }
        }
    };
}
