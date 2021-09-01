#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn_trait_bound)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_mut_refs)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(try_reserve)]
#![feature(unchecked_math)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::type_complexity)]

#![no_std]

extern crate alloc;

mod base;
pub use base::*;

pub mod binding;

#[cfg(docsrs)]
pub mod example {
    //! The [`dep_type`] and [`dep_obj`] macro expansion example.
    //!
    //! ```ignore
    //! dep_type! {
    //!     #[derive(Debug)]
    //!     pub struct MyDepType in MyDepTypeId {
    //!         prop_1: bool = false,
    //!         prop_2: i32 = 10,
    //!     }
    //! }
    //!
    //! macro_attr! {
    //!     #[derive(Component!, Debug)]
    //!     struct MyDepTypePrivateData {
    //!         dep_data: MyDepType,
    //!     }
    //! }
    //!
    //! macro_attr! {
    //!     #[derive(NewtypeComponentId!, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    //!     pub struct MyDepTypeId(Id<MyDepTypePrivateData>);
    //! }
    //!
    //! macro_attr! {
    //!     #[derive(State!, Debug)]
    //!     pub struct MyApp {
    //!         my_dep_types: Arena<MyDepTypePrivateData>,
    //!     }
    //! }
    //!
    //! impl MyDepTypeId {
    //!     pub fn new(state: &mut dyn State) -> MyDepTypeId {
    //!         let app: &mut MyApp = state.get_mut();
    //!         app.my_dep_types.insert(|id| (MyDepTypePrivateData {
    //!             dep_data: MyDepType::new_priv()
    //!         }, MyDepTypeId(id)))
    //!     }
    //!
    //!     pub fn drop_my_dep_type(self, state: &mut dyn State) {
    //!         self.drop_bindings_priv(state);
    //!         let app: &mut MyApp = state.get_mut();
    //!         app.my_dep_types.remove(self.0);
    //!     }
    //!
    //!     dep_obj! {
    //!         pub fn obj(self as this, app: MyApp) -> (MyDepType) {
    //!             if mut {
    //!                 &mut app.my_dep_types[this.0].dep_data
    //!             } else {
    //!                 &app.my_dep_types[this.0].dep_data
    //!             }
    //!         }
    //!     }
    //! }

    use crate::{dep_obj, dep_type};
    use components_arena::{Arena, Component, Id, NewtypeComponentId};
    use dyn_context::state::{SelfState, State, StateExt};

    dep_type! {
        #[derive(Debug)]
        pub struct MyDepType in MyDepTypeId {
            prop_1: bool = false,
            prop_2: i32 = 10,
        }
    }

    #[derive(Debug)]
    struct MyDepTypePrivateData {
        dep_data: MyDepType,
    }

    Component!(() struct MyDepTypePrivateData { .. });

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct MyDepTypeId(Id<MyDepTypePrivateData>);

    NewtypeComponentId!(() pub struct MyDepTypeId(Id<MyDepTypePrivateData>););

    #[derive(Debug)]
    pub struct MyApp {
        my_dep_types: Arena<MyDepTypePrivateData>,
    }

    impl SelfState for MyApp { }

    impl MyDepTypeId {
        pub fn new(state: &mut dyn State) -> MyDepTypeId {
            let app: &mut MyApp = state.get_mut();
            app.my_dep_types.insert(|id| (MyDepTypePrivateData {
                dep_data: MyDepType::new_priv()
            }, MyDepTypeId(id)))
        }

        pub fn drop_my_dep_type(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let app: &mut MyApp = state.get_mut();
            app.my_dep_types.remove(self.0);
        }

        dep_obj! {
            pub fn obj(self as this, app: MyApp) -> (MyDepType) {
                if mut {
                    &mut app.my_dep_types[this.0].dep_data
                } else {
                    &app.my_dep_types[this.0].dep_data
                }
            }
        }
    }
}

#[doc(hidden)]
pub use alloc::vec::Vec as std_vec_Vec;
#[doc(hidden)]
pub use alloc::boxed::Box as std_boxed_Box;
#[doc(hidden)]
pub use core::any::Any as std_any_Any;
#[doc(hidden)]
pub use core::any::TypeId as std_any_TypeId;
#[doc(hidden)]
pub use core::compile_error as std_compile_error;
#[doc(hidden)]
pub use core::concat as std_concat;
#[doc(hidden)]
pub use core::convert::From as std_convert_From;
#[doc(hidden)]
pub use core::default::Default as std_default_Default;
#[doc(hidden)]
pub use core::fmt::Debug as std_fmt_Debug;
#[doc(hidden)]
pub use core::mem::take as std_mem_take;
#[doc(hidden)]
pub use core::option::Option as std_option_Option;
#[doc(hidden)]
pub use core::stringify as std_stringify;
#[doc(hidden)]
pub use dyn_context::state::State as dyn_context_state_State;
#[doc(hidden)]
pub use dyn_context::state::StateExt as dyn_context_state_StateExt;
#[doc(hidden)]
pub use generics::concat as generics_concat;
#[doc(hidden)]
pub use generics::parse as generics_parse;
#[doc(hidden)]
pub use memoffset::offset_of as memoffset_offset_of;
#[doc(hidden)]
pub use paste::paste as paste_paste;

use crate::binding::*;
use alloc::boxed::Box;
use alloc::collections::TryReserveError;
use alloc::vec;
use alloc::vec::Vec;
use components_arena::{Arena, Component, ComponentId, Id};
use core::any::{Any, TypeId};
use core::fmt::Debug;
use core::mem::{replace, take};
use core::ops::{Deref, DerefMut};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::state::State;
use educe::Educe;
use macro_attr_2018::macro_attr;
use phantom_type::PhantomType;

