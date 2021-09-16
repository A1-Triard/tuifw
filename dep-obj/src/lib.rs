#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn_trait_bound)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_mut_refs)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(never_type)]
#![feature(option_result_unwrap_unchecked)]
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
    //! impl DepObjId for MyDepTypeId { }
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

    use crate::{DepObjId, dep_obj, dep_type};
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

    impl DepObjId for MyDepTypeId { }

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
use alloc::collections::{TryReserveError, VecDeque};
use alloc::vec::Vec;
use components_arena::{Arena, ArenaItemsIntoValues, Component, ComponentId, Id};
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

pub struct GlobRef<'a, Id: ComponentId, Obj> {
    pub arena: &'a dyn Any,
    pub glob: Glob<Id, Obj>,
}

impl<'a, Id: ComponentId, Obj> Deref for GlobRef<'a, Id, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        ((self.glob.descriptor)().field_ref)(self.arena.deref(), self.glob.id)
    }
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
    pub fn get(self, state: &dyn State) -> GlobRef<Id, Obj> {
        let arena = (self.descriptor)().arena;
        GlobRef {
            arena: state.get_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            glob: self
        }
    }

    pub fn get_mut(self, state: &mut dyn State) -> GlobMut<Id, Obj> {
        let arena = (self.descriptor)().arena;
        GlobMut {
            arena: state.get_mut_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            glob: self
        }
    }
}

macro_attr! {
    #[derive(Educe, Component!(class=ItemHandlerComponent))]
    #[educe(Debug)]
    struct ItemHandler<ItemType: Convenient> {
        handler: Box<dyn Handler<ItemChange<ItemType>>>,
        update: Option<BindingBase<()>>,
    }
}

macro_attr! {
    #[derive(Educe, Component!(class=HandlerComponent))]
    #[educe(Debug, Clone)]
    struct BoxedHandler<T>(Box<dyn Handler<T>>);
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Change<PropType: Convenient> {
    pub old: PropType,
    pub new: PropType,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum ItemChangeAction {
    Insert,
    Remove,
    Update,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct ItemChange<ItemType: Convenient> {
    pub item: ItemType,
    pub action: ItemChangeAction,
}

impl<ItemType: Convenient> ItemChange<ItemType> {
    pub fn is_update(&self) -> bool { self.action == ItemChangeAction::Update }

    pub fn is_insert(&self) -> bool { self.action == ItemChangeAction::Insert }

    pub fn is_remove(&self) -> bool { self.action == ItemChangeAction::Remove }
}

#[derive(Debug)]
struct DepPropHandlers<PropType: Convenient> {
    children_has_handlers: Option<bool>,
    value_handlers: Arena<BoxedHandler<PropType>>,
    change_handlers: Arena<BoxedHandler<Change<PropType>>>,
    change_initial_handler: Option<Box<dyn Handler<Change<PropType>>>>,
    change_final_handler: Option<Box<dyn Handler<Change<PropType>>>>,
}

#[derive(Debug)]
struct DepPropHandlersCopy<PropType: Convenient> {
    notify_children: bool,
    value_handlers: ArenaItemsIntoValues<BoxedHandler<PropType>>,
    change_handlers: ArenaItemsIntoValues<BoxedHandler<Change<PropType>>>,
    change_initial_handler: Option<Box<dyn Handler<Change<PropType>>>>,
    change_final_handler: Option<Box<dyn Handler<Change<PropType>>>>,
}

impl<PropType: Convenient> DepPropHandlers<PropType> {
    const fn new(inherits: bool) -> Self {
        DepPropHandlers {
            children_has_handlers: if inherits { Some(false) } else { None },
            value_handlers: Arena::new(),
            change_handlers: Arena::new(),
            change_initial_handler: None,
            change_final_handler: None,
        }
    }

    fn is_empty(&self) -> bool {
        self.children_has_handlers != Some(true) &&
            self.value_handlers.items().is_empty() &&
            self.change_handlers.items().is_empty() &&
            self.change_initial_handler.is_none() &&
            self.change_final_handler.is_none()
    }

    fn take_all(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.value_handlers).into_items().into_values().map(|x| x.0.into_any()));
        handlers.extend(take(&mut self.change_handlers).into_items().into_values().map(|x| x.0.into_any()));
        self.change_initial_handler.take().map(|x| handlers.push(x.into_any()));
        self.change_final_handler.take().map(|x| handlers.push(x.into_any()));
    }

    fn clone(&self) -> DepPropHandlersCopy<PropType> {
        DepPropHandlersCopy {
            notify_children: self.children_has_handlers == Some(true),
            value_handlers: Clone::clone(self.value_handlers.items()).into_values(),
            change_handlers: Clone::clone(self.change_handlers.items()).into_values(),
            change_initial_handler: self.change_initial_handler.clone(),
            change_final_handler: self.change_final_handler.clone(),
        }
    }
}

impl<PropType: Convenient> DepPropHandlersCopy<PropType> {
    fn execute<Owner: DepType>(
        self,
        state: &mut dyn State,
        change: &Change<PropType>,
        obj: Glob<Owner::Id, Owner>,
        prop: DepProp<Owner, PropType>
    ) {
        for handler in self.value_handlers {
            handler.0.execute(state, change.new.clone());
        }
        if let Some(change_initial_handler) = self.change_initial_handler {
            change_initial_handler.execute(state, change.clone());
        }
        for handler in self.change_handlers {
            handler.0.execute(state, change.clone());
        }
        if let Some(change_final_handler) = self.change_final_handler {
            change_final_handler.execute(state, change.clone());
        }
        if self.notify_children {
            prop.notify_children(state, obj, change);
        }
    }
}

#[derive(Debug)]
pub struct DepPropEntry<PropType: Convenient> {
    default: &'static PropType,
    style: Option<PropType>,
    local: Option<PropType>,
    handlers: DepPropHandlers<PropType>,
    binding: Option<BindingBase<PropType>>,
    queue: Option<VecDeque<Option<PropType>>>,
    enqueue: bool,
}

impl<PropType: Convenient> DepPropEntry<PropType> {
    pub const fn new(default: &'static PropType, inherits: bool) -> Self {
        DepPropEntry {
            default,
            handlers: DepPropHandlers::new(inherits),
            style: None,
            local: None,
            binding: None,
            queue: None,
            enqueue: false,
        }
    }

    fn inherits(&self) -> bool { self.handlers.children_has_handlers.is_some() }

    #[doc(hidden)]
    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        self.handlers.take_all(handlers);
    }

    #[doc(hidden)]
    pub fn binding(&self) -> Option<BindingBase<PropType>> {
        self.binding
    }
}

