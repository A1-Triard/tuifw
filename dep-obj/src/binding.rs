use crate::base::*;
use alloc::boxed::Box;
use components_arena::{Component, ComponentId, Id, Arena, NewtypeComponentId, RawId};
use core::any::{Any, TypeId};
use core::fmt::Debug;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::state::{SelfState, State, StateExt};
use educe::Educe;
use macro_attr_2018::macro_attr;
use panicking::panicking;
use phantom_type::PhantomType;

#[must_use]
pub struct BYield<T: Convenient>(Option<T>);

pub fn b_yield<T: Convenient>(value: T) -> BYield<T> {
    BYield(Some(value))
}

pub fn b_continue<T: Convenient>() -> BYield<T> {
    BYield(None)
}

pub fn b_immediate(x: BYield<!>) {
    let _ = x;
}

pub trait Target<T: Convenient>: Debug + DynClone {
    fn execute(&self, state: &mut dyn State, value: T);
    fn clear(&self, state: &mut dyn State);
}

clone_trait_object!(<T: Convenient> Target<T>);

#[derive(Educe)]
#[educe(Debug, Clone)]
struct DispatchTarget<Context: Debug + Clone, T: Convenient> {
    context: Context,
    #[educe(Debug(ignore))]
    execute: fn(state: &mut dyn State, context: Context, value: T) -> BYield<!>,
}

impl<Context: Debug + Clone, T: Convenient> Target<T> for DispatchTarget<Context, T> {
    fn execute(&self, state: &mut dyn State, value: T) {
        let _ = (self.execute)(state, self.context.clone(), value);
    }

    fn clear(&self, _state: &mut dyn State) { }
}

#[derive(Educe)]
#[educe(Debug, Clone)]
struct FnTarget<Context: Debug + Clone, T: Convenient> {
    context: Context,
    #[educe(Debug(ignore))]
    execute: fn(state: &mut dyn State, context: Context, value: T)
}

impl<Context: Debug + Clone, T: Convenient> Target<T> for FnTarget<Context, T> {
    fn execute(&self, state: &mut dyn State, value: T) {
        (self.execute)(state, self.context.clone(), value);
    }

    fn clear(&self, _state: &mut dyn State) { }
}

pub trait AnyHandler: Debug {
    fn clear(&self, state: &mut dyn State);
}

pub trait Handler<T>: Debug + DynClone + Send + Sync {
    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler>;
    fn execute(&self, state: &mut dyn State, args: T);
}

clone_trait_object!(<T> Handler<T>);

pub trait SourceCache<T: Convenient>: Default + Debug + Send + Sync {
    type Value: Convenient;
    fn update(&mut self, value: T);
    fn get(&self, current: Option<T>) -> Option<Self::Value>;
}

#[derive(Educe)]
#[educe(Debug)]
pub struct ValueCache<T: Convenient>(Option<T>);

impl<T: Convenient> Default for ValueCache<T> {
    fn default() -> Self { ValueCache(None) }
}

impl<T: Convenient> SourceCache<T> for ValueCache<T> {
    type Value = T;

    fn update(&mut self, value: T) { self.0 = Some(value); }

