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
    drop: Vec<(isize, unsafe fn(*mut u8))>,
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
    drop: Vec<(isize, unsafe fn(*mut u8))>,
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

unsafe fn drop_entry<Type>(storage: *mut u8) {
    ptr::drop_in_place(storage as *mut Entry<Type>);
}

impl<Type: DepType> DepTypeBuilder<Type> {
    pub fn new() -> Option<DepTypeBuilder<Type>> {
        let lock = Type::lock();
        if lock.0.compare_and_swap(false, true, Ordering::Relaxed) {
            None
        } else {
            Some(DepTypeBuilder {
                size: 0,
                align: 1,
                default: Vec::new(),
                drop: Vec::new(),
                phantom: PhantomData
            })
        }
    }
}

impl<Type: DepType> DepTypeBuilder<Type> {
    pub fn prop<PropType>(&mut self, default: fn() -> PropType) -> DepPropRaw<Type, PropType> {
        let align = align_of::<Entry<PropType>>();
        self.align = max(self.align, align);
        let padding = (align - self.size % align) % align;
        self.size = self.size.checked_add(padding).expect("out of memory");
        let offset = self.size.try_into().expect("out of memory");
        debug_assert_ne!(size_of::<Entry<PropType>>(), 0);
        self.size = self.size.checked_add(size_of::<Entry<PropType>>()).expect("out of memory");
        self.default.push((offset, store_default::<PropType>, default as usize));
        self.drop.push((offset, drop_entry::<PropType>));
        DepPropRaw { offset, phantom: (PhantomData, PhantomData) }
    }

