use crate::base::Convenient;
use alloc::boxed::Box;
use alloc::vec::Vec;
use components_arena::{Component, ComponentId, Id, Arena, RawId};
use core::fmt::Debug;
use downcast_rs::{Downcast, impl_downcast};
use dyn_context::{State, StateExt};
use educe::Educe;
use macro_attr_2018::macro_attr;
use phantom_type::PhantomType;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Fst<T: Convenient>(pub T);

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Snd<T: Convenient>(pub T);

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Just<T: Convenient>(pub T);

impl<T: Convenient, X: Convenient> From<Just<(T, X)>> for Fst<T> {
    fn from(value: Just<(T, X)>) -> Fst<T> { Fst(value.0.0) }
}

impl<T: Convenient, X: Convenient> From<Just<(T, X)>> for Snd<X> {
    fn from(value: Just<(T, X)>) -> Snd<X> { Snd(value.0.1) }
}

impl<T: Convenient> From<Fst<T>> for Just<T> {
    fn from(value: Fst<T>) -> Just<T> { Just(value.0) }
}

impl<T: Convenient> From<Snd<T>> for Just<T> {
    fn from(value: Snd<T>) -> Just<T> { Just(value.0) }
}

impl<T: Convenient> From<Just<T>> for Fst<T> {
    fn from(value: Just<T>) -> Fst<T> { Fst(value.0) }
}

impl<T: Convenient> From<Just<T>> for Snd<T> {
    fn from(value: Just<T>) -> Snd<T> { Snd(value.0) }
}

pub struct Through<T: Convenient>(PhantomType<T>);

impl<T: Convenient> Through<T> {
    pub fn new() -> Self { Through(PhantomType::new()) }
}

pub trait FlowSource {
    type Value: Convenient;

    fn handle<Id: ComponentId, R>(
        &mut self,
        handler: impl FnOnce(
            Self::Value,
            &mut dyn State
        ) -> (Id, fn(state: &mut dyn State, handler_id: RawId, value: Self::Value), R),
    ) -> R;
}

trait FlowDataBase: Debug + Downcast { }

impl_downcast!(FlowDataBase);

#[derive(Educe)]
#[educe(Debug)]
struct FlowData<T: Convenient> {
    value: T,
    #[educe(Debug(ignore))]
    handlers: Vec<(RawId, fn(state: &mut dyn State, handler_id: RawId, value: T))>,
}

impl<T: Convenient> FlowDataBase for FlowData<T> { }

macro_attr! {
    #[derive(Debug, Component!)]
    struct FlowBox(Box<dyn FlowDataBase>);
}

macro_attr! {
    #[derive(Educe, ComponentId!)]
    #[educe(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
    pub struct Flow<T: Convenient>(Id<FlowBox>, PhantomType<T>);
}

macro_attr! {
    #[derive(Debug, State!)]
    pub struct Flows(Arena<FlowBox>);
}

fn handle_base<T: Convenient>(state: &mut dyn State, id: RawId, value: T) {
    let flows: &mut Flows = state.get_mut();
    let flow = flows.0[Id::from_raw(id)].0.downcast_mut::<FlowData<T>>().unwrap();
    flow.value = value.clone();
    let handlers = flow.handlers.clone();
    for (handler_id, handler) in handlers {
        handler(state, handler_id, value.clone());
    }
}

fn handle_new<T: Convenient, R: Convenient>(state: &mut dyn State, id: RawId, value: T) where Just<T>: Into<R> {
    handle_base(state, id, Just(value).into());
}

fn handle_new_through<T: Convenient, I: Convenient, R: Convenient>(
    state: &mut dyn State,
    id: RawId,
    value: T
) where Just<T>: Into<I>, I: Into<R> {
    handle_base(state, id, Just(value).into().into());
}

fn handle_merge_fst<T: Convenient, X: Convenient, R: Convenient>(
    state: &mut dyn State,
    id: RawId,
    value: Just<T>
) where Just<(T, X)>: Into<R>, R: Into<Just<(T, X)>> {
    let flows: &Flows = state.get();
    let flow = flows.0[Id::from_raw(id)].0.downcast_ref::<FlowData<R>>().unwrap();
    let snd = flow.value.clone().into().0.1;
    handle_base(state, id, Just((value.0, snd)).into());
}

fn handle_merge_snd<T: Convenient, X: Convenient, R: Convenient>(
    state: &mut dyn State,
    id: RawId,
    value: Just<X>
) where Just<(T, X)>: Into<R>, R: Into<Just<(T, X)>> {
    let flows: &Flows = state.get();
    let flow = flows.0[Id::from_raw(id)].0.downcast_ref::<FlowData<R>>().unwrap();
    let fst = flow.value.clone().into().0.0;
    handle_base(state, id, Just((fst, value.0)).into());
}