#[derive(Debug)]
pub struct DepEventEntry<ArgsType: DepEventArgs> {
    bubble: bool,
    handlers: Arena<BoxedHandler<ArgsType>>,
}

impl<ArgsType: DepEventArgs> DepEventEntry<ArgsType> {
    pub const fn new(bubble: bool) -> Self {
        DepEventEntry {
            bubble,
            handlers: Arena::new(),
        }
    }

    #[doc(hidden)]
    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.handlers).into_items().into_values().map(|x| x.0.into_any()));
    }
}


#[derive(Debug)]
struct DepVecHandlers<ItemType: Convenient> {
    changed_handlers: Arena<BoxedHandler<()>>,
    item_handlers: Arena<ItemHandler<ItemType>>,
    item_initial_final_handler: Option<Box<dyn Handler<ItemChange<ItemType>>>>,
}

impl<ItemType: Convenient> DepVecHandlers<ItemType> {
    const fn new() -> Self {
        DepVecHandlers {
            changed_handlers: Arena::new(),
            item_handlers: Arena::new(),
            item_initial_final_handler: None,
        }
    }

    fn take_all(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        handlers.extend(take(&mut self.changed_handlers).into_items().into_values().map(|x| x.0.into_any()));
        handlers.extend(take(&mut self.item_handlers).into_items().into_values().map(|x| x.handler.into_any()));
        self.item_initial_final_handler.take().map(|x| handlers.push(x.into_any()));
    }

    fn clone(&self) -> DepVecHandlersCopy<ItemType> {
        DepVecHandlersCopy {
            changed_handlers: self.changed_handlers.items().clone().into_values(),
            item_handlers: self.item_handlers.items().values().map(|x| x.handler.clone()).collect(),
            item_initial_final_handler: self.item_initial_final_handler.clone(),
        }
    }
}

#[derive(Debug)]
struct DepVecHandlersCopy<ItemType: Convenient> {
    changed_handlers: ArenaItemsIntoValues<BoxedHandler<()>>,
    item_handlers: Vec<Box<dyn Handler<ItemChange<ItemType>>>>,
    item_initial_final_handler: Option<Box<dyn Handler<ItemChange<ItemType>>>>,
}

impl<ItemType: Convenient> DepVecHandlersCopy<ItemType> {
    fn execute(self, state: &mut dyn State, action: ItemChangeAction, items: &[ItemType]) {
        if action == ItemChangeAction::Insert {
            if let Some(item_initial_final_handler) = self.item_initial_final_handler.as_ref() {
                for item in items {
                    item_initial_final_handler.execute(state, ItemChange { action: ItemChangeAction::Insert, item: item.clone() });
                }
            }
        }
        for handler in self.item_handlers {
            for item in items {
                handler.execute(state, ItemChange { action, item: item.clone() });
            }
        }
        if action == ItemChangeAction::Remove {
            if let Some(item_initial_final_handler) = self.item_initial_final_handler.as_ref() {
                for item in items {
                    item_initial_final_handler.execute(state, ItemChange { action: ItemChangeAction::Remove, item: item.clone() });
                }
            }
        }
        for handler in self.changed_handlers {
            handler.0.execute(state, ());
        }
    }
}

#[derive(Debug)]
pub struct DepVecEntry<ItemType: Convenient> {
    items: Vec<ItemType>,
    handlers: DepVecHandlers<ItemType>,
    queue: Option<VecDeque<DepVecModification<ItemType>>>,
    enqueue: bool,
}

impl<ItemType: Convenient> DepVecEntry<ItemType> {
    pub const fn new() -> Self {
        DepVecEntry {
            items: Vec::new(),
            handlers: DepVecHandlers::new(),
            queue: None,
            enqueue: false,
        }
    }

    #[doc(hidden)]
    pub fn take_all_handlers(&mut self, handlers: &mut Vec<Box<dyn AnyHandler>>) {
        self.handlers.take_all(handlers);
    }

    #[doc(hidden)]
    pub fn collect_all_bindings(&mut self, bindings: &mut Vec<AnyBindingBase>) {
        for binding in self.handlers.item_handlers.items().values().filter_map(|x| x.update) {
            bindings.push(binding.into());
        }
    }
}

#[derive(Debug)]
pub struct BaseDepObjCore<Owner: DepType> {
    style: Option<Style<Owner>>,
    added_bindings: Vec<AnyBindingBase>,
}

impl<Owner: DepType> BaseDepObjCore<Owner> {
    pub const fn new() -> Self {
        BaseDepObjCore {
            style: None,
            added_bindings: Vec::new(),
        }
    }

    #[doc(hidden)]
    pub fn take_bindings(&mut self) -> Vec<AnyBindingBase> { take(&mut self.added_bindings) }
}

pub trait DepObjIdBase: ComponentId {
    fn parent(self, state: &dyn State) -> Option<Self>;
    fn next(self, state: &dyn State) -> Self;
    fn last_child(self, state: &dyn State) -> Option<Self>;
}

pub trait DepObjId: ComponentId { }

impl<T: DepObjId> DepObjIdBase for T {
    fn parent(self, _state: &dyn State) -> Option<Self> { None }