    fn get(&self, current: Option<T>) -> Option<T> {
        current.map_or_else(|| self.0.clone(), |current| Some(current))
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct NoCache(());

impl Default for NoCache {
    fn default() -> Self { NoCache(()) }
}

impl<T: Convenient> SourceCache<T> for NoCache {
    type Value = Option<T>;

    fn update(&mut self, _: T) { }

    fn get(&self, current: Option<T>) -> Option<Option<T>> { Some(current) }
}

pub trait Source: Debug {
    type Value: Convenient;
    type Cache: SourceCache<Self::Value>;
    fn handle(&self, state: &mut dyn State, handler: Box<dyn Handler<Self::Value>>) -> HandledSource;
}

pub trait HandlerId: Debug {
    fn unhandle(&self, state: &mut dyn State);
}

#[derive(Educe)]
#[educe(Debug)]
pub struct HandledSource {
    pub handler_id: Box<dyn HandlerId>,
    #[educe(Debug(ignore))]
    pub init: Option<Box<dyn FnOnce(&mut dyn State)>>,
}

trait AnyBindingNode: Debug + Downcast {
    fn unhandle_sources_and_clear_target(&mut self, state: &mut dyn State);
}

impl_downcast!(AnyBindingNode);

macro_attr! {
    #[doc(hidden)]
    #[derive(Debug, Component!)]
    pub struct BoxedBindingNode(Box<dyn AnyBindingNode>);
}

#[derive(Debug)]
pub struct Bindings(Arena<BoxedBindingNode>);

impl SelfState for Bindings { }

impl Bindings {
    pub const fn new() -> Self { Bindings(Arena::new()) }
}

impl Drop for Bindings {
    fn drop(&mut self) {
        if !panicking() {
            debug_assert!(self.0.items().is_empty(), "there are non-dropped bindings (count: {})", self.0.items().len());
        }
    }
}

trait AnyBindingNodeSources: Downcast {
    type Value: Convenient;
    fn unhandle(&mut self, state: &mut dyn State);
    fn get_value(&self) -> Option<Self::Value>;
    fn is_empty(&self) -> bool;
}

impl_downcast!(AnyBindingNodeSources assoc Value where Value: Convenient);

#[derive(Educe)]
#[educe(Debug)]
struct BindingNode<T: Convenient> {
    #[educe(Debug(ignore))]
    sources: Box<dyn AnyBindingNodeSources<Value=T>>,
    target: Option<Box<dyn Target<T>>>,
}

impl<T: Convenient> AnyBindingNode for BindingNode<T> {
    fn unhandle_sources_and_clear_target(&mut self, state: &mut dyn State) {
        self.target.as_ref().map(|x| x.clear(state));
        self.sources.unhandle(state);
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct AnyBindingBase(Id<BoxedBindingNode>);
}

impl AnyBindingBase {
    pub fn drop_binding(self, state: &mut dyn State) {
        let bindings: &mut Bindings = state.get_mut();
        let mut node = bindings.0.remove(self.0);
        node.0.unhandle_sources_and_clear_target(state);
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct BindingBase<T: Convenient>(Id<BoxedBindingNode>, PhantomType<T>);
}

impl<T: Convenient> BindingBase<T> {
    pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
        node.target = Some(target);
        assert!(node.sources.is_empty(), "set_target/bind should be called before any set_source_*");
    }

    pub fn dispatch<Context: Debug + Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T) -> BYield<!>
    ) {
        self.set_target(state, Box::new(DispatchTarget { context, execute }));
    }

    pub fn set_target_fn<Context: Debug + Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T)
    ) {
        self.set_target(state, Box::new(FnTarget { context, execute }));
    }

    pub fn drop_binding(self, state: &mut dyn State) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0.remove(self.0);
        let mut node = node.0.downcast::<BindingNode<T>>().unwrap();
        node.unhandle_sources_and_clear_target(state);
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct Binding<T: Convenient>(Id<BoxedBindingNode>, PhantomType<T>);
}

impl<T: Convenient> From<Binding<T>> for BindingBase<T> {
    fn from(binding: Binding<T>) -> Self {
        BindingBase(binding.0, PhantomType::new())
    }
}

impl<T: Convenient> From<Binding<T>> for AnyBindingBase {
    fn from(v: Binding<T>) -> AnyBindingBase {
        AnyBindingBase(v.0)
    }
}

impl<T: Convenient> Binding<T> {
    pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
        BindingBase::from(self).set_target(state, target);
    }

    pub fn dispatch<Context: Debug + Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T) -> BYield<!>
    ) {
        BindingBase::from(self).dispatch(state, context, execute);
    }

    pub fn set_target_fn<Context: Debug + Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T)
    ) {
        BindingBase::from(self).set_target_fn(state, context, execute);
    }

    pub fn drop_binding(self, state: &mut dyn State) {
        BindingBase::from(self).drop_binding(state);
    }

    pub fn get_value(self, state: &dyn State) -> Option<T> {
        let bindings: &Bindings = state.get();
        let node = bindings.0[self.0].0.downcast_ref::<BindingNode<T>>().unwrap();
        node.sources.get_value()
    }
}

