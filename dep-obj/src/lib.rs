#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn_trait_bound)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_mut_refs)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(shrink_to)]
#![feature(try_reserve)]
#![feature(unchecked_math)]

#![deny(warnings)]

#![cfg_attr(not(feature="std"), no_std)]
#[cfg(feature="std")]
extern crate core;
extern crate alloc;

mod base;
pub use base::*;

pub mod flow;

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
    //!     #[derive(ComponentId!, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
    //!     pub fn new(app: &mut MyApp) -> MyDepTypeId {
    //!         app.my_dep_types.insert(|id| (MyDepTypePrivateData {
    //!             dep_data: MyDepType::new_priv()
    //!         }, MyDepTypeId(id)))
    //!     }
    //!
    //!     dep_obj! {
    //!         pub fn obj(self as this, app: MyApp) -> MyDepType {
    //!             if mut {
    //!                 &mut app.my_dep_types[this.0].dep_data
    //!             } else {
    //!                 &app.my_dep_types[this.0].dep_data
    //!             }
    //!         }
    //!     }
    //! }

    use crate::{dep_obj, dep_type};
    use components_arena::{Arena, Component, ComponentId, Id};
    use dyn_context::State;

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

    ComponentId!(() pub struct MyDepTypeId(Id<MyDepTypePrivateData>););

    #[derive(Debug)]
    pub struct MyApp {
        my_dep_types: Arena<MyDepTypePrivateData>,
    }

    State!(() pub struct MyApp { .. });

    impl MyDepTypeId {
        pub fn new(app: &mut MyApp) -> MyDepTypeId {
            app.my_dep_types.insert(|id| (MyDepTypePrivateData {
                dep_data: MyDepType::new_priv()
            }, MyDepTypeId(id)))
        }

        dep_obj! {
            pub fn obj(self as this, app: MyApp) -> MyDepType {
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
pub use core::compile_error as std_compile_error;
#[doc(hidden)]
pub use core::concat as std_concat;
#[doc(hidden)]
pub use core::default::Default as std_default_Default;
#[doc(hidden)]
pub use core::fmt::Debug as std_fmt_Debug;
#[doc(hidden)]
pub use core::option::Option as std_option_Option;
#[doc(hidden)]
pub use core::stringify as std_stringify;
#[doc(hidden)]
pub use dyn_context::State as dyn_context_State;
#[doc(hidden)]
pub use generics::concat as generics_concat;
#[doc(hidden)]
pub use generics::parse as generics_parse;
#[doc(hidden)]
pub use memoffset::offset_of as memoffset_offset_of;
#[doc(hidden)]
pub use paste::paste as paste_paste;

use crate::flow::{Flow, FlowSource, Just, Through, Snd, RemovedInserted};
use alloc::boxed::Box;
use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use components_arena::{ComponentId, RawId};
use core::fmt::Debug;
use core::mem::replace;
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::{State, StateExt};
use educe::Educe;
use phantom_type::PhantomType;

#[derive(Educe)]
#[educe(Debug)]
pub struct DepPropEntry<PropType: Convenient> {
    default: &'static PropType,
    style: Option<PropType>,
    local: Option<PropType>,
    #[educe(Debug(ignore))]
    on_changed: Vec<(RawId, fn(state: &mut dyn State, handler_id: RawId, old_new: (PropType, PropType)))>,
}

impl<PropType: Convenient> DepPropEntry<PropType> {
    pub const fn new(default: &'static PropType) -> Self {
        DepPropEntry {
            default,
            style: None,
            local: None,
            on_changed: Vec::new()
        }
    }
}


#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecEntry<ItemType: Convenient> {
    items: Vec<ItemType>,
    #[educe(Debug(ignore))]
    on_changed: Vec<(RawId, fn(state: &mut dyn State, handler_id: RawId, change_vec: (VecChange<ItemType>, Vec<ItemType>)))>,
}

impl<ItemType: Convenient> DepVecEntry<ItemType> {
    pub const fn new() -> Self {
        DepVecEntry {
            items: Vec::new(),
            on_changed: Vec::new()
        }
    }
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
/// use components_arena::{Arena, Component, ComponentClassToken, ComponentId, Id};
/// use dep_obj::{dep_obj, dep_type};
/// use dep_obj::flow::{Flows, FlowsToken, Just};
/// use dyn_context::{State, StateExt};
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
///     #[derive(ComponentId!, Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
///     pub struct MyDepTypeId(Id<MyDepTypePrivateData>);
/// }
///
/// pub struct MyApp {
///     flows: Flows,
///     my_dep_types: Arena<MyDepTypePrivateData>,
///     res: i32,
/// }
///
/// impl State for MyApp {
///     fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
///         if ty == TypeId::of::<Flows>() {
///             Some(&self.flows)
///         } else if ty == TypeId::of::<MyApp>() {
///             Some(self)
///         } else {
///             None
///         }
///     }
///
///     fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
///         if ty == TypeId::of::<Flows>() {
///             Some(&mut self.flows)
///         } else if ty == TypeId::of::<MyApp>() {
///             Some(self)
///         } else {
///             None
///         }
///     }
/// }
///
/// impl MyDepTypeId {
///     pub fn new(app: &mut MyApp) -> MyDepTypeId {
///         app.my_dep_types.insert(|id| (MyDepTypePrivateData {
///             dep_data: MyDepType::new_priv()
///         }, MyDepTypeId(id)))
///     }
///
///     dep_obj! {
///         pub fn obj(self as this, app: MyApp) -> MyDepType {
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
///     let mut my_dep_types_token = ComponentClassToken::new().unwrap();
///     let mut flows_token = FlowsToken::new().unwrap();
///     let mut app = MyApp {
///         flows: Flows::new(&mut flows_token),
///         my_dep_types: Arena::new(&mut my_dep_types_token),
///         res: 0,
///     };
///     let id = MyDepTypeId::new(&mut app);
///     id.obj(&mut app).prop(MyDepType::PROP_2).values().handle(&mut app, (), |state, _, Just(value)| {
///         let app: &mut MyApp = state.get_mut();
///         app.res = value;
///     });
///     assert_eq!(app.res, 10);
///     id.obj(&mut app).prop(MyDepType::PROP_2).set_distinct(5);
///     assert_eq!(app.res, 5);
/// }
/// ```
pub trait DepType: Sized {
    type Id: ComponentId;

    #[doc(hidden)]
    fn style__(&mut self) -> &mut Option<Style<Self>>;
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
        let on_changed = if entry_mut.local.is_some() {
            None
        } else {
            Some(entry_mut.on_changed.clone())
        };
        let value = if unapply { None } else { Some(self.value.clone()) };
        let old = replace(&mut entry_mut.style, value.clone());
        on_changed.map(|on_changed| {
            let old = old.unwrap_or_else(|| entry_mut.default.clone());
            let value = value.unwrap_or_else(|| entry_mut.default.clone());
            Box::new(move |state: &'_ mut dyn State| {
                for (handler_id, handler) in on_changed {
                    handler(state, handler_id, (old.clone(), value.clone()));
                }
            }) as _
        })
    }
}

/// A dictionary mapping a subset of target type properties to the values.
/// Every dependency object can have an applied style at every moment.
/// To switch an applied style, use the [`DepObj::apply_style`] function.
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

pub trait DepObjBuilderCore<OwnerId: ComponentId> {
    fn state(&self) -> &dyn State;
    fn state_mut(&mut self) -> &mut dyn State;
    fn id(&self) -> OwnerId;
}

pub struct DepObjProp<'a, 'b, Owner: DepType, Arena, PropType: Convenient> {
    obj: &'b mut DepObj<'a, Owner, Arena>,
    prop: DepProp<Owner, PropType>,
}

impl<'a, 'b, Owner: DepType, Arena: 'static, PropType: Convenient> DepObjProp<'a, 'b, Owner, Arena, PropType> {
    pub fn values(&mut self) -> Flow<Just<PropType>> {
        Flow::new_through(<Through<Snd<PropType>>>::new(), self)
    }

    pub fn changes(&mut self) -> Flow<Just<(PropType, PropType)>> {
        Flow::new(self)
    }

    pub fn set_uncond(&mut self, value: PropType) {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.prop.entry_mut(obj);
        let old = entry_mut.local.replace(value.clone()).unwrap_or_else(||
            entry_mut.style.as_ref().unwrap_or_else(|| entry_mut.default).clone()
        );
        for (hanlder_id, handler) in entry_mut.on_changed.clone() {
            handler(self.obj.state, hanlder_id, (old.clone(), value.clone()));
        }
    }

    pub fn unset_uncond(&mut self) {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.prop.entry_mut(obj);
        if let Some(old) = entry_mut.local.take() {
            let default = entry_mut.style.as_ref().unwrap_or_else(|| entry_mut.default).clone();
            for (handler_id, handler) in entry_mut.on_changed.clone() {
                handler(self.obj.state, handler_id, (old.clone(), default.clone()));
            }
        }
    }

    pub fn set_distinct(&mut self, value: PropType) where PropType: PartialEq {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.prop.entry_mut(obj);
        let old = entry_mut.local.as_ref()
            .or_else(|| entry_mut.style.as_ref())
            .unwrap_or_else(|| entry_mut.default)
            ;
        if &value == old { return; }
        let old = entry_mut.local.replace(value.clone()).unwrap_or_else(||
            entry_mut.style.as_ref().unwrap_or_else(|| entry_mut.default).clone()
        );
        for (handler_id, handler) in entry_mut.on_changed.clone() {
            handler(self.obj.state, handler_id, (old.clone(), value.clone()));
        }
    }

    pub fn unset_distinct(&mut self) where PropType: PartialEq {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.prop.entry_mut(obj);
        if let Some(old) = entry_mut.local.take() {
            let new = entry_mut.style.as_ref().unwrap_or_else(|| entry_mut.default).clone();
            if new == old { return; }
            for (handler_id, handler) in entry_mut.on_changed.clone() {
                handler(self.obj.state, handler_id, (old.clone(), new.clone()));
            }
        }
    }
}