    pub fn build(mut self, type_: Type) -> DepTypeToken<Type> {
        self.default.shrink_to_fit();
        self.drop.shrink_to_fit();
        DepTypeToken {
            layout: Layout::from_size_align(self.size, self.align).expect("out of memory"),
            default: self.default,
            drop: self.drop,
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
pub struct DepPropRaw<OwnerType: DepType, Type> {
    offset: isize,
    phantom: (PhantomData<OwnerType>, PhantomData<Type>),
}

unsafe impl<OwnerType: DepType, Type> Send for DepPropRaw<OwnerType, Type> { }
unsafe impl<OwnerType: DepType, Type> Sync for DepPropRaw<OwnerType, Type> { }
impl<OwnerType: DepType, Type> Unpin for DepPropRaw<OwnerType, Type> { }

impl<OwnerType: DepType, Type> DepPropRaw<OwnerType, Type> {
    pub fn owned_by<Owner: DepObj<Type=OwnerType>>(self) -> DepProp<Owner, Type> {
        DepProp(self, PhantomData)
    }

    fn get_entry<OwnerId: ComponentId>(
        self,
        obj_props: &DepObjProps<OwnerType, OwnerId>
    ) -> &Entry<Type> {
        unsafe { &*(obj_props.storage.offset(self.offset) as *const Entry<Type>) }
    }

    fn get_entry_mut<OwnerId: ComponentId>(
        self,
        obj_props: &mut DepObjProps<OwnerType, OwnerId>
    ) -> &mut Entry<Type> {
        unsafe { &mut *(obj_props.storage.offset(self.offset) as *mut Entry<Type>) }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound=""), Copy(bound=""), Clone(bound=""), Eq(bound=""), PartialEq(bound=""))]
#[derivative(Hash(bound=""), Ord(bound=""), PartialOrd(bound=""))]
pub struct DepProp<Owner: DepObj, Type>(
    DepPropRaw<Owner::Type, Type>,
    PhantomData<Owner>,
);

unsafe impl<Owner: DepObj, Type> Send for DepProp<Owner, Type> { }
unsafe impl<Owner: DepObj, Type> Sync for DepProp<Owner, Type> { }
impl<Owner: DepObj, Type> Unpin for DepProp<Owner, Type> { }

impl<Owner: DepObj, Type: Eq> DepProp<Owner, Type> {
    pub fn set_distinct(
        self,
        obj: &mut Owner,
        value: Type
    ) -> (Type, OnChanged<Owner::Id, Type>) {
        let entry = self.0.get_entry_mut(obj.dep_props_mut());
        let old = replace(&mut entry.value, value);
        let on_changed = if old == entry.value { None } else { entry.on_changed.clone() };
        (old, OnChanged(on_changed, (PhantomData, PhantomData)))
    }
}

impl<Owner: DepObj, Type> DepProp<Owner, Type> {
    pub fn set_uncond(
        self,
        obj: &mut Owner,
        value: Type
    ) -> (Type, OnChanged<Owner::Id, Type>) {
        let entry = self.0.get_entry_mut(obj.dep_props_mut());
        let old = replace(&mut entry.value, value);
        (old, OnChanged(entry.on_changed.clone(), (PhantomData, PhantomData)))
    }

    pub fn get(
        self,
        obj: &Owner
    ) -> &Type {
        &self.0.get_entry(obj.dep_props()).value
    }

    pub fn on_changed(
        self,
        obj: &mut Owner,
        callback: fn(owner_id: Owner::Id, context: &mut dyn Context, old: &Type)
    ) {
        let callback = unsafe { transmute(callback) };
        let entry = self.0.get_entry_mut(obj.dep_props_mut());
        if let Some(on_changed) = entry.on_changed.as_mut() {
            on_changed.push(callback);
        } else {
            entry.on_changed = Some(vec![callback]);
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct DepObjProps<OwnerType: DepType, OwnerId: ComponentId> {
    layout: Layout,
    storage: *mut u8,
    drop: Vec<(isize, unsafe fn(*mut u8))>,
    phantom: (PhantomData<OwnerType>, PhantomData<OwnerId>)
}

unsafe impl<OwnerType: DepType, OwnerId: ComponentId> Send for DepObjProps<OwnerType, OwnerId> { }
unsafe impl<OwnerType: DepType, OwnerId: ComponentId> Sync for DepObjProps<OwnerType, OwnerId> { }
impl<OwnerType: DepType, OwnerId: ComponentId> Unpin for DepObjProps<OwnerType, OwnerId> { }

impl<OwnerType: DepType, OwnerId: ComponentId> DepObjProps<OwnerType, OwnerId> {
    pub fn new(type_token: &DepTypeToken<OwnerType>) -> DepObjProps<OwnerType, OwnerId> {
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
            drop: type_token.drop.clone(),
            phantom: (PhantomData, PhantomData)
        }
    }
}

impl<OwnerType: DepType, OwnerId: ComponentId> Drop for DepObjProps<OwnerType, OwnerId> {
    fn drop(&mut self) {
        if !self.storage.is_null() {
            for &(offset, drop_entry) in &self.drop {
                unsafe { drop_entry(self.storage.offset(offset)) };
            }
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

#[macro_export]
macro_rules! dep_obj {
    ( $(#[$($a:tt)+])* struct $name:ident 
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ $(,)?>)?
        : $type_:ident as $id:ty {
            $(
               $field:ident : $field_type:ty = $val:expr
            ),+
            $(,)?
        }) => {
        dep_obj! {
            @impl builder [$(#[$($a)+])*] () $name : $type_ as $id ;
            [] [] [] [] [$($field : $field_type = $val),+];
            $(
                [ $( $lt ),+ ],
                [ $( $lt $( : $clt $(+ $dlt )* )? ),+ ]
            )?
        }
    };
    ( $(#[$($a:tt)+])* pub $(($($vis:tt)+))? struct $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ $(,)?>)?
        : $type_:ident as $id:ty {
            $(
               $field:ident : $field_type:ty = $val:expr
            ),+
            $(,)?
        }) => {
        dep_obj! {
            @impl builder [$(#[$($a)+])*] (pub $(($($vis)+))?) $name : $type_ as $id ;
            [] [] [] [] [$($field : $field_type = $val),+];
            $(
                [ $( $lt ),+ ],
                [ $( $lt $( : $clt $(+ $dlt )* )? ),+ ]
            )?
        }
    };
    ( $(#[$($a:tt)+])* struct $name:ident 
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ $(,)?>)?
        : $type_:ident as $id:ty {
        }) => {
        dep_obj! {
            @impl builder [$(#[$($a)+])*] () $name : $type_ as $id ;
            [] [] [] [] [];
            $(
                [ $( $lt ),+ ],
                [ $( $lt $( : $clt $(+ $dlt )* )? ),+ ]
            )?
        }
    };
    ( $(#[$($a:tt)+])* pub $(($($vis:tt)+))? struct $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ $(,)?>)?
        : $type_:ident as $id:ty {
        }) => {
        dep_obj! {
            @impl builder [$(#[$($a)+])*] (pub $(($($vis)+))?) $name : $type_ as $id ;
            [] [] [] [] [];
            $(
                [ $( $lt ),+ ],
                [ $( $lt $( : $clt $(+ $dlt )* )? ),+ ]
            )?
        }
    };
    ( @impl $builder:ident [$(#[$($a:tt)+])*] ($($vis:tt)*) $name:ident : $type_:ident as $id:ty ;
        [$($s:tt)*] [$($p:tt)*] [$($c:tt)*] [$($l:tt)*] [$field:ident : $field_type:ty = $val:expr $(, $($tail:tt)+)?];
        $([ $($g:tt)+ ], [ $($r:tt)+ ])? ) => {
        dep_obj! {
            @impl $builder [$(#[$($a)+])*] ($($vis)*) $name : $type_ as $id ;
            [$field : $crate::DepPropRaw<$type_, $field_type>, $($s)*]
            [
                pub fn $field $(< $($g)+ >)? (&self) -> $crate::DepProp<$name $(< $($r)+ >)?, $field_type> {
                    self.$field.owned_by() 
                }
                $($p)*
            ]
            [
                let $field = $builder.prop(|| $val);
                $($c)*
            ]
            [$field, $($l)*]
            [$($($tail)+)?];
            $([ $($g)+ ], [ $($r)+ ])?
        }
    };
    ( @impl $builder:ident [$(#[$($a:tt)+])*] ($($vis:tt)*) $name:ident : $type_:ident as $id:ty ;
        [$($s:tt)*] [$($p:tt)*] [$($c:tt)*] [$($l:tt)*] [];
        $([ $($g:tt)+ ], [ $($r:tt)+ ])? ) => {
        $($vis)* struct $type_ { $($s)* }

        unsafe impl $crate::DepType for $type_ {
            fn lock() -> &'static $crate::DepTypeLock {
                static LOCK: $crate::DepTypeLock = $crate::DepTypeLock::new();
                &LOCK
            }
        }

        impl $type_ {
            $($p)*

            fn new_raw() -> Option<$crate::DepTypeToken<Self>> {
                $crate::DepTypeBuilder::new().map(|mut $builder| {
                    $($c)*
                    $builder.build(Self { $($l)* })
                })
            }
        }

        $(#[$($a)+])*
        $($vis)* struct $name $(< $($g)+ >)? {
            dep_props: $crate::DepObjProps<$type_, $id>,
        }

        impl $(< $($g)+ >)? $crate::DepObj for $name $(< $($r)+ >)? {
            type Type = $type_;
            type Id = $id;
            fn dep_props(&self) -> &$crate::DepObjProps<Self::Type, Self::Id> { &self.dep_props }
            fn dep_props_mut(&mut self) -> &mut $crate::DepObjProps<Self::Type, Self::Id> { &mut self.dep_props }
        }

        impl $(< $($g)+ >)? $name $(< $($r)+ >)? {
            fn new_raw(token: &$crate::DepTypeToken<$type_>) -> Self {
                Self { dep_props: $crate::DepObjProps::new(token) }
            }
        }
    };
}
