use alloc::alloc::{alloc, Layout};
use alloc::vec::Vec;
use core::cmp::max;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem::{align_of, size_of, transmute};
use core::ptr::{self, NonNull, null_mut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct DepObjLock(AtomicBool);

impl DepObjLock {
    pub const fn new() -> Self { DepObjLock(AtomicBool::new(false)) }
}

impl Default for DepObjLock {
    fn default() -> Self { DepObjLock::new() }
}

pub trait DepObj {
    fn lock() -> &'static DepObjLock;
}

pub struct DepTypeBuilder<Owner: DepObj> {
    align: usize,
    size: usize,
    default: Vec<(isize, unsafe fn(usize, *mut u8), usize)>,
    phantom: PhantomData<Owner>,
}

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

    pub fn build(self) -> DepType<Owner> {
        DepType {
            layout: Layout::from_size_align(self.size, self.align).expect("out of memory"),
            default: self.default,
            phantom: PhantomData
        }
    }
}

pub struct DepProp<Owner: DepObj, T> {
    offset: isize,
    phantom: PhantomData<(Owner, T)>,
}

impl<Owner: DepObj, T> DepProp<Owner, T> {
}

pub struct DepType<Owner: DepObj> {
    layout: Layout,
    default: Vec<(isize, unsafe fn(usize, *mut u8), usize)>,
    phantom: PhantomData<Owner>,
}

impl<Owner: DepObj> DepType<Owner> {
}

pub struct DepObjProps<Owner: DepObj> {
    storage: *mut u8,
    phantom: PhantomData<Owner>,
}

impl<Owner: DepObj> DepObjProps<Owner> {
    pub fn new(type_: &DepType<Owner>) -> DepObjProps<Owner> {
        let storage = if type_.layout.size() == 0 {
            null_mut()
        } else {
            NonNull::new(unsafe { alloc(type_.layout) }).expect("out of memory").as_ptr()
        };
        for &(offset, store, fn_ptr) in &type_.default {
            unsafe { store(fn_ptr, storage.offset(offset)) };
        }
        DepObjProps {
            storage,
            phantom: PhantomData
        }
    }
}



