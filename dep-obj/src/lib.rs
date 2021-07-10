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

#![cfg_attr(not(test), no_std)]
#[cfg(test)]
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

use crate::flow::{Flow, FlowSource, Just, Through, Snd};
use alloc::boxed::Box;
use alloc::collections::TryReserveError;
use alloc::vec::Vec;
use components_arena::{ComponentId, RawId};
use core::fmt::Debug;
use core::mem::replace;
use core::ops::Range;
//use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::{State, StateExt};
use educe::Educe;
//use macro_attr_2018::macro_attr;
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

#[derive(Debug, Clone)]
pub enum DepVecChange<ItemType: Convenient> {
    Reset(Vec<ItemType>),
    Inserted(Range<usize>),
    Removed(usize, Vec<ItemType>),
    Swapped(Range<usize>, Range<usize>),
}

#[derive(Educe)]
#[educe(Debug)]
pub struct DepVecEntry<ItemType: Convenient> {
    items: Vec<ItemType>,
    #[educe(Debug(ignore))]
    _on_changed: Vec<(RawId, fn(state: &mut dyn State, handler_id: RawId, change: DepVecChange<ItemType>))>,
}

impl<ItemType: Convenient> DepVecEntry<ItemType> {
    pub const fn new() -> Self {
        DepVecEntry {
            items: Vec::new(),
            _on_changed: Vec::new()
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
/// use dep_obj::{Dispatcher, dep_obj, dep_type};
/// use dyn_context::State;
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
///     dispatcher: Dispatcher,
///     my_dep_types: Arena<MyDepTypePrivateData>,
/// }
///
/// impl State for MyApp {
///     fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
///         if ty == TypeId::of::<Dispatcher>() {
///             Some(&self.dispatcher)
///         } else if ty == TypeId::of::<MyApp>() {
///             Some(self)
///         } else {
///             None
///         }
///     }
///
///     fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
///         if ty == TypeId::of::<Dispatcher>() {
///             Some(&mut self.dispatcher)
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
///     let mut app = MyApp {
///         dispatcher: Dispatcher::new(),
///         my_dep_types: Arena::new(&mut my_dep_types_token),
///     };
///     let id = MyDepTypeId::new(&mut app);
///     assert_eq!(id.obj_ref(&app).get(MyDepType::PROP_2), &10);
///     id.obj_mut(&mut app).set_distinct(MyDepType::PROP_2, 5);
///     assert_eq!(id.obj_ref(&app).get(MyDepType::PROP_2), &5);
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

    /*
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
    }*/
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
/// To switch an applied style, use the [`DepObjMut::apply_style`] function.
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

/*
pub struct DepObjRef<'a, Owner: DepType, Arena> {
    arena: &'a Arena,
    id: Owner::Id,
    get_obj: for<'b> fn(arena: &'b Arena, id: Owner::Id) -> &'b Owner,
}

impl<'a, Owner: DepType + 'static, Arena> DepObjRef<'a, Owner, Arena> {
    pub fn new(
        arena: &'a Arena,
        id: Owner::Id,
        get_obj: for<'b> fn(arena: &'b Arena, id: Owner::Id) -> &'b Owner,
    ) -> Self {
        DepObjRef { arena, id, get_obj }
    }

    pub fn get<PropType: Convenient>(&self, prop: DepProp<Owner, PropType>) -> &'a PropType {
        let obj = (self.get_obj)(self.arena, self.id);
        let entry = prop.entry(obj);
        entry.local.as_ref().or_else(|| entry.style.as_ref()).unwrap_or_else(|| entry.default)
    }

    pub fn items<ItemType: Convenient>(&self, vec: DepVec<Owner, ItemType>) -> &'a Vec<ItemType> {
        let obj = (self.get_obj)(self.arena, self.id);
        let entry = vec.entry(obj);
        &entry.items
    }
}

pub struct DepObjRefMut<'a, Owner: DepType, Arena> {
    arena: &'a mut Arena,
    id: Owner::Id,
    get_obj_mut: for<'b> fn(arena: &'b mut Arena, id: Owner::Id) -> &'b mut Owner,
}

impl<'a, Owner: DepType, Arena> DepObjRefMut<'a, Owner, Arena> {
    pub fn new(
        arena: &'a mut Arena,
        id: Owner::Id,
        get_obj_mut: for<'b> fn(arena: &'b mut Arena, id: Owner::Id) -> &'b mut Owner,
    ) -> Self {
        DepObjRefMut { arena, id, get_obj_mut }
    }



    pub fn on<ArgsType>(
        &mut self,
        event: DepEvent<Owner, ArgsType>,
        callback: fn(
            state: &mut dyn State,
            id: Owner::Id,
            args: &mut ArgsType
        ),
    ) {
        let obj = (self.get_obj_mut)(self.arena, self.id);
        let entry_mut = event.entry_mut(obj);
        entry_mut.on_raised.push(callback);
    }

    pub fn on_vec_changed<ItemType: Convenient>(
        &mut self,
        vec: DepVec<Owner, ItemType>,
        callback: fn(
            state: &mut dyn State,
            id: Owner::Id,
            change: &DepVecChange<ItemType>
        ),
    ) {
        let obj = (self.get_obj_mut)(self.arena, self.id);
        let entry_mut = vec.entry_mut(obj);
        entry_mut.on_changed.push(callback);
    }
}
*/

struct DepPropFlowSource<'a, Owner: DepType, Arena, PropType: Convenient> {
    obj: DepObjMut<'a, Owner, Arena>,
    prop: DepProp<Owner, PropType>,
}

impl<'a, Owner: DepType, Arena: 'static, PropType: Convenient> FlowSource for DepPropFlowSource<'a, Owner, Arena, PropType> {
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

pub struct DepObjMut<'a, Owner: DepType, Arena> {
    id: Owner::Id,
    state: &'a mut dyn State,
    get_obj_mut: for<'b> fn(arena: &'b mut Arena, id: Owner::Id) -> &'b mut Owner,
}