fn handle_merge_through_fst<T: Convenient, X: Convenient, I: Convenient, R: Convenient>(
    state: &mut dyn State,
    id: RawId,
    value: Just<T>
) where Just<(T, X)>: Into<I>, I: Into<R>, R: Into<I>, I: Into<Just<(T, X)>> {
    let flows: &Flows = state.get();
    let flow = flows.0[Id::from_raw(id)].0.downcast_ref::<FlowData<R>>().unwrap();
    let old_value: Just<(T, X)> = flow.value.clone().into().into();
    let snd = old_value.0.1;
    handle_base::<R>(state, id, Just((value.0, snd)).into().into());
}

fn handle_merge_through_snd<T: Convenient, X: Convenient, I: Convenient, R: Convenient>(
    state: &mut dyn State,
    id: RawId,
    value: Just<X>
) where Just<(T, X)>: Into<I>, I: Into<R>, R: Into<I>, I: Into<Just<(T, X)>> {
    let flows: &Flows = state.get();
    let flow = flows.0[Id::from_raw(id)].0.downcast_ref::<FlowData<R>>().unwrap();
    let old_value: Just<(T, X)> = flow.value.clone().into().into();
    let fst = old_value.0.0;
    handle_base::<R>(state, id, Just((fst, value.0)).into().into());
}

impl<T: Convenient> Flow<T> {
    pub fn handle<Id: ComponentId, R>(
        self,
        state: &mut dyn State,
        handler: impl FnOnce(T, &mut dyn State) -> (Id, fn(state: &mut dyn State, handler_id: RawId, value: T), R),
    ) -> R {
        let flows: &Flows = state.get();
        let value = flows.0[self.0].0.downcast_ref::<FlowData<T>>().unwrap().value.clone();
        let (handler_id, handler, res) = handler(value, state);
        let flows: &mut Flows = state.get_mut();
        let flow = flows.0[self.0].0.downcast_mut::<FlowData<T>>().unwrap();
        flow.handlers.push((handler_id.into_raw(), handler));
        res
    }

    pub fn new<S: FlowSource>(source: &mut S) -> Flow<T> where Just<S::Value>: Into<T> {
        source.handle(|value, state| {
            let value = Just(value).into();
            let flows: &mut Flows = state.get_mut();
            let id = flows.0.insert(|id| (FlowBox(Box::new(FlowData { value, handlers: Vec::new() })), id));
            (id, handle_new, Flow(id, PhantomType::new()))
        })
    }

    pub fn new_through<I: Convenient, S: FlowSource>(
        _through: Through<I>,
        source: &mut S
    ) -> Flow<T> where Just<S::Value>: Into<I>, I: Into<T> {
        source.handle(|value, state| {
            let value = Just(value).into().into();
            let flows: &mut Flows = state.get_mut();
            let id = flows.0.insert(|id| (FlowBox(Box::new(FlowData { value, handlers: Vec::new() })), id));
            (id, handle_new_through, Flow(id, PhantomType::new()))
        })
    }
}

impl<T: Convenient> Flow<Just<T>> {
    pub fn merge<X: Convenient, R: Convenient>(
        self,
        other: Flow<Just<X>>,
        state: &mut dyn State
    ) -> Flow<R> where Just<(T, X)>: Into<R>, R: Into<Just<(T, X)>> {
        self.handle(state, |fst, state| {
            let id = other.handle(state, |snd, state| {
                let value: R = Just((fst.0, snd.0)).into();
                let flows: &mut Flows = state.get_mut();
                let id = flows.0.insert(|id| (FlowBox(Box::new(FlowData { value, handlers: Vec::new() })), id));
                (id, handle_merge_snd, Flow(id, PhantomType::new()))
            });
            (id, handle_merge_fst, id)
        })
    }

    pub fn merge_through<X: Convenient, I: Convenient, R: Convenient>(
        self,
        _through: Through<I>,
        other: Flow<Just<X>>,
        state: &mut dyn State
    ) -> Flow<R> where Just<(T, X)>: Into<I>, I: Into<R>, R: Into<I>, I: Into<Just<(T, X)>> {
        self.handle(state, |fst, state| {
            let id = other.handle(state, |snd, state| {
                let value: I = Just((fst.0, snd.0)).into();
                let value: R = value.into();
                let flows: &mut Flows = state.get_mut();
                let id = flows.0.insert(|id| (FlowBox(Box::new(FlowData { value, handlers: Vec::new() })), id));
                (id, handle_merge_through_snd::<T, X, I, R>, Flow(id, PhantomType::new()))
            });
            (id, handle_merge_through_fst::<T, X, I, R>, id)
        })
    }
}
