#![feature(const_fn)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(raw_ref_macros)]
#![feature(shrink_to)]
#![feature(try_reserve)]
#![feature(unchecked_math)]
#![deny(warnings)]

#![cfg_attr(not(test), no_std)]
#[cfg(test)]
extern crate core;
extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use core::fmt::Debug;
use core::mem::replace;
use components_arena::ComponentId;
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::Context;
use educe::Educe;
use phantom_type::PhantomType;

#[doc(hidden)]
pub use core::default::Default as std_default_Default;
#[doc(hidden)]
pub use core::fmt::Debug as std_fmt_Debug;
#[doc(hidden)]
pub use core::option::Option as std_option_Option;
#[doc(hidden)]
pub use dyn_context::Context as dyn_context_Context;
#[doc(hidden)]
pub use dyn_context::ContextExt as dyn_context_ContextExt;
#[doc(hidden)]
pub use generics::parse as generics_parse;
#[doc(hidden)]
pub use memoffset::offset_of as memoffset_offset_of;
#[doc(hidden)]
pub use paste::paste as paste_paste;

pub trait DepPropType: Clone + Debug + Send + Sync + 'static { }

impl<PropType: Clone + Debug + Send + Sync + 'static> DepPropType for PropType { }

#[derive(Educe)]
#[educe(Debug)]
pub struct DepEntry<OwnerId: ComponentId, PropType: DepPropType> {
    default: &'static PropType,
    style: Option<PropType>,
    local: Option<PropType>,
    #[educe(Debug(ignore))]
    on_changed: Vec<fn(context: &mut dyn Context, id: OwnerId, old: &PropType)>,
}

pub struct DepPropOnChanged<OwnerId: ComponentId, PropType: DepPropType> {
    callbacks: Vec<fn(context: &mut dyn Context, id: OwnerId, old: &PropType)>,
}

impl<OwnerId: ComponentId, PropType: DepPropType> DepPropOnChanged<OwnerId, PropType> {
    pub fn raise(self, context: &mut dyn Context, id: OwnerId, old: &PropType) {
        for callback in self.callbacks {
            callback(context, id, old);
        }
    }
}

impl<OwnerId: ComponentId, PropType: DepPropType> DepEntry<OwnerId, PropType> {
    pub const fn new(default: &'static PropType) -> Self {
        DepEntry {
            default,
            style: None,
            local: None,
            on_changed: Vec::new()
        }
    }
}

pub trait DepType: Sized {
    type Id: ComponentId;

    #[doc(hidden)]
    fn style__(&mut self) -> &mut Option<Style<Self>>;
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepProp<Owner: DepType, PropType: DepPropType> {
    offset: usize,
    _phantom: PhantomType<(Owner, PropType)>
}


impl<Owner: DepType, PropType: DepPropType> DepProp<Owner, PropType> {
    pub const unsafe fn new(offset: usize) -> Self {
        DepProp { offset, _phantom: PhantomType::new() }
    }

    pub fn offset(self) -> usize { self.offset }