impl<'a, Owner: DepType, Arena: 'static> DepObjMut<'a, Owner, Arena> {
    pub fn new(
        id: Owner::Id,
        state: &'a mut dyn State,
        get_obj_mut: for<'b> fn(arena: &'b mut Arena, id: Owner::Id) -> &'b mut Owner,
    ) -> Self {
        DepObjMut { id, state, get_obj_mut }
    }

    pub fn values<PropType: Convenient>(
        self,
        prop: DepProp<Owner, PropType>,
    ) -> Flow<Just<PropType>> {
        let mut source = DepPropFlowSource { obj: self, prop };
        Flow::new_through(<Through<Snd<PropType>>>::new(), &mut source)
    }

    pub fn changes<PropType: Convenient>(
        self,
        prop: DepProp<Owner, PropType>,
    ) -> Flow<Just<(PropType, PropType)>> {
        let mut source = DepPropFlowSource { obj: self, prop };
        Flow::new(&mut source)
    }

    pub fn set_uncond<PropType: Convenient>(
        &mut self,
        prop: DepProp<Owner, PropType>,
        value: PropType
    ) {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let entry_mut = prop.entry_mut(obj);
        let old = entry_mut.local.replace(value.clone()).unwrap_or_else(||
            entry_mut.style.as_ref().unwrap_or_else(|| entry_mut.default).clone()
        );
        for (hanlder_id, handler) in entry_mut.on_changed.clone() {
            handler(self.state, hanlder_id, (old.clone(), value.clone()));
        }
    }

    pub fn unset_uncond<PropType: Convenient>(
        &mut self,
        prop: DepProp<Owner, PropType>,
    ) {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let entry_mut = prop.entry_mut(obj);
        if let Some(old) = entry_mut.local.take() {
            let default = entry_mut.style.as_ref().unwrap_or_else(|| entry_mut.default).clone();
            for (handler_id, handler) in entry_mut.on_changed.clone() {
                handler(self.state, handler_id, (old.clone(), default.clone()));
            }
        }
    }

    pub fn set_distinct<PropType: Convenient + PartialEq>(
        &mut self,
        prop: DepProp<Owner, PropType>,
        value: PropType,
    ) {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let entry_mut = prop.entry_mut(obj);
        let old = entry_mut.local.as_ref()
            .or_else(|| entry_mut.style.as_ref())
            .unwrap_or_else(|| entry_mut.default)
        ;
        if &value == old { return; }
        let old = entry_mut.local.replace(value.clone()).unwrap_or_else(||
            entry_mut.style.as_ref().unwrap_or_else(|| entry_mut.default).clone()
        );
        for (handler_id, handler) in entry_mut.on_changed.clone() {
            handler(self.state, handler_id, (old.clone(), value.clone()));
        }
    }

    pub fn unset_distinct<PropType: Convenient + PartialEq>(
        &mut self,
        prop: DepProp<Owner, PropType>,
    ) {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let entry_mut = prop.entry_mut(obj);
        if let Some(old) = entry_mut.local.take() {
            let new = entry_mut.style.as_ref().unwrap_or_else(|| entry_mut.default).clone();
            if new == old { return; }
            for (handler_id, handler) in entry_mut.on_changed.clone() {
                handler(self.state, handler_id, (old.clone(), new.clone()));
            }
        }
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

    /*
    pub fn clear<ItemType: Convenient>(
        &mut self,
        vec: DepVec<Owner, ItemType>,
    ) -> DepVecChange<ItemType> {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let entry_mut = vec.entry_mut(obj);
        let old_items = replace(&mut entry_mut.items, Vec::new());
        let change = DepVecChange::Reset(old_items);
        for on_changed in entry_mut.on_changed.clone() {
            on_changed(self.state, self.id, &change);
        }
        change
    }

    pub fn push<ItemType: Convenient>(
        &mut self,
        vec: DepVec<Owner, ItemType>,
        item: ItemType,
    ) -> DepVecChange<ItemType> {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let entry_mut = vec.entry_mut(obj);
        entry_mut.items.push(item);
        let change = DepVecChange::Inserted(
            unsafe { entry_mut.items.len().unchecked_sub(1) } .. entry_mut.items.len()
        );
        for on_changed in entry_mut.on_changed.clone() {
            on_changed(self.state, self.id, &change);
        }
        change
    }

    pub fn insert<ItemType: Convenient>(
        &mut self,
        vec: DepVec<Owner, ItemType>,
        index: usize,
        item: ItemType
    ) -> DepVecChange<ItemType> {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let entry_mut = vec.entry_mut(obj);
        entry_mut.items.insert(index, item);
        let change = DepVecChange::Inserted(index .. index + 1);
        for on_changed in entry_mut.on_changed.clone() {
            on_changed(self.state, self.id, &change);
        }
        change
    }

    pub fn append<ItemType: Convenient>(
        &mut self,
        vec: DepVec<Owner, ItemType>,
        other: &mut Vec<ItemType>,
    ) -> DepVecChange<ItemType> {
        let arena: &mut Arena = self.state.get_mut();
        let obj = (self.get_obj_mut)(arena, self.id);
        let entry_mut = vec.entry_mut(obj);
        let appended = other.len();
        entry_mut.items.append(other);
        let change = DepVecChange::Inserted(
            unsafe { entry_mut.items.len().unchecked_sub(appended) } .. entry_mut.items.len()
        );
        for on_changed in entry_mut.on_changed.clone() {
            on_changed(self.state, self.id, &change);
        }
        change
    }
     */
}