impl<'a, 'b, Owner: DepType, Arena: 'static, PropType: Convenient> FlowSource for DepObjProp<'a, 'b, Owner, Arena, PropType> {
    type Value = (PropType, PropType);

    fn handle<Id:ComponentId, R>(
        &mut self,
        handler: impl FnOnce(
            Self::Value,
            &mut dyn State
        ) -> (Id, fn(state: &mut dyn State, handler_id: RawId, value: Self::Value), R),
    ) -> R {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry = self.prop.entry(obj);
        let old = entry.default.clone();
        let new = entry.local.as_ref().or_else(|| entry.style.as_ref()).unwrap_or_else(|| entry.default).clone();
        let (handler_id, handler, res) = handler((old, new), self.obj.state);
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.prop.entry_mut(obj);
        entry_mut.on_changed.push((handler_id.into_raw(), handler));
        res
    }
}

pub struct DepObjVec<'a, 'b, Owner: DepType, Arena, ItemType: Convenient> {
    obj: &'b mut DepObj<'a, Owner, Arena>,
    vec: DepVec<Owner, ItemType>,
}

impl<'a, 'b, Owner: DepType, Arena: 'static, ItemType: Convenient> DepObjVec<'a, 'b, Owner, Arena, ItemType> {
    pub fn changes(&mut self) -> Flow<Just<(VecChange<ItemType>, Vec<ItemType>)>> {
        Flow::new(self)
    }

    pub fn removed_inserted_items(&mut self) -> Flow<Just<(Vec<ItemType>, Vec<ItemType>)>> {
        Flow::new_through(<Through<RemovedInserted<ItemType>>>::new(), self)
    }

    pub fn clear(&mut self) {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.vec.entry_mut(obj);
        let old_items = replace(&mut entry_mut.items, Vec::new());
        let change_vec = (VecChange::Reset(old_items), Vec::new());
        for (handler_id, handler) in entry_mut.on_changed.clone() {
            handler(self.obj.state, handler_id, change_vec.clone());
        }
    }

    pub fn push(&mut self, item: ItemType) {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.vec.entry_mut(obj);
        entry_mut.items.push(item);
        let change = VecChange::Inserted(
            unsafe { entry_mut.items.len().unchecked_sub(1) } .. entry_mut.items.len()
        );
        let vec = entry_mut.items.clone();
        for (handler_id, handler) in entry_mut.on_changed.clone() {
            handler(self.obj.state, handler_id, (change.clone(), vec.clone()));
        }
    }

    pub fn insert(&mut self, index: usize, item: ItemType) {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.vec.entry_mut(obj);
        entry_mut.items.insert(index, item);
        let change = VecChange::Inserted(index .. index + 1);
        let vec = entry_mut.items.clone();
        for (handler_id, handler) in entry_mut.on_changed.clone() {
            handler(self.obj.state, handler_id, (change.clone(), vec.clone()));
        }
    }

    pub fn append(&mut self, other: &mut Vec<ItemType>) {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.vec.entry_mut(obj);
        let appended = other.len();
        entry_mut.items.append(other);
        let change = VecChange::Inserted(
            unsafe { entry_mut.items.len().unchecked_sub(appended) } .. entry_mut.items.len()
        );
        let vec = entry_mut.items.clone();
        for (handler_id, handler) in entry_mut.on_changed.clone() {
            handler(self.obj.state, handler_id, (change.clone(), vec.clone()));
        }
    }
}