    fn entry(self, owner: &Owner) -> &DepEntry<Owner::Id, PropType> {
        unsafe {
            let entry = (owner as *const _ as usize).unchecked_add(self.offset);
            let entry = entry as *const DepEntry<Owner::Id, PropType>;
            &*entry
        }
    }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepEntry<Owner::Id, PropType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepEntry<Owner::Id, PropType>;
            &mut *entry
        }
    }

    fn get_non_local(self, owner: &Owner) -> &PropType {
        let entry = self.entry(owner);
        entry.style.as_ref().unwrap_or(entry.default)
    }

    fn get_local(self, owner: &Owner) -> Option<&PropType> {
        let entry = self.entry(owner);
        entry.local.as_ref()
    }

    pub fn get(self, owner: &Owner) -> &PropType {
        self.get_local(owner).unwrap_or_else(|| self.get_non_local(owner))
    }

    pub fn set_uncond(
        self,
        owner: &mut Owner,
        value: Option<PropType>
    ) -> (PropType, DepPropOnChanged<Owner::Id, PropType>) {
        let entry_mut = self.entry_mut(owner);
        let on_changed = DepPropOnChanged { callbacks: entry_mut.on_changed.clone() };
        let old = replace(&mut entry_mut.local, value).unwrap_or_else(|| self.get_non_local(owner).clone());
        (old, on_changed)
    }

    pub fn on_changed(
        self,
        owner: &mut Owner,
        callback: fn(context: &mut dyn Context, id: Owner::Id, old: &PropType),
    ) {
        let entry_mut = self.entry_mut(owner);
        entry_mut.on_changed.push(callback);
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct Setter<Owner: DepType, PropType: DepPropType> {
    prop: DepProp<Owner, PropType>,
    value: PropType,
}

trait AnySetter<Owner: DepType>: Debug + DynClone + Send + Sync {
    fn prop_offset(&self) -> usize;
    fn un_apply(
        &self,
        owner: &mut Owner,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn Context, Owner::Id)>>;
}

clone_trait_object!(<Owner: DepType> AnySetter<Owner>);

impl<Owner: DepType, PropType: DepPropType> AnySetter<Owner> for Setter<Owner, PropType> where Owner::Id: 'static {
    fn prop_offset(&self) -> usize { self.prop.offset }

    fn un_apply(
        &self,
        owner: &mut Owner,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn Context, Owner::Id)>> {
        let entry_mut = self.prop.entry_mut(owner);
        let on_changed = if entry_mut.local.is_some() {
            None
        } else {
            Some(DepPropOnChanged { callbacks: entry_mut.on_changed.clone() })
        };
        let value = if unapply { None } else { Some(self.value.clone()) };
        let old = replace(&mut entry_mut.style, value);
        on_changed.map(|on_changed| {
            let old = old.unwrap_or_else(|| entry_mut.default.clone());
            Box::new(move |context: &'_ mut dyn Context, id| on_changed.raise(context, id, &old)) as _
        })
    }
}

#[derive(Educe)]
#[educe(Debug, Clone, Default)]
pub struct Style<Owner: DepType> {
    setters: Vec<Box<dyn AnySetter<Owner>>>,
}

impl<Owner: DepType> Style<Owner> {
    pub fn new() -> Self { Style { setters: Vec::new() } }

    pub fn with_capacity(capacity: usize) -> Self { Style { setters: Vec::with_capacity(capacity) } }

    pub fn capacity(&self) -> usize { self.setters.capacity() }

    pub fn clear(&mut self) { self.setters.clear(); }

    pub fn contains_prop<PropType: DepPropType>(&self, prop: DepProp<Owner, PropType>) -> bool {
        self.setters.binary_search_by_key(&prop.offset, |x| x.prop_offset()).is_ok()
    }

    pub fn insert<PropType: DepPropType>(
        &mut self,
        prop: DepProp<Owner, PropType>,
        value: PropType
    ) -> bool where Owner: 'static {
        let setter = Box::new(Setter { prop, value });
        match self.setters.binary_search_by_key(&prop.offset, |x| x.prop_offset()) {
            Ok(index) => { self.setters[index] = setter; true }
            Err(index) => { self.setters.insert(index, setter); false }
        }
    }

    pub fn is_empty(&self) -> bool { self.setters.is_empty() }

    pub fn len(&self) -> usize { self.setters.len() }

    pub fn remove<PropType: DepPropType>(&mut self, prop: DepProp<Owner, PropType>) -> bool {
        match self.setters.binary_search_by_key(&prop.offset, |x| x.prop_offset()) {
            Ok(index) => { self.setters.remove(index); true }
            Err(_) => false
        }
    }

    pub fn reserve(&mut self, additional: usize) { self.setters.reserve(additional) }

    pub fn shrink_to(&mut self, min_capacity: usize) { self.setters.shrink_to(min_capacity) }

    pub fn shrink_to_fit(&mut self) { self.setters.shrink_to_fit() }

    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.setters.try_reserve(additional)
    }
}