pub struct GlobDescriptor<Id: ComponentId, Obj> {
    pub arena: TypeId,
    pub field_ref: fn(arena: &dyn Any, id: Id) -> &Obj,
    pub field_mut: fn(arena: &mut dyn Any, id: Id) -> &mut Obj
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct Glob<Id: ComponentId, Obj> {
    pub id: Id,
    pub descriptor: fn() -> GlobDescriptor<Id, Obj>,
}

pub struct GlobMut<'a, Id: ComponentId, Obj> {
    pub arena: &'a mut dyn Any,
    pub glob: Glob<Id, Obj>,
}

impl<'a, Id: ComponentId, Obj> Deref for GlobMut<'a, Id, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        ((self.glob.descriptor)().field_ref)(self.arena.deref(), self.glob.id)
    }
}

impl<'a, Id: ComponentId, Obj> DerefMut for GlobMut<'a, Id, Obj> {
    fn deref_mut(&mut self) -> &mut Obj {
        ((self.glob.descriptor)().field_mut)(self.arena.deref_mut(), self.glob.id)
    }
}

impl<Id: ComponentId, Obj> Glob<Id, Obj> {
    pub fn get_mut(self, state: &mut dyn State) -> GlobMut<Id, Obj> {
        let arena = (self.descriptor)().arena;
        GlobMut {
            arena: state.get_mut_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            glob: self
        }
    }
}

macro_attr! {
    #[derive(Educe, Component!(class=ValueHandlerComponent))]
    #[educe(Debug, Clone)]
    struct BoxedValueHandler<T: Convenient>(Box<dyn ValueHandler<T>>);
}

macro_attr! {
    #[derive(Educe, Component!(class=EventHandlerComponent))]
    #[educe(Debug, Clone)]
    struct BoxedEventHandler<T>(Box<dyn EventHandler<T>>);
}

#[derive(Debug)]
pub struct DepPropEntry<PropType: Convenient> {
    default: &'static PropType,
    style: Option<PropType>,
    local: Option<PropType>,
    handlers: Arena<BoxedValueHandler<(PropType, PropType)>>,
    binding: Option<Binding<PropType>>,
}

impl<PropType: Convenient> DepPropEntry<PropType> {
    pub const fn new(default: &'static PropType) -> Self {
        DepPropEntry {
            default,
            style: None,
            local: None,
            handlers: Arena::new(),
            binding: None,
        }
    }

    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.handlers).into_items().into_values().map(|x| x.0.into_any()));
    }

    pub fn binding(&self) -> Option<Binding<PropType>> {
        self.binding
    }
}

#[derive(Debug)]
pub struct DepEventEntry<ArgsType> {
    handlers: Arena<BoxedEventHandler<ArgsType>>,
}

impl<ArgsType> DepEventEntry<ArgsType> {
    pub const fn new() -> Self {
        DepEventEntry {
            handlers: Arena::new(),
        }
    }

    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.handlers).into_items().into_values().map(|x| x.0.into_any()));
    }
}

#[derive(Debug)]
pub struct DepVecEntry<ItemType: Convenient> {
    items: Vec<ItemType>,
    removed_items_handlers: Arena<BoxedValueHandler<Vec<ItemType>>>,
    inserted_items_handlers: Arena<BoxedValueHandler<Vec<ItemType>>>,
}

impl<ItemType: Convenient> DepVecEntry<ItemType> {
    pub const fn new() -> Self {
        DepVecEntry {
            items: Vec::new(),
            removed_items_handlers: Arena::new(),
            inserted_items_handlers: Arena::new(),
        }
    }

    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.removed_items_handlers).into_items().into_values().map(|x| x.0.into_any()));
        handlers.extend(take(&mut self.inserted_items_handlers).into_items().into_values().map(|x| x.0.into_any()));
    }
}

#[derive(Debug)]
pub struct BaseDepObjCore<Owner: DepType> {
    style: Option<Style<Owner>>,
    parent: Option<Owner::Id>,
    added_bindings: Vec<AnyBinding>,
}

impl<Owner: DepType> BaseDepObjCore<Owner> {
    pub const fn new() -> Self {
        BaseDepObjCore {
            style: None,
            parent: None,
            added_bindings: Vec::new(),
        }
    }

    pub fn take_bindings(&mut self) -> Vec<AnyBinding> { take(&mut self.added_bindings) }
}

impl<Owner: DepType> Default for BaseDepObjCore<Owner> {
    fn default() -> Self { BaseDepObjCore::new() }
}

