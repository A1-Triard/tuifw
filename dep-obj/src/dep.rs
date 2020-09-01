use alloc::alloc::{alloc, dealloc, Layout};
use alloc::vec::Vec;
use core::any::{Any, TypeId};
use core::cmp::max;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem::{replace, align_of, size_of, transmute};
use core::ptr::{self, NonNull, null_mut};
use core::sync::atomic::{AtomicBool, Ordering};
use components_arena::ComponentId;

pub trait Context {
    fn get_raw(&self, type_: TypeId) -> Option<&dyn Any>;
    fn get_mut_raw(&mut self, type_: TypeId) -> Option<&mut dyn Any>;
}

pub trait ContextExt: Context {
    fn get<T: 'static>(&self) -> Option<&T> {
        self.get_raw(TypeId::of::<T>()).map(|x| x.downcast_ref::<T>().expect("invalid cast"))
    }

    fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.get_mut_raw(TypeId::of::<T>()).map(|x| x.downcast_mut::<T>().expect("invalid cast"))
    }
}

impl<T: Context + ?Sized> ContextExt for T { }

pub struct DepTypeLock(AtomicBool);

impl DepTypeLock {
    pub const fn new() -> Self { DepTypeLock(AtomicBool::new(false)) }
}

impl Default for DepTypeLock {
    fn default() -> Self { DepTypeLock::new() }
}

pub struct DepTypeToken<Type: DepType> {
    layout: Layout,
    default: Vec<(isize, unsafe fn(usize, *mut u8), usize)>,
    type_: Type,
}

impl<Type: DepType> DepTypeToken<Type> {
    pub fn type_(&self) -> &Type { &self.type_ }
}

pub unsafe trait DepType {
    fn lock() -> &'static DepTypeLock;
}

pub trait DepObj {
    type Type: DepType;
    type Id: ComponentId;
    fn dep_props(&self) -> &DepObjProps<Self::Type, Self::Id> where Self: Sized;
    fn dep_props_mut(&mut self) -> &mut DepObjProps<Self::Type, Self::Id> where Self: Sized;
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct DepTypeBuilder<Type: DepType> {
    align: usize,
    size: usize,
    default: Vec<(isize, unsafe fn(usize, *mut u8), usize)>,
    phantom: PhantomData<Type>,
}

unsafe impl<Type: DepType> Send for DepTypeBuilder<Type> { }
unsafe impl<Type: DepType> Sync for DepTypeBuilder<Type> { }
impl<Type: DepType> Unpin for DepTypeBuilder<Type> { }

#[derive(Debug)]
struct Entry<Type> {
    value: Type,
    on_changed: Option<Vec<usize>>,
}

unsafe fn store_default<Type>(fn_ptr: usize, storage: *mut u8) {
    let fn_ptr: fn() -> Type = transmute(fn_ptr);
    ptr::write(storage as *mut Entry<Type>, Entry { value: fn_ptr(), on_changed: None });
}

impl<Type: DepType> DepTypeBuilder<Type> {
    pub fn new() -> Option<DepTypeBuilder<Type>> {
        let lock = Type::lock();
        if lock.0.compare_and_swap(false, true, Ordering::Relaxed) {
            None
        } else {
            Some(DepTypeBuilder { size: 0, align: 1, default: Vec::new(), phantom: PhantomData })
        }
    }
}

impl<Type: DepType> DepTypeBuilder<Type> {
    pub fn prop<PropType>(&mut self, default: fn() -> PropType) -> DepProp<Type, PropType> {
        let align = align_of::<Entry<PropType>>();
        self.align = max(self.align, align);
        let padding = (align - self.size % align) % align;
        self.size = self.size.checked_add(padding).expect("out of memory");
        let offset = self.size.try_into().expect("out of memory");
        self.size = self.size.checked_add(size_of::<Entry<PropType>>()).expect("out of memory");
        self.default.push((offset, store_default::<PropType>, default as usize));
        DepProp { offset, phantom: (PhantomData, PhantomData) }
    }

