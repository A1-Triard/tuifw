#![deny(warnings)]

#![no_std]
extern crate alloc;

use alloc::alloc::{alloc, dealloc, Layout};
use alloc::{vec};
use alloc::vec::{Vec};
use core::cmp::max;
use core::convert::TryInto;
use core::marker::PhantomData;
use core::mem::{replace, align_of, size_of, transmute};
use core::ptr::{self, NonNull, null_mut};
use core::sync::atomic::{AtomicBool, Ordering};
use components_arena::ComponentId;
use educe::Educe;
use dyn_context::Context;

#[doc(hidden)]
pub use paste::paste as paste_paste;
#[doc(hidden)]
pub use core::compile_error as std_compile_error;
#[doc(hidden)]
pub use core::stringify as std_stringify;
#[doc(hidden)]
pub use core::concat as std_concat;
#[doc(hidden)]
pub use dyn_context::Context as dyn_context_Context;
#[doc(hidden)]
pub use dyn_context::ContextExt as dyn_context_ContextExt;
#[doc(hidden)]
pub use core::option::Option as std_option_Option;
#[doc(hidden)]
pub use alloc::vec::Vec as std_vec_Vec;
#[doc(hidden)]
pub use core::cmp::Eq as std_cmp_Eq;
#[doc(hidden)]
pub use core::ops::FnOnce as std_ops_FnOnce;

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
    on_changed: Vec<usize>,
}

unsafe fn store_default<PropType>(fn_ptr: usize, props: *mut u8) {
    let fn_ptr: fn() -> PropType = transmute(fn_ptr);
    ptr::write(props as *mut Entry<PropType>, Entry { value: fn_ptr(), on_changed: Vec::new() });
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
    Vec<usize>,
    (PhantomData<OwnerId>, PhantomData<ArgsType>),
);