impl<'a, 'b, Owner: DepType, Arena: 'static, ItemType: Convenient> FlowSource for DepObjVec<'a, 'b, Owner, Arena, ItemType> {
    type Value = (VecChange<ItemType>, Vec<ItemType>);

    fn handle<Id:ComponentId, R>(
        &mut self,
        handler: impl FnOnce(
            Self::Value,
            &mut dyn State
        ) -> (Id, fn(state: &mut dyn State, handler_id: RawId, value: Self::Value), R),
    ) -> R {
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry = self.vec.entry(obj);
        let items = entry.items.clone();
        let (handler_id, handler, res) = handler((VecChange::Reset(Vec::new()), items), self.obj.state);
        let arena: &mut Arena = self.obj.state.get_mut();
        let obj = (self.obj.get_obj_mut)(arena, self.obj.id);
        let entry_mut = self.vec.entry_mut(obj);
        entry_mut.on_changed.push((handler_id.into_raw(), handler));
        res
    }
}

pub struct DepObj<'a, Owner: DepType, Arena> {
    id: Owner::Id,
    state: &'a mut dyn State,
    get_obj_mut: for<'b> fn(arena: &'b mut Arena, id: Owner::Id) -> &'b mut Owner,
}

impl<'a, Owner: DepType, Arena: 'static> DepObj<'a, Owner, Arena> {
    pub fn new(
        id: Owner::Id,
        state: &'a mut dyn State,
        get_obj_mut: for<'b> fn(arena: &'b mut Arena, id: Owner::Id) -> &'b mut Owner,
    ) -> Self {
        DepObj { id, state, get_obj_mut }
    }

    pub fn prop<'b, PropType: Convenient>(&'b mut self, prop: DepProp<Owner, PropType>) -> DepObjProp<'a, 'b, Owner, Arena, PropType> {
        DepObjProp { obj: self, prop }
    }

    pub fn vec<'b, ItemType: Convenient>(&'b mut self, vec: DepVec<Owner, ItemType>) -> DepObjVec<'a, 'b, Owner, Arena, ItemType> {
        DepObjVec { obj: self, vec }
    }

    pub fn apply_style(
        &mut self,
        style: Option<Style<Owner>>,
    ) -> Option<Style<Owner>> {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let mut on_changed = Vec::new();
        let old = obj.style__().take();
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
        *obj.style__() = style;
        for on_changed in on_changed {
            on_changed(self.state);
        }
        old
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
        type BuilderCore $($token:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @type BuilderCore
            }
        }
        $($token)*
    };
    (
        @type BuilderCore
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BuilderCore:ty;

        $(#[$attr:meta])* $vis:vis struct $name:ident $($body:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @struct
                [[$BuilderCore] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
                [$([$attr])*] [$vis] [$name]
            }
            $($body)*
        }
    };
    (
        @type BuilderCore
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BuilderCore:ty;

        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition; allowed form is \
            '$(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? \
            become $obj in $Id { ... }'\
        ");
    };
    (
        @type BuilderCore
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type builder core definition; allowed form is \
            'type BuilderCore $(<$generics> $($where_clause)?)? = $builder_core_type;\
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
        [[$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
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
            [[$BuilderCore] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
            [$($([$field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @struct
        [[$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
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

        type BuilderCore $($token:tt)*
    ) => {
        $crate::generics_parse! {
            $crate::dep_type_with_builder_impl {
                @type BuilderCore after
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
            missing dep type builder core definition; add the definition in the following form \
            before or after dep type definition: \
            'type BuilderCore $(<$generics> $($where_clause)?)? = $builder_core_type;\
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
            invalid dep type builder core definition; allowed form is \
            'type BuilderCore $(<$generics> $(where $where_clause)?)? = $builder_core_type;
        ");
    };
    (
        @struct
        [$([$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*])?]
        [$([$attr:meta])*] [$vis:vis] [$name:ident]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type definition, allowed form is\n\
            \n\
            $(#[$attr])* $vis struct $name $(<$generics> $(where $where_clause)?)? become $obj in $Id {\n\
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type]),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type]),\n\
                ...\n\
            }\n\
            \n\
        ");
    };
    (
        @type BuilderCore after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BuilderCore:ty;
    ) => {
        $crate::dep_type_with_builder_impl! {
            @concat_generics
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [[$BuilderCore] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]]
            [$($([$field $delim $($field_ty $(= $field_val)?)?])+)?]
        }
    };
    (
        @type BuilderCore after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        = $BuilderCore:ty;

        $($token:tt)*
    ) => {
        $crate::std_compile_error!("unexpected extra tokens after dep type builder core definition");
    };
    (
        @type BuilderCore after
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])+)?]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
        $($token:tt)*
    ) => {
        $crate::std_compile_error!("\
            invalid dep type builder core definition; allowed form is \
            'type BuilderCore $(<$generics> $(where $where_clause)?)? = $builder_core_type;
        ");
    };
    (
        @concat_generics
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [[$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]]
        [$([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
    ) => {
        $crate::generics_concat! {
            $crate::dep_type_with_builder_impl {
                @concat_generics_done
                [$BuilderCore]
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
        [$BuilderCore:ty]
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$([$field:ident $delim:tt $($field_ty:ty $(= $field_val:expr)?)?])*]
        [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] []
            [[$BuilderCore] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*] []]
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
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type]),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type]),\n\
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
            [$([$attr])*] [$vis] [$name] [obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [] [] [] []
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
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type]),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type]),\n\
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
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type]),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type]),\n\
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
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$(
            [$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[$field:ident : $field_ty:ty = $field_val:expr] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
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
            [$(
                [$BuilderCore] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*

                    $vis fn $field(mut self, value: $field_ty) -> Self {
                        let id = <$BuilderCore as $crate::DepObjBuilderCore<$Id>>::id(&self.core);
                        let state = <$BuilderCore as $crate::DepObjBuilderCore<$Id>>::state_mut(&mut self.core);
                        id. [< $obj _mut >] (state).set_uncond($name:: [< $field:upper >] , value);
                        self
                    }
                ]
            )?]
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
        [$(
            [$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
        [[$field:ident [$field_ty:ty]] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $field: $crate::DepVecEntry<$Id, $field_ty>,
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
            [$(
                [$BuilderCore] [$($bc_g)*] [$($bc_r)*] [$($bc_w)*]
                [
                    $($builder_methods)*
                ]
            )?]
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
        [$(
            [$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
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
            '$field_name : $field_type = $field_value', and \
            '$field_name [$field_type]'\
        "));
    };
    (
        @unroll_fields
        [$([$attr:meta])*] [$vis:vis] [$name:ident] [$obj:ident] [$Id:ty]
        [$($g:tt)*] [$($r:tt)*] [$($w:tt)*]
        [$($core_fields:tt)*]
        [$($core_new:tt)*]
        [$($core_consts:tt)*]
        [$($dep_props:tt)*]
        [$(
            [$BuilderCore:ty] [$($bc_g:tt)*] [$($bc_r:tt)*] [$($bc_w:tt)*]
            [$($builder_methods:tt)*]
        )?]
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

                #[doc(hidden)]
                fn style__(&mut self) -> &mut $crate::std_option_Option<$crate::Style<$name $($r)*>> {
                    &mut self.core.dep_type_core_style
                }
            }

            $(
                $vis struct [< $name Builder >] $($bc_g)* $($bc_w)* {
                    core: $BuilderCore,
                }

                impl $($bc_g)* [< $name Builder >] $($bc_r)* $($bc_w)* {
                    fn new_priv(core: $BuilderCore) -> Self {
                        Self { core }
                    }

                    #[allow(dead_code)]
                    fn core_priv(self) -> $BuilderCore { self.core }

                    #[allow(dead_code)]
                    fn core_priv_ref(&self) -> &$BuilderCore { &self.core }

                    #[allow(dead_code)]
                    fn core_priv_mut(&mut self) -> &mut $BuilderCore { &mut self.core }

                    $($builder_methods)*
                }
            )?
        }
    };
}

#[macro_export]
macro_rules! dep_obj {
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident : $Arena:ty) -> dyn $ty:tt {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            fn [< $name _get_obj_priv >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                $arena: &'arena_lifetime mut $Arena,
                $this: Self
            ) -> &'arena_lifetime mut DepObjType {
                $field_mut.downcast_mut::<DepObjType>().expect("invalid cast")
            }

            #[allow(dead_code)]
            $vis fn $name <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                self,
                state: &'arena_lifetime mut dyn $crate::dyn_context_State,
            ) -> $crate::DepObj<'arena_lifetime, DepObjType, $Arena> {
                $crate::DepObj::new(self, state, Self:: [< $name _get_obj_priv >] )
            }
        }
    };
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident: $Arena:ty) -> $ty:ty {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            fn [< $name _get_obj_priv >] <'arena_lifetime>(
                $arena: &'arena_lifetime mut $Arena,
                $this: Self
            ) -> &'arena_lifetime mut $ty {
                $field_mut
            }

            #[allow(dead_code)]
            $vis fn $name <'arena_lifetime>(
                self,
                state: &'arena_lifetime mut dyn $crate::dyn_context_State,
            ) -> $crate::DepObj<'arena_lifetime, $ty, $Arena> {
                $crate::DepObj::new(self, state, Self:: [< $name _get_obj_priv >] )
            }
        }
    };
}
