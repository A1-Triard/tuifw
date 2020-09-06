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
use educe::Educe;

pub trait Context {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any>;
    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any>;
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

pub struct DepTypeToken<OwnerType: DepType> {
    layout: Layout,
    default: Vec<(isize, unsafe fn(usize, *mut u8), usize)>,
    drop: Vec<(isize, unsafe fn(*mut u8))>,
    events: usize,
    ty: OwnerType,
}

impl<Type: DepType> DepTypeToken<Type> {
    pub fn ty(&self) -> &Type { &self.ty }
}

pub unsafe trait DepType {
    fn lock() -> &'static DepTypeLock;
}

pub trait DepObj {
    type Type: DepType;
    type Id: ComponentId;
    fn core(&self) -> &DepObjCore<Self::Type, Self::Id> where Self: Sized;
    fn core_mut(&mut self) -> &mut DepObjCore<Self::Type, Self::Id> where Self: Sized;
}

#[derive(Debug)]
struct Entry<PropType> {
    value: PropType,
    on_changed: Option<Vec<usize>>,
}

unsafe fn store_default<PropType>(fn_ptr: usize, props: *mut u8) {
    let fn_ptr: fn() -> PropType = transmute(fn_ptr);
    ptr::write(props as *mut Entry<PropType>, Entry { value: fn_ptr(), on_changed: None });
}

unsafe fn drop_entry<PropType>(props: *mut u8) {
    ptr::drop_in_place(props as *mut Entry<PropType>);
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepTypeBuilder<OwnerType: DepType> {
    align: usize,
    size: usize,
    default: Vec<(isize, unsafe fn(usize, *mut u8), usize)>,
    drop: Vec<(isize, unsafe fn(*mut u8))>,
    events: usize,
    phantom: PhantomData<OwnerType>,
}

unsafe impl<OwnerType: DepType> Send for DepTypeBuilder<OwnerType> { }
unsafe impl<OwnerType: DepType> Sync for DepTypeBuilder<OwnerType> { }
impl<OwnerType: DepType> Unpin for DepTypeBuilder<OwnerType> { }

impl<OwnerType: DepType> DepTypeBuilder<OwnerType> {
    pub fn new() -> Option<Self> {
        let lock = OwnerType::lock();
        if lock.0.compare_and_swap(false, true, Ordering::Relaxed) {
            None
        } else {
            Some(DepTypeBuilder {
                size: 0,
                align: 1,
                default: Vec::new(),
                drop: Vec::new(),
                events: 0,
                phantom: PhantomData
            })
        }
    }

    pub fn prop<PropType>(&mut self, default: fn() -> PropType) -> DepPropRaw<OwnerType, PropType> {
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

    pub fn event<ArgsType>(&mut self) -> DepEventRaw<OwnerType, ArgsType> {
        let index = self.events;
        self.events = index.checked_add(1).expect("out of memory");
        DepEventRaw { index, phantom: (PhantomData, PhantomData) }
    }

    pub fn build(mut self, ty: OwnerType) -> DepTypeToken<OwnerType> {
        self.default.shrink_to_fit();
        self.drop.shrink_to_fit();
        DepTypeToken {
            layout: Layout::from_size_align(self.size, self.align).expect("out of memory"),
            default: self.default,
            drop: self.drop,
            events: self.events,
            ty
        }
    }
}

pub struct OnRaised<OwnerId: ComponentId, ArgsType>(
    Option<Vec<usize>>,
    (PhantomData<OwnerId>, PhantomData<ArgsType>),
);

impl<OwnerId: ComponentId, ArgsType> OnRaised<OwnerId, ArgsType> {
    pub fn raise(self, owner_id: OwnerId, context: &mut dyn Context, args: &mut ArgsType) {
        if let Some(on_raised) = self.0 {
            for on_raised in on_raised {
                let on_raised: fn(owner_id: OwnerId, context: &mut dyn Context, args: &mut ArgsType) =
                    unsafe { transmute(on_raised) };
                on_raised(owner_id, context, args);
            }
        }
    }
}

#[derive(Educe)]
#[educe(Debug, Copy, Clone, Eq, PartialEq)]
#[educe(Hash, Ord, PartialOrd)]
pub struct DepEventRaw<OwnerType: DepType, ArgsType> {
    index: usize,
    phantom: (PhantomData<OwnerType>, PhantomData<ArgsType>),
}

unsafe impl<OwnerType: DepType, ArgsType> Send for DepEventRaw<OwnerType, ArgsType> { }
unsafe impl<OwnerType: DepType, ArgsType> Sync for DepEventRaw<OwnerType, ArgsType> { }
impl<OwnerType: DepType, ArgsType> Unpin for DepEventRaw<OwnerType, ArgsType> { }

impl<OwnerType: DepType, ArgsType> DepEventRaw<OwnerType, ArgsType> {
    pub fn owned_by<Owner: DepObj<Type=OwnerType>>(self) -> DepEvent<Owner, ArgsType> {
        DepEvent(self, PhantomData)
    }
}

#[derive(Educe)]
#[educe(Debug, Copy, Clone, Eq, PartialEq)]
#[educe(Hash, Ord, PartialOrd)]
pub struct DepEvent<Owner: DepObj, ArgsType>(
    DepEventRaw<Owner::Type, ArgsType>,
    PhantomData<Owner>,
);

unsafe impl<Owner: DepObj, ArgsType> Send for DepEvent<Owner, ArgsType> { }
unsafe impl<Owner: DepObj, ArgsType> Sync for DepEvent<Owner, ArgsType> { }
impl<Owner: DepObj, ArgsType> Unpin for DepEvent<Owner, ArgsType> { }

impl<Owner: DepObj, ArgsType> DepEvent<Owner, ArgsType> {
    pub fn on_raised(
        self,
        obj: &mut Owner,
        callback: fn(owner_id: Owner::Id, context: &mut dyn Context, args: &mut ArgsType)
    ) {
        let callback = unsafe { transmute(callback) };
        let on_raised = unsafe { obj.core_mut().events.get_unchecked_mut(self.0.index) };
        if let Some(on_raised) = on_raised.as_mut() {
            on_raised.push(callback);
        } else {
            on_raised.replace(vec![callback]);
        }
    }

    pub fn raise(
        self,
        obj: &mut Owner,
    ) -> OnRaised<Owner::Id, ArgsType> {
        let on_raised = unsafe { obj.core_mut().events.get_unchecked(self.0.index) };
        OnRaised(on_raised.clone(), (PhantomData, PhantomData))
    }
}

pub struct OnChanged<OwnerId: ComponentId, PropType>(
    Option<Vec<usize>>,
    (PhantomData<OwnerId>, PhantomData<PropType>),
);

impl<OwnerId: ComponentId, PropType> OnChanged<OwnerId, PropType> {
    pub fn raise(self, owner_id: OwnerId, context: &mut dyn Context, old: &PropType) {
        if let Some(on_changed) = self.0 {
            for on_changed in on_changed {
                let on_changed: fn(owner_id: OwnerId, context: &mut dyn Context, old: &PropType) =
                    unsafe { transmute(on_changed) };
                on_changed(owner_id, context, old);
            }
        }
    }
}

#[derive(Educe)]
#[educe(Debug, Copy, Clone, Eq, PartialEq)]
#[educe(Hash, Ord, PartialOrd)]
pub struct DepPropRaw<OwnerType: DepType, PropType> {
    offset: isize,
    phantom: (PhantomData<OwnerType>, PhantomData<PropType>),
}

unsafe impl<OwnerType: DepType, PropType> Send for DepPropRaw<OwnerType, PropType> { }
unsafe impl<OwnerType: DepType, PropType> Sync for DepPropRaw<OwnerType, PropType> { }
impl<OwnerType: DepType, PropType> Unpin for DepPropRaw<OwnerType, PropType> { }

impl<OwnerType: DepType, PropType> DepPropRaw<OwnerType, PropType> {
    pub fn owned_by<Owner: DepObj<Type=OwnerType>>(self) -> DepProp<Owner, PropType> {
        DepProp(self, PhantomData)
    }

    fn get_entry<OwnerId: ComponentId>(
        self,
        obj_props: &DepObjCore<OwnerType, OwnerId>
    ) -> &Entry<PropType> {
        unsafe { &*(obj_props.props.offset(self.offset) as *const Entry<PropType>) }
    }

    fn get_entry_mut<OwnerId: ComponentId>(
        self,
        obj_props: &mut DepObjCore<OwnerType, OwnerId>
    ) -> &mut Entry<PropType> {
        unsafe { &mut *(obj_props.props.offset(self.offset) as *mut Entry<PropType>) }
    }
}

#[derive(Educe)]
#[educe(Debug, Copy, Clone, Eq, PartialEq)]
#[educe(Hash, Ord, PartialOrd)]
pub struct DepProp<Owner: DepObj, PropType>(
    DepPropRaw<Owner::Type, PropType>,
    PhantomData<Owner>,
);

unsafe impl<Owner: DepObj, PropType> Send for DepProp<Owner, PropType> { }
unsafe impl<Owner: DepObj, PropType> Sync for DepProp<Owner, PropType> { }
impl<Owner: DepObj, PropType> Unpin for DepProp<Owner, PropType> { }

impl<Owner: DepObj, PropType: Eq> DepProp<Owner, PropType> {
    pub fn set_distinct(
        self,
        obj: &mut Owner,
        value: PropType
    ) -> (PropType, OnChanged<Owner::Id, PropType>) {
        let entry = self.0.get_entry_mut(obj.core_mut());
        let old = replace(&mut entry.value, value);
        let on_changed = if old == entry.value { None } else { entry.on_changed.clone() };
        (old, OnChanged(on_changed, (PhantomData, PhantomData)))
    }
}

impl<Owner: DepObj, PropType> DepProp<Owner, PropType> {
    pub fn set_uncond(
        self,
        obj: &mut Owner,
        value: PropType
    ) -> (PropType, OnChanged<Owner::Id, PropType>) {
        let entry = self.0.get_entry_mut(obj.core_mut());
        let old = replace(&mut entry.value, value);
        (old, OnChanged(entry.on_changed.clone(), (PhantomData, PhantomData)))
    }

    pub fn get(
        self,
        obj: &Owner
    ) -> &PropType {
        &self.0.get_entry(obj.core()).value
    }

    pub fn on_changed(
        self,
        obj: &mut Owner,
        callback: fn(owner_id: Owner::Id, context: &mut dyn Context, old: &PropType)
    ) {
        let callback = unsafe { transmute(callback) };
        let entry = self.0.get_entry_mut(obj.core_mut());
        if let Some(on_changed) = entry.on_changed.as_mut() {
            on_changed.push(callback);
        } else {
            entry.on_changed = Some(vec![callback]);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepObjCore<OwnerType: DepType, OwnerId: ComponentId> {
    layout: Layout,
    props: *mut u8,
    drop: Vec<(isize, unsafe fn(*mut u8))>,
    events: Vec<Option<Vec<usize>>>,
    phantom: (PhantomData<OwnerType>, PhantomData<OwnerId>)
}

unsafe impl<OwnerType: DepType, OwnerId: ComponentId> Send for DepObjCore<OwnerType, OwnerId> { }
unsafe impl<OwnerType: DepType, OwnerId: ComponentId> Sync for DepObjCore<OwnerType, OwnerId> { }
impl<OwnerType: DepType, OwnerId: ComponentId> Unpin for DepObjCore<OwnerType, OwnerId> { }

impl<OwnerType: DepType, OwnerId: ComponentId> DepObjCore<OwnerType, OwnerId> {
    pub fn new(tytoken: &DepTypeToken<OwnerType>) -> DepObjCore<OwnerType, OwnerId> {
        let props = if tytoken.layout.size() == 0 {
            null_mut()
        } else {
            NonNull::new(unsafe { alloc(tytoken.layout) }).expect("out of memory").as_ptr()
        };
        for &(offset, store, fn_ptr) in &tytoken.default {
            unsafe { store(fn_ptr, props.offset(offset)) };
        }
        DepObjCore {
            layout: tytoken.layout,
            props,
            drop: tytoken.drop.clone(),
            events: vec![None; tytoken.events],
            phantom: (PhantomData, PhantomData)
        }
    }
}

impl<OwnerType: DepType, OwnerId: ComponentId> Drop for DepObjCore<OwnerType, OwnerId> {
    fn drop(&mut self) {
        if !self.props.is_null() {
            for &(offset, drop_entry) in &self.drop {
                unsafe { drop_entry(self.props.offset(offset)) };
            }
            unsafe { dealloc(self.props, self.layout) };
            self.props = null_mut();
        }
    }
}

#[macro_export]
macro_rules! DepType {
    (
        ()
        $vis:vis enum $name:ident $($tail:tt)+
    ) => {
        DepType! {
            @impl $name
        }
    };
    (
        ()
        $vis:vis struct $name:ident $($tail:tt)+
    ) => {
        DepType! {
            @impl $name
        }
    };
    (
        @impl $name:ident
    ) => {
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
    (
        $(#[$attr:meta])* $vis:vis struct $name:ident
        $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ $(,)?>)?
        as $id:ty : $ty:ident {
            $($(
               $field:ident $delim:tt $field_ty:ty $(= $field_val:expr)?
            ),+ $(,)?)?
        }
    ) => {
        dep_obj! {
            @impl builder [$(#[$attr])*] ($vis) $name as $id : $ty ;
            [] [] [] [] [$($($field $delim $field_ty $(= $field_val)?),+)?];
            $(
                [ $( $lt ),+ ],
                [ $( $lt $( : $clt $(+ $dlt )* )? ),+ ]
            )?
        }
    };
    (
        @impl $builder:ident [$(#[$attr:meta])*] ($vis:vis) $name:ident as $id:ty : $ty:ident ;
        [$($s:tt)*]
        [$($p:tt)*]
        [$($c:tt)*]
        [$($l:tt)*]
        [$field:ident : $field_ty:ty = $field_val:expr $(, $($other_fields:tt)+)?];
        $([ $($g:tt)+ ], [ $($r:tt)+ ])?
    ) => {
        dep_obj! {
            @impl $builder [$(#[$attr])*] ($vis) $name as $id : $ty ;
            [
                $($s)*
                $field : $crate::DepPropRaw<$ty, $field_ty>,
            ]
            [
                $($p)*
                pub fn $field $(< $($g)+ >)? (&self) -> $crate::DepProp<$name $(< $($r)+ >)?, $field_ty> {
                    self.$field.owned_by() 
                }
            ]
            [
                $($c)*
                let $field = $builder.prop(|| $field_val);
            ]
            [
                $($l)*
                $field,
            ]
            [$($($other_fields)+)?];
            $([ $($g)+ ], [ $($r)+ ])?
        }
    };
    (
        @impl $builder:ident [$(#[$attr:meta])*] ($vis:vis) $name:ident as $id:ty : $ty:ident ;
        [$($s:tt)*]
        [$($p:tt)*]
        [$($c:tt)*]
        [$($l:tt)*]
        [$field:ident yield $field_ty:ty $(, $($other_fields:tt)+)?];
        $([ $($g:tt)+ ], [ $($r:tt)+ ])?
    ) => {
        dep_obj! {
            @impl $builder [$(#[$attr])*] ($vis) $name as $id : $ty ;
            [
                $($s)*
                $field : $crate::DepEventRaw<$ty, $field_ty>,
            ]
            [
                $($p)*
                pub fn $field $(< $($g)+ >)? (&self) -> $crate::DepEvent<$name $(< $($r)+ >)?, $field_ty> {
                    self.$field.owned_by()
                }
            ]
            [
                $($c)*
                let $field = $builder.event();
            ]
            [
                $($l)*
                $field,
            ]
            [$($($other_fields)+)?];
            $([ $($g)+ ], [ $($r)+ ])?
        }
    };
    (
        @impl $builder:ident [$(#[$attr:meta])*] ($vis:vis) $name:ident as $id:ty : $ty:ident ;
        [$($s:tt)*] [$($p:tt)*] [$($c:tt)*] [$($l:tt)*] [];
        $([ $($g:tt)+ ], [ $($r:tt)+ ])?
    ) => {
        $vis struct $ty { $($s)* }

        unsafe impl $crate::DepType for $ty {
            fn lock() -> &'static $crate::DepTypeLock {
                static LOCK: $crate::DepTypeLock = $crate::DepTypeLock::new();
                &LOCK
            }
        }

        impl $ty {
            $($p)*

            fn new_raw() -> Option<$crate::DepTypeToken<Self>> {
                $crate::DepTypeBuilder::new().map(|mut $builder| {
                    $($c)*
                    $builder.build(Self { $($l)* })
                })
            }
        }

        $(#[$attr])*
        $vis struct $name $(< $($g)+ >)? {
            core: $crate::DepObjCore<$ty, $id>,
        }

        impl $(< $($g)+ >)? $crate::DepObj for $name $(< $($r)+ >)? {
            type Type = $ty;
            type Id = $id;
            fn core(&self) -> &$crate::DepObjCore<Self::Type, Self::Id> { &self.core }
            fn core_mut(&mut self) -> &mut $crate::DepObjCore<Self::Type, Self::Id> { &mut self.core }
        }

        impl $(< $($g)+ >)? $name $(< $($r)+ >)? {
            fn new_raw(token: &$crate::DepTypeToken<$ty>) -> Self {
                Self { core: $crate::DepObjCore::new(token) }
            }
        }
    };
}