    fn next(self, _state: &dyn State) -> Self { self }

    fn last_child(self, _state: &dyn State) -> Option<Self> { None }
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
/// use dep_obj::{DepObjId, dep_obj, dep_type};
/// use dep_obj::binding::{Bindings, Binding, Binding1};
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
/// impl DepObjId for MyDepTypeId { }
///
/// pub struct MyApp {
///     bindings: Bindings,
///     my_dep_types: Arena<MyDepTypePrivateData>,
///     res: Binding<i32>,
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
///     use dep_obj::binding::b_immediate;
///     let mut bindings = Bindings::new();
///     let res = Binding1::new(&mut bindings, (), |(), x| Some(x));
///     let app = &mut MyApp {
///         bindings,
///         my_dep_types: Arena::new(),
///         res: res.into(),
///     };
///     let id = MyDepTypeId::new(app);
///     res.set_source_1(app, &mut MyDepType::PROP_2.value_source(id.obj()));
///     assert_eq!(app.res.get_value(app), Some(10));
///     b_immediate(MyDepType::PROP_2.set(app, id.obj(), 5));
///     assert_eq!(app.res.get_value(app), Some(5));
///     id.drop_my_dep_type(app);
///     res.drop_binding(app);
/// }
/// ```
pub trait DepType: Debug {
    type Id: DepObjIdBase;

    #[doc(hidden)]
    fn core_base_priv(&self) -> &BaseDepObjCore<Self> where Self: Sized;

    #[doc(hidden)]
    fn core_base_priv_mut(&mut self) -> &mut BaseDepObjCore<Self> where Self: Sized;

    #[doc(hidden)]
    fn take_all_handlers(&mut self) -> Vec<Box<dyn AnyHandler>>;

    #[doc(hidden)]
    fn take_added_bindings_and_collect_all(&mut self) -> Vec<AnyBindingBase>;

    #[doc(hidden)]
    fn update_parent_children_has_handlers(state: &mut dyn State, obj: Glob<Self::Id, Self>) where Self: Sized;
}

pub trait DepEventArgs: Convenient {
    fn handled(&self) -> bool;
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct DepEvent<Owner: DepType, ArgsType: DepEventArgs> {
    offset: usize,
    _phantom: PhantomType<(Owner, ArgsType)>
}

impl<Owner: DepType, ArgsType: DepEventArgs> DepEvent<Owner, ArgsType> {
    pub const unsafe fn new(offset: usize) -> Self {
        DepEvent { offset, _phantom: PhantomType::new() }
    }

    pub fn offset(self) -> usize { self.offset }

    fn entry(self, owner: &Owner) -> &DepEventEntry<ArgsType> {
        unsafe {
            let entry = (owner as *const _ as usize).unchecked_add(self.offset);
            let entry = entry as *const DepEventEntry<ArgsType>;
            &*entry
        }
    }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepEventEntry<ArgsType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepEventEntry<ArgsType>;
            &mut *entry
        }
    }

    fn raise_raw(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, args: &ArgsType) -> bool {
        let obj = obj.get(state);
        let entry = self.entry(&obj);
        let bubble = entry.bubble;
        let handlers = entry.handlers.items().clone().into_values();
        for handler in handlers {
            handler.0.execute(state, args.clone());
        }
        bubble
    }