pub struct StyleOnChanged<OwnerId: ComponentId> {
    callbacks: Vec<Box<dyn FnOnce(&mut dyn Context, OwnerId)>>
}

impl<OwnerId: ComponentId> StyleOnChanged<OwnerId> {
    pub fn raise(self, context: &mut dyn Context, id: OwnerId) {
        for callback in self.callbacks {
            callback(context, id);
        }
    }
}

impl<Owner: DepType> Style<Owner> {
    fn un_apply(
        owner: &mut Owner,
        new_style: Option<Self>
    ) -> (Option<Style<Owner>>, StyleOnChanged<Owner::Id>) {
        let mut on_changed = Vec::new();
        let old_style = owner.style__().take();
        if let Some(old_style) = old_style.as_ref() {
            old_style.setters
                .iter()
                .filter(|setter| new_style.as_ref().map_or(
                    true,
                    |new_style| new_style.setters.binary_search_by_key(
                        &setter.prop_offset(),
                        |x| x.prop_offset()
                    ).is_err()
                ))
                .filter_map(|setter| setter.un_apply(owner, true))
                .for_each(|x| on_changed.push(x))
            ;
        }
        if let Some(new_style) = new_style.as_ref() {
            new_style.setters
                .iter()
                .filter_map(|setter| setter.un_apply(owner, false))
                .for_each(|x| on_changed.push(x))
            ;
        }
        *owner.style__() = new_style;
        (old_style, StyleOnChanged { callbacks: on_changed })
    }

    pub fn apply(self, owner: &mut Owner) -> (Option<Style<Owner>>, StyleOnChanged<Owner::Id>) {
        Self::un_apply(owner, Some(self))
    }

    pub fn unapply(owner: &mut Owner) -> (Option<Style<Owner>>, StyleOnChanged<Owner::Id>) {
        Self::un_apply(owner, None)
    }
}