/// A dependency type.
/// Use the [`dep_type`] or the [`dep_type_with_builder`] macro
/// to create a type implementing this trait.
///
/// # Examples
///
/// ```rust
/// # #![feature(const_maybe_uninit_as_ptr)]
/// # #![feature(const_ptr_offset_from)]
/// # #![feature(const_raw_ptr_deref)]
/// use components_arena::{Arena, Component, NewtypeComponentId, Id};
/// use dep_obj::{dep_obj, dep_type};
/// use dep_obj::binding::{Bindings, Binding1};
/// use dyn_context::state::{State, StateExt};
/// use macro_attr_2018::macro_attr;
/// use std::any::{Any, TypeId};
///
/// dep_type! {
///     #[derive(Debug)]
///     pub struct MyDepType in MyDepTypeId {
///         prop_1: bool = false,
///         prop_2: i32 = 10,
///     }
/// }
///
/// macro_attr! {
///     #[derive(Component!, Debug)]
///     struct MyDepTypePrivateData {
///         dep_data: MyDepType,
///     }
/// }
///
/// macro_attr! {
///     #[derive(NewtypeComponentId!, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
///     pub struct MyDepTypeId(Id<MyDepTypePrivateData>);
/// }
///
/// pub struct MyApp {
///     bindings: Bindings,
///     my_dep_types: Arena<MyDepTypePrivateData>,
///     res: i32,
/// }
///
/// impl State for MyApp {
///     fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
///         if ty == TypeId::of::<Bindings>() {
///             Some(&self.bindings)
///         } else if ty == TypeId::of::<MyApp>() {
///             Some(self)
///         } else {
///             None
///         }
///     }
///
///     fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
///         if ty == TypeId::of::<Bindings>() {
///             Some(&mut self.bindings)
///         } else if ty == TypeId::of::<MyApp>() {
///             Some(self)
///         } else {
///             None
///         }
///     }
/// }
///
/// impl MyDepTypeId {
///     pub fn new(state: &mut dyn State) -> MyDepTypeId {
///         let app: &mut MyApp = state.get_mut();
///         app.my_dep_types.insert(|id| (MyDepTypePrivateData {
///             dep_data: MyDepType::new_priv()
///         }, MyDepTypeId(id)))
///     }
///
///     pub fn drop_my_dep_type(self, state: &mut dyn State) {
///         self.drop_bindings_priv(state);
///         let app: &mut MyApp = state.get_mut();
///         app.my_dep_types.remove(self.0);
///     }
///
///     dep_obj! {
///         pub fn obj(self as this, app: MyApp) -> (MyDepType) {
///             if mut {
///                 &mut app.my_dep_types[this.0].dep_data
///             } else {
///                 &app.my_dep_types[this.0].dep_data
///             }
///         }
///     }
/// }
///
/// fn main() {
///     let app = &mut MyApp {
///         bindings: Bindings::new(),
///         my_dep_types: Arena::new(),
///         res: 0,
///     };
///     let id = MyDepTypeId::new(app);
///     let res = Binding1::new(&mut app.bindings, (), |(), (_, x)| Some(x));
///     res.set_source_1(app, &mut MyDepType::PROP_2.source(id.obj()));
///     res.set_target_fn(app, (), |app, (), value| {
///         let app: &mut MyApp = app.get_mut();
///         app.res = value;
///     });
///     assert_eq!(app.res, 10);
///     MyDepType::PROP_2.set_distinct(app, id.obj(), 5);
///     assert_eq!(app.res, 5);
///     id.drop_my_dep_type(app);
///     res.drop_binding(app);
/// }
/// ```
pub trait DepType: Debug {
    type Id: ComponentId;

    #[doc(hidden)]
    fn core_base_priv(&self) -> &BaseDepObjCore<Self> where Self: Sized;

    #[doc(hidden)]
    fn core_base_priv_mut(&mut self) -> &mut BaseDepObjCore<Self> where Self: Sized;

    #[doc(hidden)]
    fn take_all_handlers(&mut self) -> Vec<Box<dyn AnyHandler>>;

    #[doc(hidden)]
    fn take_added_bindings_and_collect_all(&mut self) -> Vec<AnyBinding>;
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepEvent<Owner: DepType, ArgsType> {
    offset: usize,
    _phantom: PhantomType<(Owner, ArgsType)>
}

impl<Owner: DepType, ArgsType> DepEvent<Owner, ArgsType> {
    pub const unsafe fn new(offset: usize) -> Self {
        DepEvent { offset, _phantom: PhantomType::new() }
    }

    pub fn offset(self) -> usize { self.offset }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepEventEntry<ArgsType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepEventEntry<ArgsType>;
            &mut *entry
        }
    }

    pub fn raise(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, args: &mut ArgsType) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        let handlers = entry_mut.handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, args);
        }
    }

    pub fn source(self, obj: Glob<Owner::Id, Owner>) -> DepEventSource<Owner, ArgsType> {
        DepEventSource { obj, event: self }
    }
}

/// A dependency property.
#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepProp<Owner: DepType, PropType: Convenient> {
    offset: usize,
    _phantom: PhantomType<(Owner, PropType)>
}

impl<Owner: DepType, PropType: Convenient> DepProp<Owner, PropType> {
    /// Creates dependency property. The only safe way to call this function is through
    /// the [`dep_type`] or the [`dep_type_with_builder`] macro using.
    pub const unsafe fn new(offset: usize) -> Self {
        DepProp { offset, _phantom: PhantomType::new() }
    }