/*
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
                $field_1_name $(: $field_1_type = $field_1_value | [$field_1_type] | yield $field_1_type),\n\
                $field_2_name $(: $field_2_type = $field_2_value | [$field_2_type] | yield $field_2_type),\n\
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
        [[$event:ident yield $event_args:ty] $($fields:tt)*]
    ) => {
        $crate::dep_type_impl_raw! {
            @unroll_fields
            [$([$attr])*] [$vis] [$name] [$obj] [$Id]
            [$($g)*] [$($r)*] [$($w)*]
            [
                $($core_fields)*
                $event: $crate::DepEventEntry<$Id, $event_args>,
            ]
            [
                $($core_new)*
                $event: $crate::DepEventEntry::new(),
            ]
            [
                $($core_consts)*
            ]
            [
                $($dep_props)*

                $vis const [< $event:upper >] : $crate::DepEvent<Self, $event_args> = {
                    unsafe { 
                        let offset = $crate::memoffset_offset_of!( [< $name Core >] $($r)*, $event );
                        $crate::DepEvent::new(offset)
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
            '$field_name : $field_type = $field_value', \
            '$field_name [$field_type]', and \
            '$field_name yield $field_type'\
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
            fn [< $name _get_obj_ref_priv >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                $arena: &'arena_lifetime $Arena,
                $this: Self
            ) -> &'arena_lifetime DepObjType {
                $field.downcast_ref::<DepObjType>().expect("invalid cast")
            }

            fn [< $name _get_obj_mut_priv >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                $arena: &'arena_lifetime mut $Arena,
                $this: Self
            ) -> &'arena_lifetime mut DepObjType {
                $field_mut.downcast_mut::<DepObjType>().expect("invalid cast")
            }

            #[allow(dead_code)]
            $vis fn [< $name _ref >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                self,
                $arena: &'arena_lifetime $Arena,
            ) -> $crate::DepObjRef<'arena_lifetime, DepObjType, $Arena> {
                $crate::DepObjRef::new($arena, self, Self:: [< $name _get_obj_ref_priv >] )
            }

            #[allow(dead_code)]
            $vis fn $name <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                self,
                state: &'arena_lifetime mut dyn $crate::dyn_context_State,
            ) -> $crate::DepObjMut<'arena_lifetime, DepObjType, $Arena> {
                $crate::DepObjMut::new(self, state, Self:: [< $name _get_obj_mut_priv >] )
            }

            #[allow(dead_code)]
            $vis fn [< $name _todo >] <'arena_lifetime, DepObjType: $ty + $crate::DepType<Id=Self>>(
                self,
                $arena: &'arena_lifetime mut $Arena,
            ) -> $crate::DepObjRefMut<'arena_lifetime, DepObjType, $Arena> {
                $crate::DepObjRefMut::new($arena, self, Self:: [< $name _get_obj_mut_priv >] )
            }
        }
    };
    (
        $vis:vis fn $name:ident (self as $this:ident, $arena:ident: $Arena:ty) -> $ty:ty {
            if mut { $field_mut:expr } else { $field:expr }
        }
    ) => {
        $crate::paste_paste! {
            fn [< $name _get_obj_ref_priv >] <'arena_lifetime>(
                $arena: &'arena_lifetime $Arena,
                $this: Self
            ) -> &'arena_lifetime $ty {
                $field
            }

            fn [< $name _get_obj_mut_priv >] <'arena_lifetime>(
                $arena: &'arena_lifetime mut $Arena,
                $this: Self
            ) -> &'arena_lifetime mut $ty {
                $field_mut
            }

            #[allow(dead_code)]
            $vis fn [< $name _ref >] <'arena_lifetime>(
                self,
                $arena: &'arena_lifetime $Arena,
            ) -> $crate::DepObjRef<'arena_lifetime, $ty, $Arena> {
                $crate::DepObjRef::new($arena, self, Self:: [< $name _get_obj_ref_priv >] )
            }

            #[allow(dead_code)]
            $vis fn [< $name _mut >] <'arena_lifetime>(
                self,
                state: &'arena_lifetime mut dyn $crate::dyn_context_State,
            ) -> $crate::DepObjMut<'arena_lifetime, $ty, $Arena> {
                $crate::DepObjMut::new(self, state, Self:: [< $name _get_obj_mut_priv >] )
            }

            #[allow(dead_code)]
            $vis fn $name <'arena_lifetime>(
                self,
                $arena: &'arena_lifetime mut $Arena,
            ) -> $crate::DepObjRefMut<'arena_lifetime, $ty, $Arena> {
                $crate::DepObjRefMut::new($arena, self, Self:: [< $name _get_obj_mut_priv >] )
            }
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::boxed::Box;
    use alloc::rc::Rc;
    use components_arena::{Arena, Id, ComponentId, Component, ComponentClassMutex};
    use dyn_context::{State, free_lifetimes, StateRefMut};
    use educe::Educe;
    use macro_attr_2018::macro_attr;
    use core::cell::Cell;

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
                    arena.0[this.0].obj1.as_mut().unwrap().as_mut()
                } else {
                    arena.0[this.0].obj1.as_ref().unwrap().as_ref()
                }
            }
        }
    }

    struct TestIdBuilder<'a> {
        id: TestId,
        state: &'a mut dyn State
    }

    impl<'a> DepObjBuilderCore<TestId> for TestIdBuilder<'a> {
        fn id(&self) -> TestId { self.id }
        fn state(&self) -> &dyn State { self.state }
        fn state_mut(&mut self) -> &mut dyn State { self.state }
    }

    macro_attr! {
        #[derive(State!)]
        struct TestArena(Arena<TestNode>);
    }


    dep_type_with_builder! {
        #[derive(Debug)]
        struct TestObj1 become obj1 in TestId {
            int_val: i32 = 42,
            coll [u64],
        }

        type BuilderCore<'a> = TestIdBuilder<'a>;
    }

    impl TestObj1 {
        pub fn new(arena: &mut TestArena, id: TestId) {
            arena.0[id.0].obj1 = Some(Box::new(TestObj1::new_priv()));
        }
    }

    free_lifetimes! {
        struct TestState {
            changed: 'changed mut u16,
        }
    }
    
    State!(() struct TestState { .. });

    #[test]
    fn create_test_obj_1() {
        let mut dispatcher = Dispatcher::new();
        let mut arena = TestArena(Arena::new(&mut TEST_NODE.lock().unwrap()));
        let id = arena.0.insert(|id| (TestNode { obj1: None }, TestId(id)));
        TestObj1::new(&mut arena, id);
        let mut changed = 0;
        TestStateBuilder { changed: &mut changed }.build_and_then(|state| {
            state.merge_mut_and_then(|state| {
                state.merge_mut_and_then(|state| {
                    let arena = state.get_mut::<TestArena>();
                    let v = id.obj1_ref(arena).get(TestObj1::INT_VAL);
                    assert_eq!(v, &42);
                    id.obj1(arena).on_changed(TestObj1::INT_VAL, |state, _, _| {
                        let test_state = state.get_mut::<TestState>();
                        *test_state.changed_mut() += 1;
                    });
                    let test_state = state.get::<TestState>();
                    assert_eq!(test_state.changed(), &0);
                    id.obj1_mut(state).set_uncond(TestObj1::INT_VAL, 43);
                    let test_state = state.get::<TestState>();
                    assert_eq!(test_state.changed(), &0);
                    Dispatcher::dispatch(state);
                    let test_state = state.get::<TestState>();
                    assert_eq!(test_state.changed(), &1);
                    let arena = state.get::<TestArena>();
                    assert_eq!(id.obj1_ref(arena).get(TestObj1::INT_VAL), &43);
                }, &mut dispatcher);
            }, &mut arena);
        });
    }

    #[test]
    fn test_obj_1_style() {
        let mut dispatcher = Dispatcher::new();
        let mut arena = TestArena(Arena::new(&mut TEST_NODE.lock().unwrap()));
        let id = arena.0.insert(|id| (TestNode { obj1: None }, TestId(id)));
        TestObj1::new(&mut arena, id);
        assert_eq!(id.obj1_ref(&arena).get(TestObj1::INT_VAL), &42);
        let mut style = Style::new();
        style.insert(TestObj1::INT_VAL, 43);
        (&mut arena).merge_mut_and_then(|s| id.obj1_mut(s).apply_style(Some(style.clone())), &mut dispatcher);
        assert_eq!(id.obj1_ref(&arena).get(TestObj1::INT_VAL), &43);
        (&mut arena).merge_mut_and_then(|s| id.obj1_mut(s).set_uncond(TestObj1::INT_VAL, 44), &mut dispatcher);
        assert_eq!(id.obj1_ref(&arena).get(TestObj1::INT_VAL), &44);
        style.insert(TestObj1::INT_VAL, 45);
        (&mut arena).merge_mut_and_then(|s| id.obj1_mut(s).apply_style(Some(style)), &mut dispatcher);
        assert_eq!(id.obj1_ref(&arena).get(TestObj1::INT_VAL), &44);
        (&mut arena).merge_mut_and_then(|s| id.obj1_mut(s).unset_uncond(TestObj1::INT_VAL), &mut dispatcher);
        assert_eq!(id.obj1_ref(&arena).get(TestObj1::INT_VAL), &45);
    }

    #[test]
    fn test_obj_1_builder() {
        let mut dispatcher = Dispatcher::new();
        let mut arena = TestArena(Arena::new(&mut TEST_NODE.lock().unwrap()));
        let id = arena.0.insert(|id| (TestNode { obj1: None }, TestId(id)));
        TestObj1::new(&mut arena, id);
        (&mut arena).merge_mut_and_then(|state| {
            let builder = TestObj1Builder::new_priv(TestIdBuilder { id, state });
            builder.int_val(1);
        }, &mut dispatcher);
        assert_eq!(id.obj1_ref(&arena).get(TestObj1::INT_VAL), &1);
    }

    #[test]
    fn test_obj_1_coll() {
        let mut dispatcher = Dispatcher::new();
        let mut arena = TestArena(Arena::new(&mut TEST_NODE.lock().unwrap()));
        let id = arena.0.insert(|id| (TestNode { obj1: None }, TestId(id)));
        TestObj1::new(&mut arena, id);
        assert!(id.obj1_ref(&arena).items(TestObj1::COLL).is_empty());
        let change_match = Rc::new(Cell::new(false));
        let change_match_ref = change_match.clone();
        (&mut arena).merge_mut_and_then(|state| {
            id.obj1_mut(state).push_and_then(move |_, _, change| {
                change_match_ref.set(match &change {
                    DepVecChange::Inserted(x) => *x == (0 .. 1),
                    _ => false,
                });
            }, TestObj1::COLL, 7);
            Dispatcher::dispatch(state);
        }, &mut dispatcher);
        assert!(change_match.get());
        assert_eq!(id.obj1_ref(&arena).items(TestObj1::COLL)[0], 7);
    }
}
*/