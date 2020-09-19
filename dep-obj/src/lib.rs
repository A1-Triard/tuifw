#![feature(const_fn)]
#![feature(unchecked_math)]
#![deny(warnings)]

#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::replace;
use components_arena::ComponentId;
use dyn_context::Context;
use educe::Educe;

#[doc(hidden)]
pub use core::default::Default as std_default_Default;
#[doc(hidden)]
pub use core::fmt::Debug as std_fmt_Debug;
#[doc(hidden)]
pub use generics::parse as generics_parse;
#[doc(hidden)]
pub use paste::paste as paste_paste;

pub trait DepPropType: Debug + Clone + 'static { }

impl<PropType: Debug + Clone + 'static> DepPropType for PropType { }

#[derive(Educe)]
#[educe(Debug)]
pub struct DepObjEntry<OwnerId: ComponentId, PropType: DepPropType> {
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

impl<OwnerId: ComponentId, PropType: DepPropType> DepObjEntry<OwnerId, PropType> {
    pub const fn new(default: &'static PropType) -> Self {
        DepObjEntry {
            default,
            style: None,
            local: None,
            on_changed: Vec::new()
        }
    }
}

pub trait DepObj {
    type Id: ComponentId;
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepProp<Owner: DepObj, PropType: DepPropType> {
    offset: usize,
    phantom: (PhantomData<Owner>, PhantomData<PropType>)
}

impl<Owner: DepObj, PropType: DepPropType> DepProp<Owner, PropType> {
    pub const unsafe fn new(offset: usize) -> Self {
        DepProp { offset, phantom: (PhantomData, PhantomData) }
    }

    fn entry(self, owner: &Owner) -> &DepObjEntry<Owner::Id, PropType> {
        unsafe {
            let entry = (owner as *const _ as usize).unchecked_add(self.offset);
            let entry = entry as *const DepObjEntry<Owner::Id, PropType>;
            &*entry
        }
    }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepObjEntry<Owner::Id, PropType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepObjEntry<Owner::Id, PropType>;
            &mut *entry
        }
    }

    pub fn get(self, owner: &Owner) -> &PropType {
        let entry = self.entry(owner);
        entry.local.as_ref().or_else(|| entry.style.as_ref()).unwrap_or(entry.default)
    }

    pub fn set_uncond(
        self,
        owner: &mut Owner,
        value: Option<PropType>
    ) -> (Option<PropType>, DepPropOnChanged<Owner::Id, PropType>) {
        let entry_mut = self.entry_mut(owner);
        let old = replace(&mut entry_mut.local, value);
        let on_changed = DepPropOnChanged { callbacks: entry_mut.on_changed.clone() };
        (old, on_changed)
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
            [] [] []
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
        [$($dep_props:tt)*]
        [[$field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepObjEntry<$Id, $field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepObjEntry::new(&Self:: [< $field:upper _DEFAULT >] ),
            ]
            [
                $($dep_props)*

                const [< $field:upper _DEFAULT >] : $field_ty = $field_val;

                $vis const [< $field:upper >] : $crate::DepProp<Self, $field_ty> = {
                    let offset = &raw const (0usize as *const [< $name Core >] $($r)*).$field as usize;
                    unsafe { $crate::DepProp::new(offset) }
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
        [$($dep_props:tt)*]
        []
    ) => {
        $crate::paste_paste! {
            #[derive($crate::std_fmt_Debug)]
            struct [< $name Core >] $($g)* $($w)* {
                $($core_fields)*
            }

            impl $($g)* [< $name Core >] $($r)* $($w)* {
                const fn new() -> Self {
                    Self {
                        $($core_new)*
                    }
                }
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

            impl $($g)* $crate::DepObj for $name $($r)* $($w)* {
                type Id = $Id;
            }
        }
    };
}

macro_rules! dep_obj {
    (
    ) => {
    };
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;
    use components_arena::{Arena, Id, ComponentId, Component, ComponentClassToken};
    use educe::Educe;
    use macro_attr_2018::macro_attr;

    macro_attr! {
        #[derive(Debug, Component!)]
        struct TestNode {
            obj1: Option<Box<TestObj1>>,
        }
    }

    macro_attr! {
        #[derive(Educe, ComponentId!)]
        #[educe(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        struct TestId(Id<TestNode>);
    }

    dep_type! {
        #[derive(Debug)]
        struct TestObj1 become obj1 in TestId {
        }
    }

    impl TestObj1 {
        pub fn new(arena: &mut Arena<TestNode>, id: TestId) {
            arena[id.0].obj1 = Some(Box::new(TestObj1::new_priv()));
        }
    }

    #[test]
    fn create_test_obj_1() {
        let mut test_node_token = ComponentClassToken::new().unwrap();
        let mut arena = Arena::new(&mut test_node_token);
        let id = arena.insert(|id| (TestNode { obj1: None }, TestId(id)));
        TestObj1::new(&mut arena, id);
    }
}