    pub fn raise<X: Convenient>(self, state: &mut dyn State, mut obj: Glob<Owner::Id, Owner>, args: ArgsType) -> BYield<X> {
        let bubble = self.raise_raw(state, obj, &args);
        if !bubble || args.handled() { return b_continue(); }
        while let Some(parent) = obj.parent(state) {
            obj = parent;
            let bubble = self.raise_raw(state, obj, &args);
            debug_assert!(bubble);
            if args.handled() { return b_continue(); }
        }
        b_continue()
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

    fn entry(self, owner: &Owner) -> &DepPropEntry<PropType> {
        unsafe {
            let entry = (owner as *const _ as usize).unchecked_add(self.offset);
            let entry = entry as *const DepPropEntry<PropType>;
            &*entry
        }
    }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepPropEntry<PropType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepPropEntry<PropType>;
            &mut *entry
        }
    }

    fn unstyled_non_local_value<T>(self, state: &dyn State, obj: Glob<Owner::Id, Owner>, f: impl FnOnce(&PropType) -> T) -> T {
        let obj_ref = obj.get(state);
        let entry = self.entry(&obj_ref);
        if entry.inherits() {
            if let Some(parent) = obj.parent(state) {
                self.current_value(state, parent, f)
            } else {
                f(&entry.default)
            }
        } else {
            f(&entry.default)
        }
    }

    fn non_local_value<T>(self, state: &dyn State, obj: Glob<Owner::Id, Owner>, f: impl FnOnce(&PropType) -> T) -> T {
        let obj_ref = obj.get(state);
        let entry = self.entry(&obj_ref);
        if let Some(value) = entry.style.as_ref() {
            f(value)
        } else {
            self.unstyled_non_local_value(state, obj, f)
        }
    }

    fn current_value<T>(self, state: &dyn State, obj: Glob<Owner::Id, Owner>, f: impl FnOnce(&PropType) -> T) -> T {
        let obj_ref = obj.get(state);
        let entry = self.entry(&obj_ref);
        if let Some(value) = entry.local.as_ref() {
            f(value)
        } else {
            self.non_local_value(state, obj, f)
        }
    }

    #[doc(hidden)]
    pub fn update_parent_children_has_handlers(self, state: &mut dyn State, mut obj: Glob<Owner::Id, Owner>) {
        while let Some(parent) = obj.parent(state) {
            obj = parent;
            let children_has_handlers = if let Some(last_child) = obj.id.last_child(state) {
                let mut child = last_child;
                loop {
                    child = child.next(state);
                    let child_obj = Glob { id: child, descriptor: obj.descriptor };
                    let obj = child_obj.get(state);
                    let entry = self.entry(&obj);
                    debug_assert!(entry.inherits());
                    if !entry.handlers.is_empty() { break true; }
                    if child == last_child { break false; }
                }
            } else {
                false
            };
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            if children_has_handlers == entry_mut.handlers.children_has_handlers.unwrap() { return; }
            entry_mut.handlers.children_has_handlers = Some(children_has_handlers);
        }
    }

    fn notify_children(
        self,
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        change: &Change<PropType>,
    ) {
        if let Some(last_child) = obj.id.last_child(state) {
            let mut child = last_child;
            loop {
                child = child.next(state);
                let child_obj = Glob { id: child, descriptor: obj.descriptor };
                let mut obj_mut = child_obj.get_mut(state);
                let entry_mut = self.entry_mut(&mut obj_mut);
                debug_assert!(entry_mut.inherits());
                if entry_mut.local.is_none() && entry_mut.style.is_none() {
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute(state, change, child_obj, self);
                }
                if child == last_child { break; }
            }
        }
    }

    fn un_set_core(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, value: Option<PropType>) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        let old = replace(&mut entry_mut.local, value.clone());
        if old == value { return; }
        let handlers = entry_mut.handlers.clone();
        let change = if old.is_some() && value.is_some() {
            unsafe { Change { old: old.unwrap_unchecked(), new: value.unwrap_unchecked() } }
        } else {
            if let Some(change) = self.non_local_value(state, obj, |non_local| {
                let old_ref = old.as_ref().unwrap_or(non_local);
                let value_ref = value.as_ref().unwrap_or(non_local);
                if old_ref == value_ref {
                    None
                } else {
                    let old = old.unwrap_or_else(|| non_local.clone());
                    let new = value.unwrap_or_else(|| non_local.clone());
                    Some(Change { old, new })
                }
            }) {
                change
            } else {
                return;
            }
        };
        handlers.execute(state, &change, obj, self);
    }

    fn un_set(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, mut value: Option<PropType>) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        if replace(&mut entry_mut.enqueue, true) {
            let queue = entry_mut.queue.get_or_insert_with(VecDeque::new);
            queue.push_back(value);
            return;
        }
        loop {
            self.un_set_core(state, obj, value);
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            if let Some(queue) = entry_mut.queue.as_mut() {
                if let Some(queue_head) = queue.pop_front() { value = queue_head; } else { break; }
            } else {
                break;
            }
        }
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.enqueue = false;
    }

    pub fn set<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, value: PropType) -> BYield<X> {
        self.un_set(state, obj, Some(value));
        b_continue()
    }

    pub fn unset<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>) -> BYield<X> {
        self.un_set(state, obj, None);
        b_continue()
    }

    fn bind_raw(
        self,
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        binding: BindingBase<PropType>
    ) where Owner: 'static {
        self.unbind(state, obj);
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.binding = Some(binding);
        binding.set_target(state, Box::new(DepPropSet { prop: self, obj }));
    }

    pub fn bind(
        self,
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        binding: impl Into<BindingBase<PropType>>
    ) where Owner: 'static {
        self.bind_raw(state, obj, binding.into());
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
        entry_mut.binding = None;
    }

    pub fn value_source(self, obj: Glob<Owner::Id, Owner>) -> DepPropValueSource<Owner, PropType> {
        DepPropValueSource { obj, prop: self }
    }

    pub fn change_source(self, obj: Glob<Owner::Id, Owner>) -> DepPropChangeSource<Owner, PropType> {
        DepPropChangeSource { obj, prop: self }
    }

    pub fn change_initial_source(self, obj: Glob<Owner::Id, Owner>) -> DepPropChangeInitialSource<Owner, PropType> {
        DepPropChangeInitialSource { obj, prop: self }
    }

    pub fn change_final_source(self, obj: Glob<Owner::Id, Owner>) -> DepPropChangeFinalSource<Owner, PropType> {
        DepPropChangeFinalSource { obj, prop: self }
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DepPropSet<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> Target<PropType> for DepPropSet<Owner, PropType> {
    fn execute(&self, state: &mut dyn State, value: PropType) {
        b_immediate(self.prop.set(state, self.obj, value));
    }

    fn clear(&self, state: &mut dyn State) {
        self.prop.clear_binding(state, self.obj);
    }
}

#[derive(Debug)]
enum DepVecModification<ItemType: Convenient> {
    Clear,
    Push(ItemType),
    Insert(usize, ItemType),
    Remove(usize),
    ExtendFrom(Vec<ItemType>),
    Update(Id<ItemHandler<ItemType>>),
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

    fn entry(self, owner: &Owner) -> &DepVecEntry<ItemType> {
        unsafe {
            let entry = (owner as *const _ as usize).unchecked_add(self.offset);
            let entry = entry as *const DepVecEntry<ItemType>;
            &*entry
        }
    }

    fn entry_mut(self, owner: &mut Owner) -> &mut DepVecEntry<ItemType> {
        unsafe {
            let entry = (owner as *mut _ as usize).unchecked_add(self.offset);
            let entry = entry as *mut DepVecEntry<ItemType>;
            &mut *entry
        }
    }

    fn modify(
        self,
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        mut modification: DepVecModification<ItemType>,
    ) {
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        if replace(&mut entry_mut.enqueue, true) {
            let queue = entry_mut.queue.get_or_insert_with(VecDeque::new);
            queue.push_back(modification);
            return;
        }
        loop {
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            match modification {
                DepVecModification::Clear => {
                    let items = take(&mut entry_mut.items);
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute(state, ItemChangeAction::Remove, &items);
                },
                DepVecModification::Push(item) => {
                    entry_mut.items.push(item.clone());
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute(state, ItemChangeAction::Insert, &[item]);
                },
                DepVecModification::Insert(index, item) => {
                    entry_mut.items.insert(index, item.clone());
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute(state, ItemChangeAction::Insert, &[item]);
                },
                DepVecModification::Remove(index) => {
                    let item = entry_mut.items.remove(index);
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute(state, ItemChangeAction::Remove, &[item]);
                },
                DepVecModification::ExtendFrom(vec) => {
                    entry_mut.items.extend_from_slice(&vec);
                    let handlers = entry_mut.handlers.clone();
                    handlers.execute(state, ItemChangeAction::Insert, &vec);
                },
                DepVecModification::Update(handler_id) => {
                    let items = entry_mut.items.clone();
                    let handler = entry_mut.handlers.item_handlers[handler_id].handler.clone();
                    for item in &items {
                        handler.execute(state, ItemChange { action: ItemChangeAction::Update, item: item.clone() });
                    }
                },
            };
            let mut obj_mut = obj.get_mut(state);
            let entry_mut = self.entry_mut(&mut obj_mut);
            if let Some(queue) = entry_mut.queue.as_mut() {
                if let Some(queue_head) = queue.pop_front() { modification = queue_head; } else { break; }
            } else {
                break;
            }
        }
        let mut obj_mut = obj.get_mut(state);
        let entry_mut = self.entry_mut(&mut obj_mut);
        entry_mut.enqueue = false;
    }

    pub fn clear<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>) -> BYield<X> {
        self.modify(state, obj, DepVecModification::Clear);
        b_continue()
    }

    pub fn push<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, item: ItemType) -> BYield<X> {
        self.modify(state, obj, DepVecModification::Push(item));
        b_continue()
    }

    pub fn insert<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, index: usize, item: ItemType) -> BYield<X> {
        self.modify(state, obj, DepVecModification::Insert(index, item));
        b_continue()
    }

    pub fn remove<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, index: usize) -> BYield<X> {
        self.modify(state, obj, DepVecModification::Remove(index));
        b_continue()
    }

    pub fn extend_from<X: Convenient>(self, state: &mut dyn State, obj: Glob<Owner::Id, Owner>, other: Vec<ItemType>) -> BYield<X> {
        self.modify(state, obj, DepVecModification::ExtendFrom(other));
        b_continue()
    }

    pub fn changed_source(self, obj: Glob<Owner::Id, Owner>) -> DepVecChangedSource<Owner, ItemType> {
        DepVecChangedSource { obj, vec: self }
    }

    pub fn item_source(self, obj: Glob<Owner::Id, Owner>) -> DepVecItemSource<Owner, ItemType> {
        DepVecItemSource { obj, vec: self, update: None }
    }

    pub fn item_source_with_update(self, update: impl Into<BindingBase<()>>, obj: Glob<Owner::Id, Owner>) -> DepVecItemSource<Owner, ItemType> {
        DepVecItemSource { obj, vec: self, update: Some(update.into()) }
    }

    pub fn item_initial_final_source(self, obj: Glob<Owner::Id, Owner>) -> DepVecItemInitialFinalSource<Owner, ItemType> {
        DepVecItemInitialFinalSource { obj, vec: self }
    }
}