    pub fn build(self, type_: Type) -> DepTypeToken<Type> {
        DepTypeToken {
            layout: Layout::from_size_align(self.size, self.align).expect("out of memory"),
            default: self.default,
            type_
        }
    }
}

pub struct OnChanged<OwnerId: ComponentId, Type>(
    Option<Vec<usize>>,
    (PhantomData<OwnerId>, PhantomData<Type>),
);

impl<OwnerId: ComponentId, Type> OnChanged<OwnerId, Type> {
    pub fn raise(self, owner_id: OwnerId, context: &mut dyn Context, old: &Type) {
        if let Some(on_changed) = self.0 {
            for on_changed in on_changed {
                let on_changed: fn(owner_id: OwnerId, context: &mut dyn Context, old: &Type) =
                    unsafe { transmute(on_changed) };
                on_changed(owner_id, context, old);
            }
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound=""), Copy(bound=""), Clone(bound=""), Eq(bound=""), PartialEq(bound=""))]
#[derivative(Hash(bound=""), Ord(bound=""), PartialOrd(bound=""))]
pub struct DepProp<Owner: DepType, Type> {
    offset: isize,
    phantom: (PhantomData<Owner>, PhantomData<Type>),
}

unsafe impl<Owner: DepType, Type> Send for DepProp<Owner, Type> { }
unsafe impl<Owner: DepType, Type> Sync for DepProp<Owner, Type> { }
impl<Owner: DepType, Type> Unpin for DepProp<Owner, Type> { }

impl<Owner: DepType, Type: Eq> DepProp<Owner, Type> {
    pub fn set_distinct<OwnerId: ComponentId>(
        self,
        obj_props: &mut DepObjProps<Owner, OwnerId>,
        value: Type
    ) -> (Type, OnChanged<OwnerId, Type>) {
        let entry = self.get_entry_mut(obj_props);
        let old = replace(&mut entry.value, value);
        let on_changed = if old == entry.value { None } else { entry.on_changed.clone() };
        (old, OnChanged(on_changed, (PhantomData, PhantomData)))
    }
}

impl<Owner: DepType, Type> DepProp<Owner, Type> {
    pub fn set_uncond<OwnerId: ComponentId>(
        self,
        obj_props: &mut DepObjProps<Owner, OwnerId>,
        value: Type
    ) -> (Type, OnChanged<OwnerId, Type>) {
        let entry = self.get_entry_mut(obj_props);
        let old = replace(&mut entry.value, value);
        (old, OnChanged(entry.on_changed.clone(), (PhantomData, PhantomData)))
    }

    pub fn get<OwnerId: ComponentId>(
        self,
        obj_props: &DepObjProps<Owner, OwnerId>
    ) -> &Type {
        &self.get_entry(obj_props).value
    }

    pub fn on_changed<OwnerId: ComponentId>(
        self,
        obj_props: &mut DepObjProps<Owner, OwnerId>,
        callback: fn(owner_id: OwnerId, context: &mut dyn Context, old: &Type)
    ) {
        let callback = unsafe { transmute(callback) };
        let entry = self.get_entry_mut(obj_props);
        if let Some(on_changed) = entry.on_changed.as_mut() {
            on_changed.push(callback);
        } else {
            entry.on_changed = Some(vec![callback]);
        }
    }

    fn get_entry<OwnerId: ComponentId>(
        self,
        obj_props: &DepObjProps<Owner, OwnerId>
    ) -> &Entry<Type> {
        unsafe { &*(obj_props.storage.offset(self.offset) as *const Entry<Type>) }
    }

    fn get_entry_mut<OwnerId: ComponentId>(
        self,
        obj_props: &mut DepObjProps<Owner, OwnerId>
    ) -> &mut Entry<Type> {
        unsafe { &mut *(obj_props.storage.offset(self.offset) as *mut Entry<Type>) }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct DepObjProps<Owner: DepType, OwnerId: ComponentId> {
    layout: Layout,
    storage: *mut u8,
    phantom: (PhantomData<Owner>, PhantomData<OwnerId>)
}

unsafe impl<Owner: DepType, OwnerId: ComponentId> Send for DepObjProps<Owner, OwnerId> { }
unsafe impl<Owner: DepType, OwnerId: ComponentId> Sync for DepObjProps<Owner, OwnerId> { }
impl<Owner: DepType, OwnerId: ComponentId> Unpin for DepObjProps<Owner, OwnerId> { }

impl<Owner: DepType, OwnerId: ComponentId> DepObjProps<Owner, OwnerId> {
    pub fn new(type_token: &DepTypeToken<Owner>) -> DepObjProps<Owner, OwnerId> {
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
            phantom: (PhantomData, PhantomData)
        }
    }
}

impl<Owner: DepType, OwnerId: ComponentId> Drop for DepObjProps<Owner, OwnerId> {
    fn drop(&mut self) {
        if !self.storage.is_null() {
            unsafe { dealloc(self.storage, self.layout) };
            self.storage = null_mut();
        }
    }
}

#[macro_export]
macro_rules! DepType {
    (()
        $(pub $(($($vis:tt)+))?)? enum $name:ident $($tail:tt)+ ) => {
        DepType! {
            @impl $name
        }
    };
    (()
        $(pub $(($($vis:tt)+))?)? struct $name:ident $($tail:tt)+ ) => {
        DepType! {
            @impl $name
        }
    };
    (@impl $name:ident) => {
        unsafe impl $crate::DepType for $name {
            fn lock() -> &'static $crate::DepTypeLock {
                static LOCK: $crate::DepTypeLock = $crate::DepTypeLock::new();
                &LOCK
            }
        }
    };
}