    pub fn offset(self) -> usize { self.offset }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepPropEntry<PropType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepPropEntry<PropType>;
            &mut *entry
        }
    }

    pub fn set_uncond(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, value: PropType) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        let old = entry_mut.local.replace(value.clone()).unwrap_or_else(||
            entry_mut.style.as_ref().unwrap_or(entry_mut.default).clone()
        );
        let handlers = entry_mut.handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, (old.clone(), value.clone()));
        }
    }

    pub fn unset_uncond(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        if let Some(old) = entry_mut.local.take() {
            let default = entry_mut.style.as_ref().unwrap_or(entry_mut.default).clone();
            let handlers = entry_mut.handlers.items().clone().into_values();
            for handler in handlers {
                handler.0.execute(state, (old.clone(), default.clone()));
            }
        }
    }

    pub fn set_distinct(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, value: PropType) where PropType: PartialEq {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        let old = entry_mut.local.as_ref()
            .or_else(|| entry_mut.style.as_ref())
            .unwrap_or(entry_mut.default)
        ;
        if &value == old { return; }
        let old = entry_mut.local.replace(value.clone()).unwrap_or_else(||
            entry_mut.style.as_ref().unwrap_or(entry_mut.default).clone());
        let handlers = entry_mut.handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, (old.clone(), value.clone()));
        }
    }

    pub fn unset_distinct(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>) where PropType: PartialEq {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        if let Some(old) = entry_mut.local.take() {
            let new = entry_mut.style.as_ref().unwrap_or(entry_mut.default).clone();
            if new == old { return; }
            let handlers = entry_mut.handlers.items().clone().into_values();
            for handler in handlers {
                handler.0.execute(state, (old.clone(), new.clone()));
            }
        }
    }

    fn bind_raw(
        self,
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        target: impl FnOnce(Self, Glob<Owner::Id, Owner>) -> Box<dyn Target<PropType>>,
        binding: Binding<PropType>
    ) {
        self.unbind(state, obj);
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.binding = Some(binding);
        binding.set_target(state, target(self, obj));
    }

    pub fn bind_distinct(
        self,
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        binding: impl Into<Binding<PropType>>
    ) where PropType: PartialEq, Owner: 'static {
        self.bind_raw(state, obj, |prop, obj| Box::new(DepPropSetDistinct { prop, obj }), binding.into());
    }

    pub fn bind_uncond(
        self,
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        binding: impl Into<Binding<PropType>>
    ) where Owner: 'static {
        self.bind_raw(state, obj, |prop, obj| Box::new(DepPropSetUncond { prop, obj }), binding.into());
    }

    pub fn unbind(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>) {
        if let Some(binding) = {
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            entry_mut.binding
        } {
            binding.drop_binding(state);
        }
    }

    fn clear_binding(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.binding.take();
    }

    pub fn source(self, obj: Glob<Owner::Id, Owner>) -> DepPropSource<Owner, PropType> {
        DepPropSource { obj, prop: self }
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepPropSetDistinct<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient + PartialEq> Target<PropType> for DepPropSetDistinct<Owner, PropType> {
    fn execute(&self, state: &mut dyn State, value: PropType) {
        self.prop.set_distinct(state, self.obj, value);
    }

    fn clear(&self, state: &mut dyn State) {
        self.prop.clear_binding(state, self.obj);
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepPropSetUncond<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> Target<PropType> for DepPropSetUncond<Owner, PropType> {
    fn execute(&self, state: &mut dyn State, value: PropType) {
        self.prop.set_uncond(state, self.obj, value);
    }

    fn clear(&self, state: &mut dyn State) {
        self.prop.clear_binding(state, self.obj);
    }
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepVec<Owner: DepType, ItemType: Convenient> {
    offset: usize,
    _phantom: PhantomType<(Owner, ItemType)>
}

impl<Owner: DepType, ItemType: Convenient> DepVec<Owner, ItemType> {
    pub const unsafe fn new(offset: usize) -> Self {
        DepVec { offset, _phantom: PhantomType::new() }
    }

    pub fn offset(self) -> usize { self.offset }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepVecEntry<ItemType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepVecEntry<ItemType>;
            &mut *entry
        }
    }

    pub fn clear(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        let items = take(&mut entry_mut.items);
        let handlers = entry_mut.removed_items_handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, items.clone());
        }
    }

    pub fn push(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, item: ItemType) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.items.push(item.clone());
        let handlers = entry_mut.inserted_items_handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, vec![item.clone()]);
        }
    }

    pub fn insert(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, index: usize, item: ItemType) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.items.insert(index, item.clone());
        let handlers = entry_mut.inserted_items_handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, vec![item.clone()]);
        }
    }

    pub fn remove(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, index: usize) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        let item = entry_mut.items.remove(index);
        let handlers = entry_mut.removed_items_handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, vec![item.clone()]);
        }
    }

    pub fn extend_from_slice(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, other: &[ItemType]) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.items.extend_from_slice(other);
        let handlers = entry_mut.inserted_items_handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, Vec::from(other));
        }
    }

    pub fn inserted_items_source(self, obj: Glob<Owner::Id, Owner>) -> DepVecInsertedItemSource<Owner, ItemType> {
        DepVecInsertedItemSource { obj, vec: self }
    }

    pub fn removed_items_source(self, obj: Glob<Owner::Id, Owner>) -> DepVecRemovedItemSource<Owner, ItemType> {
        DepVecRemovedItemSource { obj, vec: self }
    }
}

impl<Owner: DepType> Glob<Owner::Id, Owner> {
    pub fn add_binding(self, state: &mut dyn State, binding: AnyBinding) {
        let mut obj_mut = self.get_mut(state);
        obj_mut.core_base_priv_mut().added_bindings.push(binding);
    }

    pub fn apply_style(
        self,
        state: &mut dyn State,
        style: Option<Style<Owner>>,
    ) -> Option<Style<Owner>> {
        let mut on_changed = Vec::new();
        let obj = &mut self.get_mut(state);
        let old = obj.core_base_priv_mut().style.take();
        if let Some(old) = old.as_ref() {
            old.setters
                .iter()
                .filter(|setter| style.as_ref().map_or(
                    true,
                    |new| new.setters.binary_search_by_key(
                        &setter.prop_offset(),
                        |x| x.prop_offset()
                    ).is_err()
                ))
                .filter_map(|setter| setter.un_apply(obj, true))
                .for_each(|x| on_changed.push(x))
            ;
        }
        if let Some(new) = style.as_ref() {
            new.setters
                .iter()
                .filter_map(|setter| setter.un_apply(obj, false))
                .for_each(|x| on_changed.push(x))
            ;
        }
        obj.core_base_priv_mut().style = style;
        for on_changed in on_changed {
            on_changed(state);
        }
        old
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct Setter<Owner: DepType, PropType: Convenient> {
    prop: DepProp<Owner, PropType>,
    value: PropType,
}

trait AnySetter<Owner: DepType>: Debug + DynClone + Send + Sync {
    fn prop_offset(&self) -> usize;
    fn un_apply(
        &self,
        owner: &mut Owner,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn State)>>;
}

clone_trait_object!(<Owner: DepType> AnySetter<Owner>);

impl<Owner: DepType, PropType: Convenient> AnySetter<Owner> for Setter<Owner, PropType> where Owner::Id: 'static {
    fn prop_offset(&self) -> usize { self.prop.offset }

    fn un_apply(
        &self,
        owner: &mut Owner,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn State)>> {
        let entry_mut = self.prop.entry_mut(owner);
        let handlers = if entry_mut.local.is_some() {
            None
        } else {
            Some(entry_mut.handlers.items().clone())
        };
        let value = if unapply { None } else { Some(self.value.clone()) };
        let old = replace(&mut entry_mut.style, value.clone());
        if let Some(handlers) = handlers {
            let old = old.unwrap_or_else(|| entry_mut.default.clone());
            let value = value.unwrap_or_else(|| entry_mut.default.clone());
            Some(Box::new(move |state: &'_ mut dyn State| {
                for handler in handlers.into_values() {
                    handler.0.execute(state, (old.clone(), value.clone()));
                }
            }) as _)
        } else {
            None
        }
    }
}