impl<T: Convenient> From<BindingBase<T>> for AnyBindingBase {
    fn from(v: BindingBase<T>) -> AnyBindingBase {
        AnyBindingBase(v.0)
    }
}

macro_rules! binding_n {
    ($n:tt; $($i:tt),* $(,)?) => {
        binding_n! { @unwrap [] [$n] [$($i)*] [$($i)*] }
    };
    (@unwrap [$($r:tt)*] [$n:tt] [] [$($j:tt)*]) => {
        binding_n! { @done [$n] $($r)* }
    };
    (@unwrap [$($r:tt)*] [$n:tt] [$i0:tt $($i:tt)*] [$($j:tt)*]) => {
        binding_n! { @unwrap [$($r)* [$i0 $($j)+]] [$n] [$($i)*] [$($j)*] }
    };
    (@unwrap $($x:tt)*) => {
        compile_error!(stringify! { $($x)* });
    };
    (@done [$n:tt] $([$i:tt $($j:tt)*])*) => {
        $crate::paste_paste! {
            #[derive(Educe)]
            #[educe(Debug)]
            struct [< BindingExt $n NodeSources >] <P: Debug, $( [< S $i >] : Source, )* T: Convenient> {
                param: P,
                $(
                    [< source_ $i >] : Option<(Box<dyn HandlerId>, [< S $i >] ::Cache )>,
                )*
                #[allow(dead_code)]
                #[educe(Debug(ignore))]
                dispatch: fn(
                    &mut dyn State,
                    Glob <  P >,
                    $( < < [< S $i >] as Source > ::Cache as SourceCache< [< S $i >] ::Value > >::Value ),*
                ) -> BYield<T>,
            }

            impl<
                P: Debug + 'static,
                $( [< S $i >] : Source + 'static, )*
                T: Convenient
            > AnyBindingNodeSources for [< BindingExt $n NodeSources >] <P, $( [< S $i >] , )* T> {
                type Value = T;

                fn is_empty(&self) -> bool {
                    $(
                        if self. [< source_ $i >] .is_some() {
                            return false;
                        }
                    )*
                    true
                }

                #[allow(unused_variables)]
                fn unhandle(&mut self, state: &mut dyn State) {
                    $(
                        if let Some(source) = self. [< source_ $i >] .take() {
                            source.0.unhandle(state);
                        }
                    )*
                }

                fn get_value(&self) -> Option<T> {
                    unreachable!()
                }
            }

            macro_attr! {
                #[derive(Educe, NewtypeComponentId!)]
                #[educe(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
                pub struct [< BindingExt $n >] <P, $( [< S $i >] : Source, )* T: Convenient>(
                    Id<BoxedBindingNode>,
                    PhantomType<(P, ($( [< S $i >] ,)* ), T)>
                );
            }

            impl<
                P: Debug + 'static,
                $( [< S $i >] : Source + 'static, )*
                T: Convenient
            > [< BindingExt $n >] <P, $( [< S $i >] , )* T> {
                pub fn new(
                    state: &mut dyn State,
                    param: P,
                    dispatch: fn(
                        &mut dyn State,
                        Glob < P >,
                        $( < < [< S $i >] as Source > ::Cache as SourceCache< [< S $i >] ::Value > >::Value ),*
                    ) -> BYield<T>,
                ) -> Self {
                    let bindings: &mut Bindings = state.get_mut();
                    let id = bindings.0.insert(|id| {
                        let sources: [< BindingExt $n NodeSources >] <P, $( [< S $i >] ,)* T> = [< BindingExt $n NodeSources >] {
                            param,
                            $(
                                [< source_ $i >] : None,
                            )*
                            dispatch,
                        };
                        let node: BindingNode<T> = BindingNode {
                            sources: Box::new(sources),
                            target: None,
                        };
                        (BoxedBindingNode(Box::new(node)), id)
                    });
                    [< BindingExt $n >] (id, PhantomType::new())
                }

                fn param_ref(arena: &dyn Any, id: RawId) -> &P {
                    let bindings = arena.downcast_ref::<Bindings>().unwrap();
                    let node = bindings.0[Id::from_raw(id)].0.downcast_ref::<BindingNode<T>>().unwrap();
                    let sources = node.sources.downcast_ref::< [< BindingExt $n NodeSources >] <P, $( [< S $i >] ,)* T>>().unwrap();
                    &sources.param
                }

                fn param_mut(arena: &mut dyn Any, id: RawId) -> &mut P {
                    let bindings = arena.downcast_mut::<Bindings>().unwrap();
                    let node = bindings.0[Id::from_raw(id)].0.downcast_mut::<BindingNode<T>>().unwrap();
                    let sources = node.sources.downcast_mut::< [< BindingExt $n NodeSources >] <P, $( [< S $i >] ,)* T>>().unwrap();
                    &mut sources.param
                }

                #[allow(dead_code)]
                fn param_descriptor() -> GlobDescriptor<P> {
                    GlobDescriptor {
                        arena: TypeId::of::<Bindings>(),
                        field_ref: Self::param_ref,
                        field_mut: Self::param_mut,
                    }
                }

                pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
                    BindingBase::from(self).set_target(state, target);
                }

                pub fn dispatch<Context: Debug + Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T) -> BYield<!>
                ) {
                    BindingBase::from(self).dispatch(state, context, execute);
                }

                pub fn set_target_fn<Context: Debug + Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T)
                ) {
                    BindingBase::from(self).set_target_fn(state, context, execute);
                }

                pub fn drop_binding(self, state: &mut dyn State) {
                    BindingBase::from(self).drop_binding(state);
                }

                $(
                    pub fn [< set_source_ $i >] (self, state: &mut dyn State, source: &mut [< S $i >] ) {
                        let handler: [< BindingExt $n Source $i Handler >] ::<P, $( [< S $j >] ,)* T>  = [< BindingExt $n Source $i Handler >] {
                            binding: self.0,
                            phantom: PhantomType::new()
                        };
                        let source = source.handle(
                            state,
                            Box::new(handler)
                        );
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingExt $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        if sources. [< source_ $i >] .replace((source.handler_id, [< S $i >] ::Cache::default() )).is_some() {
                            panic!("duplicate source");
                        }
                        source.init.map(|x| x(state));
                    }
                )*
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< BindingExt $n >] <P, $( [< S $i >] , )* T> > for BindingBase<T> {
                fn from(v: [< BindingExt $n >] <P, $( [< S $i >] , )* T> ) -> BindingBase<T> {
                    BindingBase(v.0, PhantomType::new())
                }
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< BindingExt $n >] <P, $( [< S $i >] , )* T> > for AnyBindingBase {
                fn from(v: [< BindingExt $n >] <P, $( [< S $i >] , )* T> ) -> AnyBindingBase {
                    AnyBindingBase(v.0)
                }
            }

            $(
                #[derive(Educe)]
                #[educe(Debug, Clone)]
                struct [< BindingExt $n Source $i Handler >] <
                    P,
                    $( [< S $j >] : Source, )*
                    T: Convenient
                > {
                    binding: Id<BoxedBindingNode>,
                    phantom: PhantomType<(P, $( [< S $j >] ,)* T)>
                }

                impl<
                    P: Debug + 'static,
                    $( [< S $j >] : Source + 'static, )*
                    T: Convenient
                > AnyHandler for [< BindingExt $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn clear(&self, state: &mut dyn State) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingExt $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .take();
                    }
                }

                impl<
                    P: Debug + 'static,
                    $( [< S $j >] : Source + 'static, )*
                    T: Convenient
                > Handler< [< S $i >] ::Value > for [< BindingExt $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                        self
                    }

                    fn execute(&self, state: &mut dyn State, value: [< S $i >] ::Value ) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingExt $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .as_mut().unwrap().1.update(value.clone());
                        $(
                            #[allow(unused_assignments, unused_mut)]
                            let mut [< current_ $j >] = None;
                        )*
                        [< current_ $i >] = Some(value);
                        $(
                            let [< value_ $j >] ;
                            if let Some(source) = sources. [< source_ $j >] .as_ref() {
                                if let Some(source) = source.1.get( [< current_ $j >] ) {
                                    [< value_ $j >] = source;
                                } else {
                                    return;
                                }
                            } else {
                                return;
                            }
                        )*

                        let target = node.target.clone();
                        let param = Glob {
                            id: self.binding.into_raw(),
                            descriptor: < [< BindingExt $n >] <P, $( [< S $j >] ,)* T> > ::param_descriptor
                        };
                        if let BYield(Some(value)) = (sources.dispatch)(state, param, $( [< value_ $j >] ),*) {
                            target.map(|x| x.execute(state, value));
                        }
                    }
                }
            )*

            #[derive(Educe)]
            #[educe(Debug(bound="P: Debug"))]
            struct [< Binding $n NodeSources >] <P, $( [< S $i >] : Source, )* T: Convenient> {
                param: P,
                $(
                    [< source_ $i >] : Option<(Box<dyn HandlerId>, [< S $i >] ::Cache )>,
                )*
                #[educe(Debug(ignore))]
                filter_map: fn(P, $( < < [< S $i >] as Source > ::Cache as SourceCache< [< S $i >] ::Value > >::Value ),* ) -> Option<T>,
            }

            impl<
                P: Clone + 'static,
                $( [< S $i >] : Source + 'static, )*
                T: Convenient
            > AnyBindingNodeSources for [< Binding $n NodeSources >] <P, $( [< S $i >] , )* T> {
                type Value = T;

                fn is_empty(&self) -> bool {
                    $(
                        if self. [< source_ $i >] .is_some() {
                            return false;
                        }
                    )*
                    true
                }

                #[allow(unused_variables)]
                fn unhandle(&mut self, state: &mut dyn State) {
                    $(
                        if let Some(source) = self. [< source_ $i >] .take() {
                            source.0.unhandle(state);
                        }
                    )*
                }

                fn get_value(&self) -> Option<T> {
                    $(
                        let [< value_ $i >] ;
                        if let Some(source) = self. [< source_ $i >] .as_ref() {
                            if let Some(source) = source.1.get(None) {
                                [< value_ $i >] = source;
                            } else {
                                return None;
                            }
                        } else {
                            return None;
                        }
                    )*
                    (self.filter_map)(self.param.clone(), $( [< value_ $i >] ),*)
                }
            }

            macro_attr! {
                #[derive(Educe, NewtypeComponentId!)]
                #[educe(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
                pub struct [< Binding $n >] <P, $( [< S $i >] : Source, )* T: Convenient>(
                    Id<BoxedBindingNode>,
                    PhantomType<(P, ($( [< S $i >] ,)* ), T)>
                );
            }

            impl<
                P: Clone + 'static,
                $( [< S $i >] : Source + 'static, )*
                T: Convenient
            > [< Binding $n >] <P, $( [< S $i >] , )* T> {
                pub fn new(
                    state: &mut dyn State,
                    param: P,
                    filter_map: fn(P, $( < < [< S $i >] as Source > ::Cache as SourceCache< [< S $i >] ::Value > >::Value ),* ) -> Option<T>,
                ) -> Self {
                    let bindings: &mut Bindings = state.get_mut();
                    let id = bindings.0.insert(|id| {
                        let sources: [< Binding $n NodeSources >] <P, $( [< S $i >] ,)* T> = [< Binding $n NodeSources >] {
                            param,
                            $(
                                [< source_ $i >] : None,
                            )*
                            filter_map,
                        };
                        let node: BindingNode<T> = BindingNode {
                            sources: Box::new(sources),
                            target: None,
                        };
                        (BoxedBindingNode(Box::new(node)), id)
                    });
                    [< Binding $n >] (id, PhantomType::new())
                }

                pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
                    BindingBase::from(self).set_target(state, target);
                }

                pub fn dispatch<Context: Debug + Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T) -> BYield<!>
                ) {
                    BindingBase::from(self).dispatch(state, context, execute);
                }

                pub fn set_target_fn<Context: Debug + Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T)
                ) {
                    BindingBase::from(self).set_target_fn(state, context, execute);
                }

                pub fn drop_binding(self, state: &mut dyn State) {
                    BindingBase::from(self).drop_binding(state);
                }

                pub fn get_value(self, state: &dyn State) -> Option<T> {
                    Binding::from(self).get_value(state)
                }

                $(
                    pub fn [< set_source_ $i >] (self, state: &mut dyn State, source: &mut [< S $i >] ) {
                        let handler: [< Binding $n Source $i Handler >] ::<P, $( [< S $j >] ,)* T>  = [< Binding $n Source $i Handler >] {
                            binding: self.0,
                            phantom: PhantomType::new()
                        };
                        let source = source.handle(
                            state,
                            Box::new(handler)
                        );
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        if sources. [< source_ $i >] .replace((source.handler_id, [< S $i >] ::Cache::default() )).is_some() {
                            panic!("duplicate source");
                        }
                        source.init.map(|x| x(state));
                    }
                )*
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< Binding $n >] <P, $( [< S $i >] , )* T> > for Binding<T> {
                fn from(v: [< Binding $n >] <P, $( [< S $i >] , )* T> ) -> Binding<T> {
                    Binding(v.0, PhantomType::new())
                }
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< Binding $n >] <P, $( [< S $i >] , )* T> > for BindingBase<T> {
                fn from(v: [< Binding $n >] <P, $( [< S $i >] , )* T> ) -> BindingBase<T> {
                    BindingBase(v.0, PhantomType::new())
                }
            }

            impl<
                P,
                $( [< S $i >] : Source, )*
                T: Convenient
            > From< [< Binding $n >] <P, $( [< S $i >] , )* T> > for AnyBindingBase {
                fn from(v: [< Binding $n >] <P, $( [< S $i >] , )* T> ) -> AnyBindingBase {
                    AnyBindingBase(v.0)
                }
            }

            $(
                #[derive(Educe)]
                #[educe(Debug, Clone)]
                struct [< Binding $n Source $i Handler >] <
                    P,
                    $( [< S $j >] : Source, )*
                    T: Convenient
                > {
                    binding: Id<BoxedBindingNode>,
                    phantom: PhantomType<(P, $( [< S $j >] ,)* T)>
                }

                impl<
                    P: Clone + 'static,
                    $( [< S $j >] : Source + 'static, )*
                    T: Convenient
                > AnyHandler for [< Binding $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn clear(&self, state: &mut dyn State) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .take();
                    }
                }

                impl<
                    P: Clone + 'static,
                    $( [< S $j >] : Source + 'static, )*
                    T: Convenient
                > Handler< [< S $i >] ::Value > for [< Binding $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                        self
                    }

                    fn execute(&self, state: &mut dyn State, value: [< S $i >] ::Value ) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .as_mut().unwrap().1.update(value.clone());
                        $(
                            #[allow(unused_assignments, unused_mut)]
                            let mut [< current_ $j >] = None;
                        )*
                        [< current_ $i >] = Some(value);
                        $(
                            let [< value_ $j >] ;
                            if let Some(source) = sources. [< source_ $j >] .as_ref() {
                                if let Some(source) = source.1.get( [< current_ $j >] ) {
                                    [< value_ $j >] = source;
                                } else {
                                    return;
                                }
                            } else {
                                return;
                            }
                        )*

                        if let Some(value) = (sources.filter_map)(sources.param.clone(), $( [< value_ $j >] ),*) {
                            if let Some(target) = node.target.clone() {
                                target.execute(state, value);
                            }
                        }
                    }
                }
            )*
        }
    };
}

binding_n!(0;);
binding_n!(1; 1);
binding_n!(2; 1, 2);
binding_n!(3; 1, 2, 3);
binding_n!(4; 1, 2, 3, 4);
binding_n!(5; 1, 2, 3, 4, 5);
binding_n!(6; 1, 2, 3, 4, 5, 6);
binding_n!(7; 1, 2, 3, 4, 5, 6, 7);
binding_n!(8; 1, 2, 3, 4, 5, 6, 7, 8);
binding_n!(9; 1, 2, 3, 4, 5, 6, 7, 8, 9);
binding_n!(10; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10);
binding_n!(11; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11);
binding_n!(12; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12);
binding_n!(13; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13);
binding_n!(14; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14);
binding_n!(15; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15);
binding_n!(16; 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16);