#[macro_export]
macro_rules! dep_type {
    (
        $(#[$attr:meta])* $vis:vis struct $name:ident $($body:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type {
                @parsed_generics
                $(#[$attr])* $vis struct $name
            }
            $($body)*
        }
    };
    (
        @parsed_generics
        $(#[$attr:meta])* $vis:vis struct $name:ident
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty
        {
            $($($field:ident $delim:tt $field_ty:ty $(= $field_val:expr)?),+ $(,)?)?
        }
    ) => {
        $crate::dep_type! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] []
            [$($([$field $delim $field_ty $(= $field_val)?])+)?]
        }
    };
    (
        @parsed_generics
        $(#[$attr:meta])* $vis:vis struct $name:ident
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($body:tt)*
    ) => {
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [[$field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepEntry<$Id, $field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepEntry::new(&Self:: [< $field:upper _DEFAULT >] ),
            ]
            [
                $($core_consts)*
                const [< $field:upper _DEFAULT >] : $field_ty = $field_val;
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepProp<Self, $field_ty> = {
                    unsafe { 
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepProp::new(offset)
                    }
                };
            ]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [[$field:ident $delim:tt $field_ty:ty $(= $field_val:expr)?] $($fields:tt)*]
    ) => {
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        []
    ) => {
        $crate::paste_paste! {
            #[derive($crate::std_fmt_Debug)]
            struct [< $name Core >] $($g)* $($w)* {
                dep_type_core_style: $crate::std_option_Option<$crate::Style<$name $($r)*>>,
                $($core_fields)*
            }

            impl $($g)* [< $name Core >] $($r)* $($w)* {
                const fn new() -> Self {
                    Self {
                        dep_type_core_style: $crate::std_option_Option::None,
                        $($core_new)*
                    }
                }

                $($core_consts)*
            }

            impl $($g)* $crate::std_default_Default for [< $name Core >] $($r)* $($w)* {
                fn default() -> Self { Self::new() }
            }

            $(#[$attr])*
            $vis struct $name $($g)* $($w)* {
                core: [< $name Core >] $($r)*
            }

            impl $($g)* $name $($r)* $($w)* {
                const fn new_priv() -> Self {
                    Self { core: [< $name Core >] ::new() }
                }

                $($dep_props)*
            }

            impl $($g)* $crate::DepType for $name $($r)* $($w)* {
                type Id = $Id;

                fn style__(&mut self) -> &mut $crate::std_option_Option<$crate::Style<$name $($r)*>> {
                    &mut self.core.dep_type_core_style
                }
            }
        }
    };
}

#[macro_export]
macro_rules! dep_obj {
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident: $Arena:ty) -> $ty:ty {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            $vis fn [< $name _get >] <DepObjValueType: $crate::DepPropType>(
                self,
                $arena: &$Arena,
                prop: $crate::DepProp<$ty, DepObjValueType>
            ) -> &DepObjValueType {
                let $this = self;
                let obj = $field;
                prop.get(obj)
            }

            $vis fn [< $name _set_uncond >] <DepObjValueType: $crate::DepPropType>(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                prop: $crate::DepProp<$ty, DepObjValueType>,
                value: DepObjValueType,
            ) -> DepObjValueType {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get_mut::<$Arena>(context);
                let obj = $field_mut;
                let (old, on_changed) = prop.set_uncond(obj, Some(value));
                on_changed.raise(context, self, &old);
                old
            }

            $vis fn [< $name _on_changed >] <DepObjValueType: $crate::DepPropType>(
                self,
                $arena: &mut $Arena,
                prop: $crate::DepProp<$ty, DepObjValueType>,
                on_changed: fn(
                    context: &mut dyn $crate::dyn_context_Context,
                    id: Self,
                    old: &DepObjValueType
                ),
            ) {
                let $this = self;
                let obj = $field_mut;
                prop.on_changed(obj, on_changed);
            }

            $vis fn [< $name _apply_style >] (
                self,
                context: &mut dyn $crate::dyn_context_Context,
                style: $crate::Style<$ty>,
            ) -> $crate::std_option_Option<$crate::Style<$ty>> {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get_mut::<$Arena>(context);
                let obj = $field_mut;
                let (old, on_changed) = style.apply(obj);
                on_changed.raise(context, self);
                old
            }

        }
    };
    (
        $vis:vis dyn fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $ty:tt {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            $vis fn [< $name _get >] <
                Owner: $ty + $crate::DepType<Id=Self>,
                DepObjValueType: $crate::DepPropType
            > (
                self,
                $arena: &$Arena,
                prop: $crate::DepProp<Owner, DepObjValueType>
            ) -> &DepObjValueType {
                let $this = self;
                let obj = $field.downcast_ref::<Owner>().expect("invalid cast");
                prop.get(obj)
            }

            $vis fn [< $name _set_uncond >] <
                Owner: $ty + $crate::DepType<Id=Self>,
                DepObjValueType: $crate::DepPropType
            >(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                prop: $crate::DepProp<Owner, DepObjValueType>,
                value: DepObjValueType,
            ) -> DepObjValueType {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get_mut::<$Arena>(context);
                let obj = $field_mut.downcast_mut::<Owner>().expect("invalid cast");
                let (old, on_changed) = prop.set_uncond(obj, value);
                on_changed.raise(context, self, &old);
                old
            }

            $vis fn [< $name _on_changed >] <
                Owner: $ty + $crate::DepType<Id=Self>,
                DepObjValueType: $crate::DepPropType
            >(
                self,
                $arena: &mut $Arena,
                prop: $crate::DepProp<Owner, DepObjValueType>,
                on_changed: fn(
                    context: &mut dyn $crate::dyn_context_Context,
                    id: Self,
                    old: &DepObjValueType
                ),
            ) {
                let $this = self;
                let obj = $field_mut.downcast_mut::<Owner>().expect("invalid cast");
                prop.on_changed(obj, on_changed);
            }

            $vis fn [< $name _apply_style >] <
                Owner: $ty + $crate::DepType<Id=Self>,
            >(
                self,
                context: &mut dyn $crate::dyn_context_Context,
                style: $crate::Style<Owner>,
            ) -> $crate::std_option_Option<$crate::Style<Owner>> {
                let $this = self;
                let $arena = $crate::dyn_context_ContextExt::get_mut::<$Arena>(context);
                let obj = $field_mut.downcast_mut::<Owner>().expect("invalid cast");
                let (old, on_changed) = style.apply(obj);
                on_changed.raise(context, self);
                old
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::boxed::Box;
    use components_arena::{Arena, Id, ComponentId, Component, ComponentClassMutex};
    use dyn_context::Context;
    use educe::Educe;
    use macro_attr_2018::macro_attr;

    macro_attr! {
        #[derive(Debug, Component!)]
        struct TestNode {
            obj1: Option<Box<TestObj1>>,
        }
    }

    static TEST_NODE: ComponentClassMutex<TestNode> = ComponentClassMutex::new();

    macro_attr! {
        #[derive(Educe, ComponentId!)]
        #[educe(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        struct TestId(Id<TestNode>);
    }

    impl TestId {
        dep_obj! {
            pub fn obj1(self as this, arena: TestArena) -> TestObj1 {
                if mut {
                    arena.0[this.0].obj1.as_mut().unwrap()
                } else {
                    arena.0[this.0].obj1.as_ref().unwrap()
                }
            }
        }
    }

    macro_attr! {
        #[derive(Context!)]
        struct TestArena(Arena<TestNode>);
    }


    dep_type! {
        #[derive(Debug)]
        struct TestObj1 become obj1 in TestId {
            int_val: i32 = 42,
        }
    }

    impl TestObj1 {
        pub fn new(arena: &mut TestArena, id: TestId) {
            arena.0[id.0].obj1 = Some(Box::new(TestObj1::new_priv()));
        }
    }

    #[test]
    fn create_test_obj_1() {
        let mut arena = TestArena(Arena::new(&mut TEST_NODE.lock().unwrap()));
        let id = arena.0.insert(|id| (TestNode { obj1: None }, TestId(id)));
        TestObj1::new(&mut arena, id);
        assert_eq!(id.obj1_get(&arena, TestObj1::INT_VAL), &42);
        id.obj1_on_changed(&mut arena, TestObj1::INT_VAL, |_, _, _| { });
        id.obj1_set_uncond(&mut arena, TestObj1::INT_VAL, 43);
        assert_eq!(id.obj1_get(&arena, TestObj1::INT_VAL), &43);
    }

    #[test]
    fn test_obj_1_style() {
        let mut arena = TestArena(Arena::new(&mut TEST_NODE.lock().unwrap()));
        let id = arena.0.insert(|id| (TestNode { obj1: None }, TestId(id)));
        TestObj1::new(&mut arena, id);
        assert_eq!(id.obj1_get(&arena, TestObj1::INT_VAL), &42);
        let mut style = Style::new();
        style.insert(TestObj1::INT_VAL, 43);
        id.obj1_apply_style(&mut arena, style.clone());
        assert_eq!(id.obj1_get(&arena, TestObj1::INT_VAL), &43);
        id.obj1_set_uncond(&mut arena, TestObj1::INT_VAL, 44);
        assert_eq!(id.obj1_get(&arena, TestObj1::INT_VAL), &44);
        style.insert(TestObj1::INT_VAL, 45);
        id.obj1_apply_style(&mut arena, style);
        assert_eq!(id.obj1_get(&arena, TestObj1::INT_VAL), &44);
    }
}