impl<Owner: DepType> Glob<Owner::Id, Owner> {
    pub fn parent(self, state: &dyn State) -> Option<Self> {
        self.id.parent(state).map(|id| Glob { id, descriptor: self.descriptor })
    }

    fn add_binding_raw(self, state: &mut dyn State, binding: AnyBindingBase) {
        let mut obj_mut = self.get_mut(state);
        obj_mut.core_base_priv_mut().added_bindings.push(binding);
    }

    pub fn add_binding(self, state: &mut dyn State, binding: impl Into<AnyBindingBase>) {
        self.add_binding_raw(state, binding.into());
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
                .filter_map(|setter| setter.un_apply(state, self, true))
                .for_each(|x| on_changed.push(x))
            ;
        }
        if let Some(new) = style.as_ref() {
            new.setters
                .iter()
                .filter_map(|setter| setter.un_apply(state, self, false))
                .for_each(|x| on_changed.push(x))
            ;
        }
        let obj = &mut self.get_mut(state);
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
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn State)>>;
}

clone_trait_object!(<Owner: DepType> AnySetter<Owner>);

impl<Owner: DepType + 'static, PropType: Convenient> AnySetter<Owner> for Setter<Owner, PropType> where Owner::Id: 'static {
    fn prop_offset(&self) -> usize { self.prop.offset }