impl<OwnerId: ComponentId, ArgsType> OnRaised<OwnerId, ArgsType> {
    pub fn raise(self, context: &mut dyn Context, owner_id: OwnerId, args: &mut ArgsType) {
        for on_raised in self.0 {
            let on_raised: fn(context: &mut dyn Context, owner_id: OwnerId, args: &mut ArgsType) =
                unsafe { transmute(on_raised) };
            on_raised(context, owner_id, args);
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
        callback: fn(context: &mut dyn Context, owner_id: Owner::Id, args: &mut ArgsType)
    ) {
        let callback = unsafe { transmute(callback) };
        let on_raised = unsafe { obj.core_mut().events.get_unchecked_mut(self.0.index) };
        on_raised.push(callback);
    }

    pub fn raise(
        self,
        obj: &Owner,
    ) -> OnRaised<Owner::Id, ArgsType> {
        let on_raised = unsafe { obj.core().events.get_unchecked(self.0.index) };
        OnRaised(on_raised.clone(), (PhantomData, PhantomData))
    }
}

pub struct OnChanged<OwnerId: ComponentId, PropType>(
    Vec<usize>,
    (PhantomData<OwnerId>, PhantomData<PropType>),
);

impl<OwnerId: ComponentId, PropType> OnChanged<OwnerId, PropType> {
    pub fn raise(self, context: &mut dyn Context, owner_id: OwnerId, old: &PropType) {
        for on_changed in self.0 {
            let on_changed: fn(context: &mut dyn Context, owner_id: OwnerId, old: &PropType) =
                unsafe { transmute(on_changed) };
            on_changed(context, owner_id, old);
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
        let on_changed = if old == entry.value { Vec::new() } else { entry.on_changed.clone() };
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
        callback: fn(context: &mut dyn Context, owner_id: Owner::Id, old: &PropType)
    ) {
        let callback = unsafe { transmute(callback) };
        let entry = self.0.get_entry_mut(obj.core_mut());
        entry.on_changed.push(callback);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepObjCore<OwnerType: DepType, OwnerId: ComponentId> {
    layout: Layout,
    props: *mut u8,
    drop: Vec<(isize, unsafe fn(*mut u8))>,
    events: Vec<Vec<usize>>,
    phantom: (PhantomData<OwnerType>, PhantomData<OwnerId>)
}

unsafe impl<OwnerType: DepType, OwnerId: ComponentId> Send for DepObjCore<OwnerType, OwnerId> { }
unsafe impl<OwnerType: DepType, OwnerId: ComponentId> Sync for DepObjCore<OwnerType, OwnerId> { }
impl<OwnerType: DepType, OwnerId: ComponentId> Unpin for DepObjCore<OwnerType, OwnerId> { }

impl<OwnerType: DepType, OwnerId: ComponentId> DepObjCore<OwnerType, OwnerId> {
    pub fn new(ty_token: &DepTypeToken<OwnerType>) -> DepObjCore<OwnerType, OwnerId> {
        let props = if ty_token.layout.size() == 0 {
            null_mut()
        } else {
            NonNull::new(unsafe { alloc(ty_token.layout) }).expect("out of memory").as_ptr()
        };
        for &(offset, store, fn_ptr) in &ty_token.default {
            unsafe { store(fn_ptr, props.offset(offset)) };
        }
        DepObjCore {
            layout: ty_token.layout,
            props,
            drop: ty_token.drop.clone(),
            events: vec![Vec::new(); ty_token.events],
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
            @impl [$name]
        }
    };
    (
        ()
        $vis:vis struct $name:ident $($tail:tt)+
    ) => {
        DepType! {
            @impl [$name]
        }
    };
    (
        @impl [$name:ident]
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
        become $system:ident in $Id:ty
        $(where BuilderCore $(< $( $bc_lt:tt $( : $bc_clt:tt $(+ $bc_dlt:tt )* )? ),+ $(,)?>)? = $BuilderCore:ty)? {
            $($(
               $field:ident $field_delim:tt $field_ty:ty $(= $field_val:expr)?
            ),+ $(,)?)?
        }
    ) => {
        $crate::dep_obj! {
            @impl
            [builder]
            [$(#[$attr])*] [$vis] [$name] [$system] [$Id]
            [ $( < $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? ]
            [ $( < $( $lt ),+ >)? ]
            [$(
                [$BuilderCore]
                [ $( < $( $bc_lt $( : $bc_clt $(+ $bc_dlt )* )? ),+ >)? ]
                [ $( < $( $bc_lt ),+ >)? ]
                []
            )?]
            [] [] [] []
            [$($($field $field_delim $field_ty $(= $field_val)?),+)?]
        }
    };
    (
        @impl 
        [$builder:ident]
        [$(#[$attr:meta])*] [$vis:vis] [$name:ident] [$system:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*]
        [$(
            [$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [$($type_fields:tt)*]
        [$($type_methods:tt)*]
        [$($type_init:tt)*]
        [$($type_bundle:tt)*]
        [$field:ident : $field_ty:ty = $field_val:expr $(, $($other_fields:tt)+)?]
    ) => {
        $crate::dep_obj! {
            @impl 
            [$builder]
            [$(#[$attr])*] [$vis] [$name] [$system] [$Id]
            [$($g)*] [$($r)*]
            [$(
                [$BuilderCore] [$($bc_g)*] [$($bc_r)*]
                [
                    $($builder_methods)*

                    $vis fn $field(&mut self, val : $field_ty) -> &mut Self {
                        let id = self.id;
                        let context = self.core.context();
                        let ty = unsafe { &*self.ty };
                        id . [< $system _set_uncond >] (context, ty.$field(), val);
                        self
                    }

                    $vis fn [< on_ $field _changed >] (
                        &mut self,
                        callback : fn(context: &mut dyn $crate::dyn_context_Context, owner: $Id, old: &$field_ty)
                    ) -> &mut Self {
                        let id = self.id;
                        let context = self.core.context();
                        let ty = unsafe { &*self.ty };
                        let arena = $crate::dyn_context_ContextExt::get_mut(context);
                        id . [< $system _on_changed >] (arena, ty.$field(), callback);
                        self
                    }
                ]
            )?]
            [
                $($type_fields)*
                $field : $crate::DepPropRaw< [< $name Type >] , $field_ty>,
            ]
            [
                $($type_methods)*
                $vis fn $field $($g)* (&self) -> $crate::DepProp<$name $($r)*, $field_ty> {
                    self.$field.owned_by() 
                }
            ]
            [
                $($type_init)*
                let $field = $builder.prop(|| $field_val);
            ]
            [
                $($type_bundle)*
                $field,
            ]
            [$($($other_fields)+)?]
        }
    };
    (
        @impl 
        [$builder:ident]
        [$(#[$attr:meta])*] [$vis:vis] [$name:ident] [$system:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*]
        [$(
            [$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [$($type_fields:tt)*]
        [$($type_methods:tt)*]
        [$($type_init:tt)*]
        [$($type_bundle:tt)*]
        [$field:ident yield $field_ty:ty $(, $($other_fields:tt)+)?]
    ) => {
        $crate::dep_obj! {
            @impl 
            [$builder]
            [$(#[$attr])*] [$vis] [$name] [$system] [$Id]
            [$($g)*] [$($r)*]
            [$(
                [$BuilderCore] [$($bc_g)*] [$($bc_r)*]
                [
                    $($builder_methods)*

                    $vis fn [< on_ $field >] (
                        &mut self,
                        callback : fn(context: &mut dyn $crate::dyn_context_Context, owner: $Id, args: &mut $field_ty)
                    ) -> &mut Self {
                        let id = self.id;
                        let context = self.core.context();
                        let ty = unsafe { &*self.ty };
                        let arena = $crate::dyn_context_ContextExt::get_mut(context);
                        id . [< $system _on >] (arena, ty.$field(), callback);
                        self
                    }
                ]
            )?]
            [
                $($type_fields)*
                $field : $crate::DepEventRaw< [< $name Type >] , $field_ty>,
            ]
            [
                $($type_methods)*
                $vis fn $field $($g)* (&self) -> $crate::DepEvent<$name $($r)*, $field_ty> {
                    self.$field.owned_by()
                }
            ]
            [
                $($type_init)*
                let $field = $builder.event();
            ]
            [
                $($type_bundle)*
                $field,
            ]
            [$($($other_fields)+)?]
        }
    };
    (
        @impl 
        [$builder:ident]
        [$(#[$attr:meta])*] [$vis:vis] [$name:ident] [$system:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*]
        [$(
            [$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [$($type_fields:tt)*]
        [$($type_methods:tt)*]
        [$($type_init:tt)*]
        [$($type_bundle:tt)*]
        [$field:ident $field_delim:tt $field_ty:ty $(= $field_val:expr)? $(, $($other_fields:tt)+)?]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dependency object field '",
            $crate::std_stringify!($field $field_delim $field_ty $(= $field_val)?),
            "', allowed forms are '$field: $type = $value', and '$event yield $args'",
        ));
    };
    (
        @impl 
        [$builder:ident]
        [$(#[$attr:meta])*] [$vis:vis] [$name:ident] [$system:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*]
        [$(
            [$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [$($type_fields:tt)*]
        [$($type_methods:tt)*]
        [$($type_init:tt)*]
        [$($type_bundle:tt)*]
        []
    ) => {
        $crate::paste_paste! {
            $(
                $vis struct [< $name Builder >] $($bc_g)* {
                    core: $BuilderCore,
                    id: $Id,
                    ty: *const [< $name Type >],
                }

                impl $($bc_g)* [< $name Builder >] $($bc_r)* {
                    $($builder_methods)*

                    fn build_priv(
                        core: $BuilderCore,
                        id: $Id,
                        ty: & [< $name Type >] ,
                        f: impl $crate::std_ops_FnOnce(&mut Self) -> &mut Self
                    ) {
                        let mut builder = Self {
                            core,
                            id,
                            ty: ty as *const _,
                        };
                        f(&mut builder);
                    }

                    fn core_priv(&self) -> &$BuilderCore { &self.core }

                    fn core_priv_mut(&mut self) -> &mut $BuilderCore { &mut self.core }
                }
            )?

            $vis struct [< $name Type >] {
                $($type_fields)*
            }

            unsafe impl $crate::DepType for [< $name Type >] {
                fn lock() -> &'static $crate::DepTypeLock {
                    static LOCK: $crate::DepTypeLock = $crate::DepTypeLock::new();
                    &LOCK
                }
            }

            impl [< $name Type >] {
                $($type_methods)*

                fn new_priv() -> $crate::std_option_Option<$crate::DepTypeToken<Self>> {
                    $crate::DepTypeBuilder::new().map(|mut $builder| {
                        $($type_init)*
                        $builder.build(Self {
                            $($type_bundle)*
                        })
                    })
                }
            }

            $(#[$attr])*
            $vis struct $name $($g)* {
                core: $crate::DepObjCore< [< $name Type  >] , $Id>,
            }

            impl $($g)* $crate::DepObj for $name $($r)* {
                type Type = [< $name Type >] ;
                type Id = $Id;
                fn core(&self) -> &$crate::DepObjCore<Self::Type, Self::Id> { &self.core }
                fn core_mut(&mut self) -> &mut $crate::DepObjCore<Self::Type, Self::Id> { &mut self.core }
            }

            impl $($g)* $name $($r)* {
                fn new_priv(token: &$crate::DepTypeToken< [< $name Type >] >) -> Self {
                    Self { core: $crate::DepObjCore::new(token) }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! dep_system {
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $System:ty {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            $vis fn [< $name _get >] <DepSystemValueType>(
                self,
                $arena: &$Arena,
                prop: $crate::DepProp<$System, DepSystemValueType>
            ) -> &DepSystemValueType {
                let $this = self;
                let system = $field;
                prop.get(system)
            }

            $vis fn [< $name _set_uncond >] <DepSystemValueType>(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                prop: $crate::DepProp<$System, DepSystemValueType>,
                value: DepSystemValueType,
            ) -> DepSystemValueType {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get_mut::<$Arena>(context);
                let system = $field_mut;
                let (old, on_changed) = prop.set_uncond(system, value);
                on_changed.raise(context, self, &old);
                old
            }

            $vis fn [< $name _set_distinct >] <DepSystemValueType: $crate::std_cmp_Eq>(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                prop: $crate::DepProp<$System, DepSystemValueType>,
                value: DepSystemValueType,
            ) -> DepSystemValueType {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get_mut::<$Arena>(context);
                let system = $field_mut;
                let (old, on_changed) = prop.set_distinct(system, value);
                on_changed.raise(context, self, &old);
                old
            }

            $vis fn [< $name _on_changed >] <DepSystemValueType>(
                self,
                $arena: &mut $Arena,
                prop: $crate::DepProp<$System, DepSystemValueType>,
                on_changed: fn(context: &mut dyn $crate::dyn_context_Context, owner: Self, old: &DepSystemValueType),
            ) {
                let $this = self;
                let system = $field_mut;
                prop.on_changed(system, on_changed);
            }

            $vis fn [< $name _raise >] <DepSystemArgsType>(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                event: $crate::DepEvent<$System, DepSystemArgsType>,
                args: &mut DepSystemArgsType,
            ) {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get::<$Arena>(context);
                let system = $field;
                let on_raised = event.raise(system);
                on_raised.raise(context, self, args);
            }

            $vis fn [< $name _on >] <DepSystemArgsType>(
                self,
                $arena: &mut $Arena,
                event: $crate::DepEvent<$System, DepSystemArgsType>,
                on_raised: fn(context: &mut dyn $crate::dyn_context_Context, owner: Self, args: &mut DepSystemArgsType),
            ) {
                let $this = self;
                let system = $field_mut;
                event.on_raised(system, on_raised);
            }
        }
    };
    (
        $vis:vis dyn fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $System:tt {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            $vis fn [< $name _get >] <DepSystemType: $System + $crate::DepObj<Id=Self>, DepSystemValueType>(
                self,
                $arena: &$Arena,
                prop: $crate::DepProp<DepSystemType, DepSystemValueType>
            ) -> &DepSystemValueType {
                let $this = self;
                let system = $field.downcast_ref::<DepSystemType>().expect("invalid cast");
                prop.get(system)
            }

            $vis fn [< $name _set_uncond >] <DepSystemType: $System + $crate::DepObj<Id=Self>, DepSystemValueType>(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                prop: $crate::DepProp<DepSystemType, DepSystemValueType>,
                value: DepSystemValueType,
            ) -> DepSystemValueType {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get_mut::<$Arena>(context);
                let system = $field_mut.downcast_mut::<DepSystemType>().expect("invalid cast");
                let (old, on_changed) = prop.set_uncond(system, value);
                on_changed.raise(context, self, &old);
                old
            }

            $vis fn [< $name _set_distinct >] <
                DepSystemType: $System + $crate::DepObj<Id=Self>, DepSystemValueType: $crate::std_cmp_Eq
            >(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                prop: $crate::DepProp<DepSystemType, DepSystemValueType>,
                value: DepSystemValueType,
            ) -> DepSystemValueType {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get_mut::<$Arena>(context);
                let system = $field_mut.downcast_mut::<DepSystemType>().expect("invalid cast");
                let (old, on_changed) = prop.set_distinct(system, value);
                on_changed.raise(context, self, &old);
                old
            }

            $vis fn [< $name _on_changed >] <DepSystemType: $System + $crate::DepObj<Id=Self>, DepSystemValueType>(
                self,
                $arena: &mut $Arena,
                prop: $crate::DepProp<DepSystemType, DepSystemValueType>,
                on_changed: fn(context: &mut dyn $crate::dyn_context_Context, owner: Self, old: &DepSystemValueType),
            ) {
                let $this = self;
                let system = $field_mut.downcast_mut::<DepSystemType>().expect("invalid cast");
                prop.on_changed(system, on_changed);
            }

            $vis fn [< $name _raise >] <DepSystemType: $System + $crate::DepObj<Id=Self>, DepSystemArgsType>(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                event: $crate::DepEvent<DepSystemType, DepSystemArgsType>,
                args: &mut DepSystemArgsType,
            ) {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get::<$Arena>(context);
                let system = $field.downcast_ref::<DepSystemType>().expect("invalid cast");
                let on_raised = event.raise(system);
                on_raised.raise(context, self, args);
            }

            $vis fn [< $name _on >] <DepSystemType: $System + $crate::DepObj<Id=Self>, DepSystemArgsType>(
                self,
                $arena: &mut $Arena,
                event: $crate::DepEvent<DepSystemType, DepSystemArgsType>,
                on_raised: fn(context: &mut dyn $crate::dyn_context_Context, owner: Self, args: &mut DepSystemArgsType),
            ) {
                let $this = self;
                let system = $field_mut.downcast_mut::<DepSystemType>().expect("invalid cast");
                event.on_raised(system, on_raised);
            }
        }
    };
}