/// A dictionary mapping a subset of target type properties to the values.
/// Every dependency object can have an applied style at every moment.
/// To switch an applied style, use the [`Glob::apply_style`] function.
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

    pub fn contains_prop<PropType: Convenient>(&self, prop: DepProp<Owner, PropType>) -> bool {
        self.setters.binary_search_by_key(&prop.offset, |x| x.prop_offset()).is_ok()
    }

    pub fn insert<PropType: Convenient>(
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

    pub fn remove<PropType: Convenient>(&mut self, prop: DepProp<Owner, PropType>) -> bool {
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

pub trait DepObjBaseBuilder<OwnerId: ComponentId> {
    fn state(&self) -> &dyn State;
    fn state_mut(&mut self) -> &mut dyn State;
    fn id(&self) -> OwnerId;
}

#[derive(Educe)]
#[educe(Debug)]
struct DepEventHandledSource<Owner: DepType, ArgsType> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<BoxedEventHandler<ArgsType>>,
    event: DepEvent<Owner, ArgsType>,
}

impl<Owner: DepType, ArgsType> HandlerId for DepEventHandledSource<Owner, ArgsType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.event.entry_mut(&mut obj);
        entry_mut.handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepEventSource<Owner: DepType, ArgsType> {
    obj: Glob<Owner::Id, Owner>,
    event: DepEvent<Owner, ArgsType>,
}

impl<Owner: DepType + 'static, ArgsType: 'static> EventSource<ArgsType> for DepEventSource<Owner, ArgsType> {
    fn handle(&self, state: &mut dyn State, handler: Box<dyn EventHandler<ArgsType>>) -> HandledEventSource<ArgsType> {
        let mut obj = self.obj.get_mut(state);
        let entry = self.event.entry_mut(&mut obj);
        let handler_id = entry.handlers.insert(|handler_id| (BoxedEventHandler(handler), handler_id));
        HandledEventSource {
            handler_id: Box::new(DepEventHandledSource { handler_id, obj: self.obj, event: self.event }),
            args: None // TODO "once" events with cached value?
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<BoxedValueHandler<(PropType, PropType)>>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        entry_mut.handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> ValueSource<(PropType, PropType)> for DepPropSource<Owner, PropType> {
    fn handle(&self, state: &mut dyn State, handler: Box<dyn ValueHandler<(PropType, PropType)>>) -> HandledValueSource<(PropType, PropType)> {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let old = entry.default.clone();
        let new = entry.local.as_ref().or_else(|| entry.style.as_ref()).unwrap_or(entry.default).clone();
        let handler_id = entry.handlers.insert(|handler_id| (BoxedValueHandler(handler), handler_id));
        HandledValueSource {
            handler_id: Box::new(DepPropHandledSource { handler_id, obj: self.obj, prop: self.prop }),
            value: (old, new)
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecInsertedItemsHandledSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<BoxedValueHandler<Vec<ItemType>>>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecInsertedItemsHandledSource<Owner, ItemType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.vec.entry_mut(&mut obj);
        entry_mut.inserted_items_handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecRemovedItemsHandledSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<BoxedValueHandler<Vec<ItemType>>>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecRemovedItemsHandledSource<Owner, ItemType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.vec.entry_mut(&mut obj);
        entry_mut.removed_items_handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecInsertedItemSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> ValueSource<Vec<ItemType>> for DepVecInsertedItemSource<Owner, ItemType> {
    fn handle(&self, state: &mut dyn State, handler: Box<dyn ValueHandler<Vec<ItemType>>>) -> HandledValueSource<Vec<ItemType>> {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let items = entry.items.clone();
        let handler_id = entry.inserted_items_handlers.insert(|handler_id| (BoxedValueHandler(handler), handler_id));
        HandledValueSource {
            handler_id: Box::new(DepVecInsertedItemsHandledSource { handler_id, obj: self.obj, vec: self.vec }),
            value: items
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecRemovedItemSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> ValueSource<Vec<ItemType>> for DepVecRemovedItemSource<Owner, ItemType> {
    fn handle(&self, state: &mut dyn State, handler: Box<dyn ValueHandler<Vec<ItemType>>>) -> HandledValueSource<Vec<ItemType>> {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let items = entry.items.clone();
        let handler_id = entry.removed_items_handlers.insert(|handler_id| (BoxedValueHandler(handler), handler_id));
        HandledValueSource {
            handler_id: Box::new(DepVecRemovedItemsHandledSource { handler_id, obj: self.obj, vec: self.vec }),
            value: items
        }
    }
}

#[macro_export]
macro_rules! dep_type_with_builder {
    (
        $($token:tt)*
    ) => {
        $crate::dep_type_with_builder_impl! { $($token)* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_type_with_builder_impl {
    (
        type BaseBuilder $($token:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @type BaseBuilder
            }
        }
        $($token)*
    };
    (
        @type BaseBuilder
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;

        $(#[$attr:meta])* $vis:vis struct $name:ident $($body:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @struct
                [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
                [$([$attr])*] [$vis] [$name]
            }
            $($body)*
        }
    };
    (
        @type BaseBuilder
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;

        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition; allowed form is \
            '$(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? \
            become $obj in $Id { ... }'\
        ");
    };
    (
        @type BaseBuilder
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type base builder definition; allowed form is \
            'type BaseBuilder $(<$generics> $($where_clause)?)? = $base_builder_type;\
        ");
    };
    (
        $(#[$attr:meta])* $vis:vis struct $name:ident $($body:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @struct
                []
                [$([$attr])*] [$vis] [$name]
            }
            $($body)*
        }
    };
    (
        @struct
        [[$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty
        {
            $($($field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::dep_type_with_builder_impl! {
            @concat_generics
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
            [$($([$field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @struct
        [[$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty
        {
            $($($field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
        $($token:tt)+
    ) => {
        $crate::std_compile_error!("unexpected extra tokens after dep type definition body");
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty
        {
            $($($field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }

        type BaseBuilder $($token:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @type BaseBuilder after
                [$([$attr])*] [$vis] [$name] [$obj] [$Id]
                [$($g)*] [$($r)*] [$($w)*]
                [$($([$field $delim $($field_ty $(= $field_val)?)?])+)?]
            }
            $($token)*
        }
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty
        {
            $($($field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::std_compile_error!("\
            missing dep type base builder definition; add the definition in the following form \
            before or after dep type definition: \
            'type BaseBuilder $(<$generics> $($where_clause)?)? = $base_builder_type;\
        ");
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty
        {
            $($($field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }

        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type base builder definition; allowed form is \
            'type BaseBuilder $(<$generics> $(where $where_clause)?)? = $base_builder_type;
        ");
    };
    (
        @struct
        [$([$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*])?]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? become $obj in $Id {\n\
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
    (
        @type BaseBuilder after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;
    ) => {
        $crate::dep_type_with_builder_impl! {
            @concat_generics
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
            [$($([$field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @type BaseBuilder after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;

        $($token:tt)*
    ) => {
        $crate::std_compile_error!("unexpected extra tokens after dep type base builder definition");
    };
    (
        @type BaseBuilder after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type base builder definition; allowed form is \
            'type BaseBuilder $(<$generics> $(where $where_clause)?)? = $base_builder_type;
        ");
    };
    (
        @concat_generics
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [[$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
        [$([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
    ) => {
        $crate::generics_concat! {
            $crate::dep_type_with_builder_impl {
                @concat_generics_done
                [$BaseBuilder]
                [$([$attr])*] [$vis] [$name] [$obj] [$Id]
                [$($g)*] [$($r)*] [$($w)*]
                [$([$field $delim $($field_ty $(= $field_val)?)?])*]
            }
            [$($g)*] [$($r)*] [$($w)*],
            [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
        }
    };
    (
        @concat_generics_done
        [$BaseBuilder:ty]
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [state] [this] [bindings] [handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] [] [] []
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*] []]
            [$([$field $delim $($field_ty $(= $field_val)?)?])*]
        }
    };
    (
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? become $obj in $Id {\n\
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
}

#[macro_export]
macro_rules! dep_type {
    (
        $($token:tt)*
    ) => {
        $crate::dep_type_impl! { $($token)* }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_type_impl {
    (
        $(#[$attr:meta])* $vis:vis struct $name:ident $($body:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_impl {
                @struct
                []
                [$([$attr])*] [$vis] [$name]
            }
            $($body)*
        }
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        in $Id:ty
        {
            $($($field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [obj] [$Id] [state] [this] [bindings] [handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] [] [] []
            []
            [$($([$field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        in $Id:ty
        {
            $($($field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
        $($token:tt)+
    ) => {
        $crate::std_compile_error!("unexpected extra tokens after dep type definition body");
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? in $Id {\n\
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
    (
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? in $Id {\n\
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_type_impl_raw {
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[$field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepPropEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepPropEntry::new(&Self:: [< $field:upper _DEFAULT >] ),
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
            [
                $($core_bindings)*
                $this . $field .binding().map(|x| $bindings.push(
                    <$crate::binding::AnyBinding as $crate::std_convert_From<$crate::binding::Binding<$field_ty>>>::from(x)
                ));
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn $field(mut self, value: $field_ty) -> Self {
                        let id = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::id(&self.base);
                        let state = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::state_mut(&mut self.base);
                        $name:: [< $field:upper >] .set_uncond(state, id.$obj(), value);
                        self
                    }
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[$field:ident yield $field_ty:ty] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepEventEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepEventEntry::new(),
            ]
            [
                $($core_consts)*
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepEvent<Self, $field_ty> = {
                    unsafe { 
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepEvent::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[$field:ident [$field_ty:ty]] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [$state] [$this] [$bindings] [$handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepVecEntry<$field_ty>,
            ]
            [
                $($core_new)*
                $field: $crate::DepVecEntry::new(),
            ]
            [
                $($core_consts)*
            ]
            [
                $($dep_props)*

                $vis const [< $field:upper >] : $crate::DepVec<Self, $field_ty> = {
                    unsafe { 
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $field );
                        $crate::DepVec::new(offset)
                    }
                };
            ]
            [
                $($core_bindings)*
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*
                ]
            )?]
            [$($fields)*]
        }
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[$field:ident $delim:tt $field_ty:ty $(= $field_val:expr)?] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!("\
            invalid dep type field definition\n\
            \n\
        ",
            $crate::std_stringify!($field $delim $field_ty $(= $field_val)?),
        "\
            \n\n\
            allowed forms are \
            '$field_name : $field_type = $field_value', \
            '$field_name [$field_type]', and \
            '$field_name yield $field_type'\
        "));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty] [$state:ident] [$this:ident] [$bindings:ident] [$handlers:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$($core_bindings:tt)*]
        [$($core_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        []
    ) => {
        $crate::paste_paste! {
            #[derive($crate::std_fmt_Debug)]
            struct [< $name Core >] $($g)* $($w)* {
                dep_type_core_base: $crate::BaseDepObjCore<$name $($r)*>,
                $($core_fields)*
            }

            impl $($g)* [< $name Core >] $($r)* $($w)* {
                const fn new() -> Self {
                    Self {
                        dep_type_core_base: $crate::BaseDepObjCore::new(),
                        $($core_new)*
                    }
                }

                $($core_consts)*

                fn dep_type_core_base(&self) -> &$crate::BaseDepObjCore<$name $($r)*> {
                    &self.dep_type_core_base
                }

                fn dep_type_core_base_mut(&mut self) -> &mut $crate::BaseDepObjCore<$name $($r)*> {
                    &mut self.dep_type_core_base
                }

                fn dep_type_core_take_all_handlers(&mut self) -> $crate::std_vec_Vec<$crate::std_boxed_Box<dyn $crate::binding::AnyHandler>> {
                    let mut $handlers = $crate::std_vec_Vec::new();
                    let $this = self;
                    $($core_handlers)*
                    $handlers
                }

                fn dep_type_core_take_added_bindings_and_collect_all(&mut self) -> $crate::std_vec_Vec<$crate::binding::AnyBinding> {
                    let mut $bindings = self.dep_type_core_base.take_bindings();
                    let $this = self;
                    $($core_bindings)*
                    $bindings
                }
            }

            impl $($g)* $crate::std_default_Default for [< $name Core >] $($r)* $($w)* {
                fn default() -> Self { Self::new() }
            }

            $( #[ $attr ] )*
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

                #[doc(hidden)]
                fn core_base_priv(&self) -> &$crate::BaseDepObjCore<$name $($r)*> {
                    &self.core.dep_type_core_base
                }

                #[doc(hidden)]
                fn core_base_priv_mut(&mut self) -> &mut $crate::BaseDepObjCore<$name $($r)*> {
                    &mut self.core.dep_type_core_base
                }

                #[doc(hidden)]
                fn take_all_handlers(&mut self) -> $crate::std_vec_Vec<$crate::std_boxed_Box<dyn $crate::binding::AnyHandler>> {
                    self.core.dep_type_core_take_all_handlers()
                }

                #[doc(hidden)]
                fn take_added_bindings_and_collect_all(&mut self) -> $crate::std_vec_Vec<$crate::binding::AnyBinding> {
                    self.core.dep_type_core_take_added_bindings_and_collect_all()
                }
            }

            $(
                $vis struct [< $name Builder >] $($bc_g)* $($bc_w)* {
                    base: $BaseBuilder,
                }

                impl $($bc_g)* [< $name Builder >] $($bc_r)* $($bc_w)* {
                    fn new_priv(base: $BaseBuilder) -> Self {
                        Self { base }
                    }

                    #[allow(dead_code)]
                    fn base_priv(self) -> $BaseBuilder { self.base }

                    #[allow(dead_code)]
                    fn base_priv_ref(&self) -> &$BaseBuilder { &self.base }

                    #[allow(dead_code)]
                    fn base_priv_mut(&mut self) -> &mut $BaseBuilder { &mut self.base }

                    $($builder_methods)*
                }
            )?
        }
    };
}

#[macro_export]
macro_rules! dep_obj {
    (
        $(
            $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $(optional(trait $opt_tr:tt))? $((trait $tr:tt))? $(optional($opt_ty:ty))? $(($ty:ty))? {
                if mut { $field_mut:expr } else { $field:expr }
            }
        )*
    ) => {
        $(
            $crate::dep_obj_impl! {
                $vis fn $name (self as $this, $arena : $Arena) -> $(optional(trait $opt_tr))? $((trait $tr))? $(optional($opt_ty))? $(($ty))? {
                    if mut { $field_mut } else { $field }
                }
            }
        )*
        fn drop_bindings_priv(self, state: &mut dyn $crate::dyn_context_state_State) {
            $(
                let $this = self;
                let $arena: &mut $Arena = <dyn $crate::dyn_context_state_State as $crate::dyn_context_state_StateExt>::get_mut(state);
                $(
                    let f = $field_mut;
                    let handlers = <dyn $tr as $crate::DepType>::take_all_handlers(f);
                    let bindings = <dyn $tr as $crate::DepType>::take_added_bindings_and_collect_all(f);
                )?
                $(
                    let (handlers, bindings) = if let $crate::std_option_Option::Some(f) = $field_mut {
                        (
                            <dyn $opt_tr as $crate::DepType>::take_all_handlers(f),
                            <dyn $opt_tr as $crate::DepType>::take_added_bindings_and_collect_all(f)
                        )
                    } else {
                        ($crate::std_vec_Vec::new(), $crate::std_vec_Vec::new())
                    };
                )?
                $(
                    let handlers = <$ty as $crate::DepType>::take_all_handlers($field_mut);
                    let bindings = <$ty as $crate::DepType>::take_added_bindings_and_collect_all($field_mut);
                )?
                $(
                    let (handlers, bindings) = if let $crate::std_option_Option::Some(f) = $field_mut {
                        (
                            <$opt_ty as $crate::DepType>::take_all_handlers(f),
                            <$opt_ty as $crate::DepType>::take_added_bindings_and_collect_all(f)
                        )
                    } else {
                        ($crate::std_vec_Vec::new(), $crate::std_vec_Vec::new())
                    };
                )?
                for handler in handlers {
                    handler.clear(state);
                }
                for binding in bindings {
                    binding.drop_binding(state);
                }
            )*
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! dep_obj_impl {
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> optional(trait $ty:tt) {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            fn [< $name _ref >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                $arena: &'arena_lifetime dyn $crate::std_any_Any,
                $this: Self
            ) -> &'arena_lifetime DepObjType {
                let $arena = $arena.downcast_ref::<$Arena>().expect("invalid arena cast");
                ($field)
                    .expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                    .downcast_ref::<DepObjType>().expect("invalid cast")
            }

            fn [< $name _mut >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                $arena: &'arena_lifetime mut dyn $crate::std_any_Any,
                $this: Self
            ) -> &'arena_lifetime mut DepObjType {
                let $arena = $arena.downcast_mut::<$Arena>().expect("invalid arena cast");
                ($field_mut)
                    .expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
                    .downcast_mut::<DepObjType>().expect("invalid cast")
            }

            $vis fn [< $name _descriptor >] <DepObjType: $ty + $crate::DepType<Id=Self>>(
            ) -> $crate::GlobDescriptor<Self, DepObjType> {
                $crate::GlobDescriptor {
                    arena: $crate::std_any_TypeId::of::<$Arena>(),
                    field_ref: Self:: [< $name _ref >] ,
                    field_mut: Self:: [< $name _mut >] ,
                }
            }

            $vis fn $name <DepObjType: $ty + $crate::DepType<Id=Self>>(
                self
            ) -> $crate::Glob<Self, DepObjType> {
                $crate::Glob { id: self, descriptor: Self:: [< $name _descriptor >] }
            }
        }
    };
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> (trait $ty:tt) {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            fn [< $name _ref >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                $arena: &'arena_lifetime dyn $crate::std_any_Any,
                $this: Self
            ) -> &'arena_lifetime DepObjType {
                let $arena = $arena.downcast_ref::<$Arena>().expect("invalid arena cast");
                ($field).downcast_ref::<DepObjType>().expect("invalid cast")
            }

            fn [< $name _mut >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                $arena: &'arena_lifetime mut dyn $crate::std_any_Any,
                $this: Self
            ) -> &'arena_lifetime mut DepObjType {
                let $arena = $arena.downcast_mut::<$Arena>().expect("invalid arena cast");
                ($field_mut).downcast_mut::<DepObjType>().expect("invalid cast")
            }

            $vis fn [< $name _descriptor >] <DepObjType: $ty + $crate::DepType<Id=Self>>(
            ) -> $crate::GlobDescriptor<Self, DepObjType> {
                $crate::GlobDescriptor {
                    arena: $crate::std_any_TypeId::of::<$Arena>(),
                    field_ref: Self:: [< $name _ref >] ,
                    field_mut: Self:: [< $name _mut >] ,
                }
            }

            $vis fn $name <DepObjType: $ty + $crate::DepType<Id=Self>>(
                self
            ) -> $crate::Glob<Self, DepObjType> {
                $crate::Glob { id: self, descriptor: Self:: [< $name _descriptor >] }
            }
        }
    };
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident: $Arena:ty) -> optional($ty:ty) {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            fn [< $name _ref >] <'arena_lifetime>(
                $arena: &'arena_lifetime dyn $crate::std_any_Any,
                $this: Self
            ) -> &'arena_lifetime $ty {
                let $arena = $arena.downcast_ref::<$Arena>().expect("invalid arena cast");
                ($field).expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
            }

            fn [< $name _mut >] <'arena_lifetime>(
                $arena: &'arena_lifetime mut dyn $crate::std_any_Any,
                $this: Self
            ) -> &'arena_lifetime mut $ty {
                let $arena = $arena.downcast_mut::<$Arena>().expect("invalid arena cast");
                ($field_mut).expect($crate::std_concat!("missing ", $crate::std_stringify!($name)))
            }

            $vis fn [< $name _descriptor >] (
            ) -> $crate::GlobDescriptor<Self, $ty> {
                $crate::GlobDescriptor {
                    arena: $crate::std_any_TypeId::of::<$Arena>(),
                    field_ref: Self:: [< $name _ref >] ,
                    field_mut: Self:: [< $name _mut >] ,
                }
            }

            $vis fn $name (
                self
            ) -> $crate::Glob<Self, $ty> {
                $crate::Glob { id: self, descriptor: Self:: [< $name _descriptor >] }
            }
        }
    };
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident: $Arena:ty) -> ($ty:ty) {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            fn [< $name _ref >] <'arena_lifetime>(
                $arena: &'arena_lifetime dyn $crate::std_any_Any,
                $this: Self
            ) -> &'arena_lifetime $ty {
                let $arena = $arena.downcast_ref::<$Arena>().expect("invalid arena cast");
                $field
            }

            fn [< $name _mut >] <'arena_lifetime>(
                $arena: &'arena_lifetime mut dyn $crate::std_any_Any,
                $this: Self
            ) -> &'arena_lifetime mut $ty {
                let $arena = $arena.downcast_mut::<$Arena>().expect("invalid arena cast");
                $field_mut
            }

            $vis fn [< $name _descriptor >] (
            ) -> $crate::GlobDescriptor<Self, $ty> {
                $crate::GlobDescriptor {
                    arena: $crate::std_any_TypeId::of::<$Arena>(),
                    field_ref: Self:: [< $name _ref >] ,
                    field_mut: Self:: [< $name _mut >] ,
                }
            }

            $vis fn $name (
                self
            ) -> $crate::Glob<Self, $ty> {
                $crate::Glob { id: self, descriptor: Self:: [< $name _descriptor >] }
            }
        }
    };
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> $(optional(trait $opt_tr:tt))? $(trait $tr:tt)? $(optional($opt_ty:ty))? $($ty:ty)? {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::std_compile_error!($crate::std_concat!("\
            invalid dep obj return type\n\
            \n\
        ",
            $crate::std_stringify!($(dyn $tr)? $($ty)?),
        "\
            \n\n\
            allowed form are \
            '$ty:ty', \
            'trait $trait:tt', \
            'optional($ty:ty)', and \
            'optional(trait $trait:tt)'\
        "));
    };
}