    fn un_apply(
        &self,
        state: &mut dyn State,
        obj: Glob<Owner::Id, Owner>,
        unapply: bool
    ) -> Option<Box<dyn for<'a> FnOnce(&'a mut dyn State)>> {
        let obj_mut = &mut obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(obj_mut);
        let value = if unapply { None } else { Some(self.value.clone()) };
        let old = replace(&mut entry_mut.style, value.clone());
        if entry_mut.local.is_some() || old == value { return None; }
        let handlers = entry_mut.handlers.clone();
        let change = if old.is_some() && value.is_some() {
            unsafe { Change { old: old.unwrap_unchecked(), new: value.unwrap_unchecked() } }
        } else {
            if let Some(change) = self.prop.unstyled_non_local_value(state, obj, |unstyled_non_local_value| {
                let old_ref = old.as_ref().unwrap_or(unstyled_non_local_value);
                let value_ref = value.as_ref().unwrap_or(unstyled_non_local_value);
                if old_ref == value_ref {
                    None
                } else {
                    let old = old.unwrap_or_else(|| unstyled_non_local_value.clone());
                    let new = value.unwrap_or_else(|| unstyled_non_local_value.clone());
                    Some(Change { old, new })
                }
            }) {
                change
            } else {
                return None;
            }
        };
        let prop = self.prop;
        Some(Box::new(move |state: &'_ mut dyn State| handlers.execute(state, &change, obj, prop)))
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
struct DepEventHandledSource<Owner: DepType, ArgsType: DepEventArgs> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<BoxedHandler<ArgsType>>,
    event: DepEvent<Owner, ArgsType>,
}

impl<Owner: DepType, ArgsType: DepEventArgs> HandlerId for DepEventHandledSource<Owner, ArgsType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.event.entry_mut(&mut obj);
        entry_mut.handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepEventSource<Owner: DepType, ArgsType: DepEventArgs> {
    obj: Glob<Owner::Id, Owner>,
    event: DepEvent<Owner, ArgsType>,
}

impl<Owner: DepType + 'static, ArgsType: DepEventArgs + 'static> Source for DepEventSource<Owner, ArgsType> {
    type Value = ArgsType;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ArgsType>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.event.entry_mut(&mut obj);
        let handler_id = entry.handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        HandledSource {
            handler_id: Box::new(DepEventHandledSource { handler_id, obj: self.obj, event: self.event }),
            init: None // TODO some events with cached value?
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledValueSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<BoxedHandler<PropType>>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledValueSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        entry_mut.handlers.value_handlers.remove(self.handler_id);
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeInitialSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeInitialSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        let handler = entry_mut.handlers.change_initial_handler.take();
        debug_assert!(handler.is_some());
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeFinalSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeFinalSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        let handler = entry_mut.handlers.change_final_handler.take();
        debug_assert!(handler.is_some());
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepPropHandledChangeSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<BoxedHandler<Change<PropType>>>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType, PropType: Convenient> HandlerId for DepPropHandledChangeSource<Owner, PropType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.prop.entry_mut(&mut obj);
        entry_mut.handlers.change_handlers.remove(self.handler_id);
        if entry_mut.inherits() && entry_mut.handlers.is_empty() {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropValueSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropValueSource<Owner, PropType> {
    type Value = PropType;
    type Cache = ValueCache<PropType>;

    fn handle(&self, state: &mut dyn State, handler: Box<dyn Handler<PropType>>) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler_id = entry.handlers.value_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
        let value = self.prop.current_value(state, self.obj, |x| x.clone());
        let prop = self.prop;
        let obj = self.obj;
        let init = Box::new(move |state: &mut dyn State| {
            let obj = obj.get(state);
            let entry = prop.entry(&obj);
            let handler = entry.handlers.value_handlers[handler_id].0.clone();
            handler.execute(state, value);
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledValueSource { handler_id, obj: self.obj, prop: self.prop }),
            init: Some(init)
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeSource<Owner, PropType> {
    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler_id = entry.handlers.change_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
        let change = self.prop.current_value(state, self.obj, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let obj = self.obj;
            Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_handlers[handler_id].0.clone();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeSource { handler_id, obj: self.obj, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeInitialSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeInitialSource<Owner, PropType> {
    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler = entry.handlers.change_initial_handler.replace(handler);
        assert!(handler.is_none(), "duplicate initial handler");
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
        let change = self.prop.current_value(state, self.obj, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let obj = self.obj;
            Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_initial_handler.clone().unwrap();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeInitialSource { obj: self.obj, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropChangeFinalSource<Owner: DepType, PropType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    prop: DepProp<Owner, PropType>,
}

impl<Owner: DepType + 'static, PropType: Convenient> Source for DepPropChangeFinalSource<Owner, PropType> {
    type Value = Change<PropType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<Change<PropType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.prop.entry_mut(&mut obj);
        let default_value = entry.default;
        let update_parent_children_has_handlers = entry.inherits() && entry.handlers.is_empty();
        let handler = entry.handlers.change_final_handler.replace(handler);
        assert!(handler.is_none(), "duplicate final handler");
        if update_parent_children_has_handlers {
            self.prop.update_parent_children_has_handlers(state, self.obj);
        }
        let change = self.prop.current_value(state, self.obj, |value| {
            if value == default_value {
                None
            } else {
                Some(Change { old: default_value.clone(), new: value.clone() })
            }
        });
        let init = change.map(|change| {
            let prop = self.prop;
            let obj = self.obj;
            Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = prop.entry(&obj);
                let handler = entry.handlers.change_final_handler.clone().unwrap();
                handler.execute(state, change);
            }) as _
        });
        HandledSource {
            handler_id: Box::new(DepPropHandledChangeFinalSource { obj: self.obj, prop: self.prop }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecChangedHandledSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<BoxedHandler<()>>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecChangedHandledSource<Owner, ItemType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.vec.entry_mut(&mut obj);
        entry_mut.handlers.changed_handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecItemHandledInitialFinalSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecItemHandledInitialFinalSource<Owner, ItemType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.vec.entry_mut(&mut obj);
        let handler = entry_mut.handlers.item_initial_final_handler.take();
        debug_assert!(handler.is_some());
    }
}

#[derive(Educe)]
#[educe(Debug)]
struct DepVecItemHandledSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    handler_id: Id<ItemHandler<ItemType>>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType, ItemType: Convenient> HandlerId for DepVecItemHandledSource<Owner, ItemType> {
    fn unhandle(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry_mut = self.vec.entry_mut(&mut obj);
        entry_mut.handlers.item_handlers.remove(self.handler_id);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecChangedSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecChangedSource<Owner, ItemType> {
    type Value = ();
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<()>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let changed = !entry.items.is_empty();
        let handler_id = entry.handlers.changed_handlers.insert(|handler_id| (BoxedHandler(handler), handler_id));
        let init = if changed {
            let vec = self.vec;
            let obj = self.obj;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = vec.entry(&obj);
                let handler = entry.handlers.changed_handlers[handler_id].0.clone();
                handler.execute(state, ());
            }) as _)
        } else {
            None
        };
        HandledSource {
            handler_id: Box::new(DepVecChangedHandledSource { handler_id, obj: self.obj, vec: self.vec }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
pub struct DepVecItemSourceUpdate<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    vec: DepVec<Owner, ItemType>,
    handler_id: Id<ItemHandler<ItemType>>,
}

impl<Owner: DepType, ItemType: Convenient> Target<()> for DepVecItemSourceUpdate<Owner, ItemType> {
    fn execute(&self, state: &mut dyn State, (): ()) {
        self.vec.modify(state, self.obj, DepVecModification::Update(self.handler_id));
    }

    fn clear(&self, state: &mut dyn State) {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        entry.handlers.item_handlers[self.handler_id].update = None;
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecItemSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    vec: DepVec<Owner, ItemType>,
    update: Option<BindingBase<()>>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecItemSource<Owner, ItemType> {
    type Value = ItemChange<ItemType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ItemChange<ItemType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let items = entry.items.clone();
        let handler_id = entry.handlers.item_handlers.insert(
            |handler_id| (ItemHandler { handler, update: self.update }, handler_id)
        );
        if let Some(update) = self.update {
            update.set_target(state, Box::new(DepVecItemSourceUpdate { obj: self.obj, vec: self.vec, handler_id }));
        }
        let init = if items.is_empty() {
            None
        } else {
            let vec = self.vec;
            let obj = self.obj;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = vec.entry(&obj);
                let handler = entry.handlers.item_handlers[handler_id].handler.clone();
                for item in items {
                    handler.execute(state, ItemChange { action: ItemChangeAction::Insert, item });
                }
            }) as _)
        };
        HandledSource {
            handler_id: Box::new(DepVecItemHandledSource { handler_id, obj: self.obj, vec: self.vec }),
            init
        }
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecItemInitialFinalSource<Owner: DepType, ItemType: Convenient> {
    obj: Glob<Owner::Id, Owner>,
    vec: DepVec<Owner, ItemType>,
}

impl<Owner: DepType + 'static, ItemType: Convenient> Source for DepVecItemInitialFinalSource<Owner, ItemType> {
    type Value = ItemChange<ItemType>;
    type Cache = NoCache;

    fn handle(
        &self,
        state: &mut dyn State,
        handler: Box<dyn Handler<ItemChange<ItemType>>>,
    ) -> HandledSource {
        let mut obj = self.obj.get_mut(state);
        let entry = self.vec.entry_mut(&mut obj);
        let items = entry.items.clone();
        assert!(entry.handlers.item_initial_final_handler.replace(handler).is_none(), "duplicate initial handler");
        let init = if items.is_empty() {
            None
        } else {
            let vec = self.vec;
            let obj = self.obj;
            Some(Box::new(move |state: &mut dyn State| {
                let obj = obj.get(state);
                let entry = vec.entry(&obj);
                let handler = entry.handlers.item_initial_final_handler.clone().unwrap();
                for item in items {
                    handler.execute(state, ItemChange { action: ItemChangeAction::Insert, item });
                }
            }) as _)
        };
        HandledSource {
            handler_id: Box::new(DepVecItemHandledInitialFinalSource { obj: self.obj, vec: self.vec }),
            init
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
            $($($(#[$inherits:tt])? $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::dep_type_with_builder_impl! {
            @concat_generics
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
            [$($([[$($inherits)?] $field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @struct
        [[$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        become $obj:ident in $Id:ty
        {
            $($($(#[$inherits:tt])? $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
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
            $($($(#[$inherits:tt])? $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }

        type BaseBuilder $($token:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @type BaseBuilder after
                [$([$attr])*] [$vis] [$name] [$obj] [$Id]
                [$($g)*] [$($r)*] [$($w)*]
                [$($([[$($inherits)?] $field $delim $($field_ty $(= $field_val)?)?])+)?]
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
            $($($(#[$inherits:tt])? $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
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
            $($($(#[$inherits:tt])? $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
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
                $(#[$field_1_attr])* $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $(#[$field_2_attr])* $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
    (
        @type BaseBuilder after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([[$($inherits:tt)?] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BaseBuilder:ty;
    ) => {
        $crate::dep_type_with_builder_impl! {
            @concat_generics
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
            [$($([[$($inherits)?] $field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @type BaseBuilder after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([[$($inherits:tt)?] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
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
        [$($([[$($inherits:tt)?] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
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
        [$([[$($inherits:tt)?] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
    ) => {
        $crate::generics_concat! {
            $crate::dep_type_with_builder_impl {
                @concat_generics_done
                [$BaseBuilder]
                [$([$attr])*] [$vis] [$name] [$obj] [$Id]
                [$($g)*] [$($r)*] [$($w)*]
                [$([[$($inherits)?] $field $delim $($field_ty $(= $field_val)?)?])*]
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
        [$([[$($inherits:tt)?] $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id] [state] [this] [bindings] [handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] [] [] [] []
            [[$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*] []]
            [$([[$($inherits)?] $field $delim $($field_ty $(= $field_val)?)?])*]
        }
    };
    (
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? become $obj in $Id {\n\
                $(#[$field_1_attr])* $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $(#[$field_2_attr])* $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
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
            $($($(#[$inherits:tt])? $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
        }
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [obj] [$Id] [state] [this] [bindings] [handlers]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] [] [] [] []
            []
            [$($([[$($inherits)?] $field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @struct
        []
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        in $Id:ty
        {
            $($($(#[$inherits:tt])? $field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?),+ $(,)?)?
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
                $(#[$field_1_attr])* $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $(#[$field_2_attr])* $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
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
                $(#[$field_1_attr])* $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $(#[$field_2_attr])* $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[inherits] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
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
                $field: $crate::DepPropEntry::new(&Self:: [< $field:upper _DEFAULT >] , true),
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
                    <$crate::binding::AnyBindingBase as $crate::std_convert_From<$crate::binding::BindingBase<$field_ty>>>::from(x)
                ));
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
                $name:: [< $field:upper >] .update_parent_children_has_handlers($state, $obj);
            ]
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn $field(mut self, value: $field_ty) -> Self {
                        let id = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::id(&self.base);
                        let state = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::state_mut(&mut self.base);
                        $crate::binding::b_immediate($name:: [< $field:upper >] .set(state, id.$obj(), value));
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
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
                $field: $crate::DepPropEntry::new(&Self:: [< $field:upper _DEFAULT >] , false),
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
                    <$crate::binding::AnyBindingBase as $crate::std_convert_From<$crate::binding::BindingBase<$field_ty>>>::from(x)
                ));
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
            ]
            [$(
                [$BaseBuilder] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn $field(mut self, value: $field_ty) -> Self {
                        let id = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::id(&self.base);
                        let state = <$BaseBuilder as $crate::DepObjBaseBuilder<$Id>>::state_mut(&mut self.base);
                        $crate::binding::b_immediate($name:: [< $field:upper >] .set(state, id.$obj(), value));
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[$inherits:tt] $field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep type property attribute: '#[",
            $crate::std_stringify!($inherits),
            "]; allowed attributes are: '#[inherits]'"
        ));
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[bubble] $field:ident yield $field_ty:ty] $($fields:tt)*]
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
                $field: $crate::DepEventEntry::new(true),
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
            [
                $($update_handlers)*
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[] $field:ident yield $field_ty:ty] $($fields:tt)*]
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
                $field: $crate::DepEventEntry::new(false),
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
            [
                $($update_handlers)*
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[$inherits:tt] $field:ident yield $field_ty:ty] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "invalid dep type event attribute: '#[",
            $crate::std_stringify!($inherits),
            "]; allowed attributes are: '#[bubble]'"
        ));
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[] $field:ident [$field_ty:ty]] $($fields:tt)*]
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
                $this . $field .collect_all_bindings(&mut $bindings);
            ]
            [
                $($core_handlers)*
                $this . $field .take_all_handlers(&mut $handlers);
            ]
            [
                $($update_handlers)*
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[$inherits:tt] $field:ident [$field_ty:ty]] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!(
            "unexpected dep type vector property attribute: '#[",
            $crate::std_stringify!($inherits),
            "]'"
        ));
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
        [$($update_handlers:tt)*]
        [$(
            [$BaseBuilder:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[[$($inherits:tt)?] $field:ident $delim:tt $field_ty:ty $(= $field_val:expr)?] $($fields:tt)*]
    ) => {
        $crate::std_compile_error!($crate::std_concat!("\
            invalid dep type field definition\n\
            \n\
        ",
            $crate::std_stringify!($(#[$inherits])? $field $delim $field_ty $(= $field_val)?),
        "\
            \n\n\
            allowed forms are \
            '$(#[inherits])? $field_name : $field_type = $field_value', \
            '$field_name [$field_type]', and \
            '$(#[bubble])? $field_name yield $field_type'\
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
        [$($update_handlers:tt)*]
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

                fn dep_type_core_take_all_handlers(&mut self) -> $crate::std_vec_Vec<$crate::std_boxed_Box<dyn $crate::binding::AnyHandler>> {
                    let mut $handlers = $crate::std_vec_Vec::new();
                    let $this = self;
                    $($core_handlers)*
                    $handlers
                }

                fn dep_type_core_take_added_bindings_and_collect_all(&mut self) -> $crate::std_vec_Vec<$crate::binding::AnyBindingBase> {
                    let mut $bindings = self.dep_type_core_base.take_bindings();
                    let $this = self;
                    $($core_bindings)*
                    $bindings
                }
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
                fn take_added_bindings_and_collect_all(&mut self) -> $crate::std_vec_Vec<$crate::binding::AnyBindingBase> {
                    self.core.dep_type_core_take_added_bindings_and_collect_all()
                }

                #[doc(hidden)]
                #[allow(unused_variables)]
                fn update_parent_children_has_handlers($state: &mut dyn $crate::dyn_context_state_State, $obj: $crate::Glob < $Id, $name $($r)* >) where Self: Sized {
                    $($update_handlers)*
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
                    let bindings = <dyn $tr as $crate::DepType>::take_added_bindings_and_collect_all($field_mut);
                )?
                $(
                    let bindings = if let $crate::std_option_Option::Some(f) = $field_mut {
                        <dyn $opt_tr as $crate::DepType>::take_added_bindings_and_collect_all(f)
                    } else {
                        $crate::std_vec_Vec::new()
                    };
                )?
                $(
                    let bindings = <$ty as $crate::DepType>::take_added_bindings_and_collect_all($field_mut);
                )?
                $(
                    let bindings = if let $crate::std_option_Option::Some(f) = $field_mut {
                        <$opt_ty as $crate::DepType>::take_added_bindings_and_collect_all(f)
                    } else {
                        $crate::std_vec_Vec::new()
                    };
                )?
                for binding in bindings {
                    binding.drop_binding(state);
                }
            )*
            $(
                let $this = self;
                let $arena: &mut $Arena = <dyn $crate::dyn_context_state_State as $crate::dyn_context_state_StateExt>::get_mut(state);
                $(
                    let handlers = <dyn $tr as $crate::DepType>::take_all_handlers($field_mut);
                )?
                $(
                    let handlers = if let $crate::std_option_Option::Some(f) = $field_mut {
                        <dyn $opt_tr as $crate::DepType>::take_all_handlers(f)
                    } else {
                        $crate::std_vec_Vec::new()
                    };
                )?
                $(
                    let handlers = <$ty as $crate::DepType>::take_all_handlers($field_mut);
                    if !handlers.is_empty() {
                        <$ty as $crate::DepType>::update_parent_children_has_handlers(state, self.$name());
                    }
                )?
                $(
                    let handlers = if let $crate::std_option_Option::Some(f) = $field_mut {
                        <$opt_ty as $crate::DepType>::take_all_handlers(f)
                    } else {
                        $crate::std_vec_Vec::new()
                    };
                )?
                for handler in handlers {
                    handler.clear(state);
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
