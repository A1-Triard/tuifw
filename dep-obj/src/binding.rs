use crate::base::*;
use alloc::boxed::Box;
use components_arena::{Component, Id, Arena, NewtypeComponentId};
use core::fmt::Debug;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::state::{SelfState, State, StateExt};
use educe::Educe;
use macro_attr_2018::macro_attr;
use panicking::panicking;
use phantom_type::PhantomType;

pub trait Target<T: Convenient>: Debug + DynClone {
    fn execute(&self, state: &mut dyn State, value: T);
    fn clear(&self, state: &mut dyn State);
}

clone_trait_object!(<T: Convenient> Target<T>);

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

pub trait Handler<T: Convenient>: Debug + DynClone + Send + Sync {
    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler>;
    fn execute(&self, state: &mut dyn State, value: T);
}

clone_trait_object!(<T: Convenient> Handler<T>);

pub trait SyncHandler<T>: Debug + DynClone + Send + Sync {
    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler>;
    fn execute(&self, state: &mut dyn State, value: &mut T);
}

clone_trait_object!(<T> SyncHandler<T>);

pub trait Source<T: Convenient>: Debug {
    fn handle(&self, state: &mut dyn State, handler: Box<dyn Handler<T>>) -> HandledSource<T>;
}

pub trait SyncSource<T>: Debug {
    fn handle(&self, state: &mut dyn State, handler: Box<dyn SyncHandler<T>>) -> HandledSyncSource<T>;
}

pub trait HandlerId: Debug {
    fn unhandle(&self, state: &mut dyn State);
}

#[derive(Debug)]
pub struct HandledSource<T: Convenient> {
    pub handler_id: Box<dyn HandlerId>,
    pub value: T,
}

#[derive(Educe)]
#[educe(Debug)]
pub struct HandledSyncSource<'a, T> {
    pub handler_id: Box<dyn HandlerId>,
    #[educe(Debug(ignore))]
    pub value: Option<&'a mut T>,
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

trait AnyBindingNodeSources: Debug + Downcast {
    type Value: Convenient;
    fn unhandle(&mut self, state: &mut dyn State);
    fn get_value(&self) -> Option<Self::Value>;
}

impl_downcast!(AnyBindingNodeSources assoc Value where Value: Convenient);

#[derive(Educe)]
#[educe(Debug)]
struct BindingNode<T: Convenient> {
    sources: Box<dyn AnyBindingNodeSources<Value=T>>,
    target: Option<Box<dyn Target<T>>>,
}

impl<T: Convenient> AnyBindingNode for BindingNode<T> {
    fn unhandle_sources_and_clear_target(&mut self, state: &mut dyn State) {
        self.sources.unhandle(state);
        self.target.as_ref().map(|x| x.clear(state));
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct AnyBinding(Id<BoxedBindingNode>);
}

impl AnyBinding {
    pub fn drop_binding(self, state: &mut dyn State) {
        let bindings: &mut Bindings = state.get_mut();
        let mut node = bindings.0.remove(self.0);
        node.0.unhandle_sources_and_clear_target(state);
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct Binding<T: Convenient>(Id<BoxedBindingNode>, PhantomType<T>);
}

impl<T: Convenient> Binding<T> {
    pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
        node.target = Some(target);
        knoke_target::<T>(self.0, state);
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

    pub fn get_value(self, state: &dyn State) -> Option<T> {
        let bindings: &Bindings = state.get();
        let node = bindings.0[self.0].0.downcast_ref::<BindingNode<T>>().unwrap();
        node.sources.get_value()
    }
}

fn knoke_target<T: Convenient>(binding: Id<BoxedBindingNode>, state: &mut dyn State) {
    let bindings: &Bindings = state.get();
    let node = bindings.0[binding].0.downcast_ref::<BindingNode<T>>().unwrap();
    if let Some(value) = node.sources.get_value() {
        if let Some(target) = node.target.clone() {
            target.execute(state, value);
        }
    }
}

impl<T: Convenient> From<Binding<T>> for AnyBinding {
    fn from(v: Binding<T>) -> AnyBinding {
        AnyBinding(v.0)
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
            struct [< Binding $n NodeSources >] <P: Convenient, $( [< S $i >] : Convenient, )* T: Convenient> {
                param: P,
                $(
                    [< source_ $i >] : Option<HandledSource< [< S $i >] >>,
                )*
                #[educe(Debug(ignore))]
                filter_map: fn(P, $( [< S $i >] ),* ) -> Option<T>,
            }

            impl<
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                T: Convenient
            > AnyBindingNodeSources for [< Binding $n NodeSources >] <P, $( [< S $i >] , )* T> {
                type Value = T;

                #[allow(unused_variables)]
                fn unhandle(&mut self, state: &mut dyn State) {
                    $(
                        if let Some(source) = self. [< source_ $i >] .take() {
                            source.handler_id.unhandle(state);
                        }
                    )*
                }

                fn get_value(&self) -> Option<T> {
                    $(
                        let [< value_ $i >] ;
                        if let Some(source) = self. [< source_ $i >] .as_ref() {
                            [< value_ $i >] = source.value.clone();
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
                pub struct [< Binding $n >] <P, $( [< S $i >] : Convenient, )* T: Convenient>(
                    Id<BoxedBindingNode>,
                    PhantomType<(P, ($( [< S $i >] ,)* ), T)>
                );
            }

            impl<
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                T: Convenient
            > [< Binding $n >] <P, $( [< S $i >] , )* T> {
                pub fn new(
                    state: &mut dyn State,
                    param: P,
                    filter_map: fn(P, $( [< S $i >] ),* ) -> Option<T>
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
                    Binding::from(self).set_target(state, target);
                }

                pub fn set_target_fn<Context: Debug + Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T)
                ) {
                    Binding::from(self).set_target_fn(state, context, execute);
                }

                pub fn drop_binding(self, state: &mut dyn State) {
                    Binding::from(self).drop_binding(state);
                }

                pub fn get_value(self, state: &dyn State) -> Option<T> {
                    Binding::from(self).get_value(state)
                }

                $(
                    pub fn [< set_source_ $i >] (self, state: &mut dyn State, source: &mut dyn Source< [< S $i >] >) {
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
                        if let Some(source) = sources. [< source_ $i >] .replace(source) {
                            source.handler_id.unhandle(state);
                        }
                        knoke_target::<T>(self.0, state);
                    }
                )*
            }

            impl<
                P,
                $( [< S $i >] : Convenient, )*
                T: Convenient
            > From< [< Binding $n >] <P, $( [< S $i >] , )* T> > for Binding<T> {
                fn from(v: [< Binding $n >] <P, $( [< S $i >] , )* T> ) -> Binding<T> {
                    Binding(v.0, PhantomType::new())
                }
            }

            impl<
                P,
                $( [< S $i >] : Convenient, )*
                T: Convenient
            > From< [< Binding $n >] <P, $( [< S $i >] , )* T> > for AnyBinding {
                fn from(v: [< Binding $n >] <P, $( [< S $i >] , )* T> ) -> AnyBinding {
                    AnyBinding(v.0)
                }
            }

            $(
                #[derive(Educe)]
                #[educe(Debug, Clone)]
                struct [< Binding $n Source $i Handler >] <
                    P,
                    $( [< S $j >] : Convenient, )*
                    T: Convenient
                > {
                    binding: Id<BoxedBindingNode>,
                    phantom: PhantomType<(P, $( [< S $j >] ,)* T)>
                }

                impl<
                    P: Convenient,
                    $( [< S $j >] : Convenient, )*
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
                    P: Convenient,
                    $( [< S $j >] : Convenient, )*
                    T: Convenient
                > Handler< [< S $i >] > for [< Binding $n Source $i Handler >] <P, $( [< S $j >] , )* T >  {
                    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                        self
                    }

                    fn execute(&self, state: &mut dyn State, value: [< S $i >] ) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <P, $( [< S $j >] ,)* T>>().unwrap();
                        sources. [< source_ $i >] .as_mut().unwrap().value = value;
                        knoke_target::<T>(self.binding, state);
                    }
                }
            )*

            fn [< knoke_target_y_ $n >] <P: Convenient, $( [< S $i >] : Convenient, )* Y: 'static, T: Convenient>(binding: Id<BoxedBindingNode>, state: &mut dyn State, sync_value: &mut Y) {
                let bindings: &Bindings = state.get();
                let node = bindings.0[binding].0.downcast_ref::<BindingNode<T>>().unwrap();
                let sources = node.sources.downcast_ref::< [< BindingY $n NodeSources >] <P, $( [< S $i >] ,)* Y, T>>().unwrap();
                $(
                    let [< value_ $i >] ;
                    if let Some(source) = sources. [< source_ $i >] .as_ref() {
                        [< value_ $i >] = source.value.clone();
                    } else {
                        return;
                    }
                )*
                if let Some(target) = node.target.clone() {
                    let filter_map = sources.filter_map;
                    let param = sources.param.clone();
                    if let Some(value) = filter_map(state, param, $( [< value_ $i >] ,)* sync_value) {
                        target.execute(state, value);
                    }
                }
            }

            #[derive(Educe)]
            #[educe(Debug)]
            struct [< BindingY $n NodeSources >] <P: Convenient, $( [< S $i >] : Convenient, )* Y, T: Convenient> {
                param: P,
                $(
                    [< source_ $i >] : Option<HandledSource< [< S $i >] >>,
                )*
                sync_source: Option<Box<dyn HandlerId>>,
                #[educe(Debug(ignore))]
                filter_map: fn(&mut dyn State, P, $( [< S $i >] ,)* &mut Y) -> Option<T>,
            }

            impl<
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                Y: 'static, T: Convenient
            > AnyBindingNodeSources for [< BindingY $n NodeSources >] <P, $( [< S $i >] , )* Y, T> {
                type Value = T;

                #[allow(unused_variables)]
                fn unhandle(&mut self, state: &mut dyn State) {
                    $(
                        if let Some(source) = self. [< source_ $i >] .take() {
                            source.handler_id.unhandle(state);
                        }
                    )*
                    if let Some(sync_source) = self.sync_source.take() {
                        sync_source.unhandle(state);
                    }
                }

                fn get_value(&self) -> Option<T> { None }
            }

            macro_attr! {
                #[derive(Educe, NewtypeComponentId!)]
                #[educe(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
                pub struct [< BindingY $n >] <P: Convenient, $( [< S $i >] : Convenient, )* Y, T: Convenient>(
                    Id<BoxedBindingNode>,
                    PhantomType<(P, ($( [< S $i >] ,)* ), Y, T)>
                );
            }

            impl<
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                Y: 'static, T: Convenient
            > [< BindingY $n >] <P, $( [< S $i >] , )* Y, T> {
                pub fn new(
                    state: &mut dyn State,
                    param: P,
                    filter_map: fn(&mut dyn State, P, $( [< S $i >] ,)* &mut Y) -> Option<T>
                ) -> Self {
                    let bindings: &mut Bindings = state.get_mut();
                    let id = bindings.0.insert(|id| {
                        let sources: [< BindingY $n NodeSources >] <P, $( [< S $i >] ,)* Y, T> = [< BindingY $n NodeSources >] {
                            param,
                            $(
                                [< source_ $i >] : None,
                            )*
                            sync_source: None,
                            filter_map,
                        };
                        let node: BindingNode<T> = BindingNode {
                            sources: Box::new(sources),
                            target: None,
                        };
                        (BoxedBindingNode(Box::new(node)), id)
                    });
                    [< BindingY $n >] (id, PhantomType::new())
                }

                pub fn set_target(self, state: &mut dyn State, target: Box<dyn Target<T>>) {
                    Binding::from(self).set_target(state, target);
                }

                pub fn set_target_fn<Context: Debug + Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T)
                ) {
                    Binding::from(self).set_target_fn(state, context, execute);
                }

                pub fn drop_binding(self, state: &mut dyn State) {
                    Binding::from(self).drop_binding(state);
                }

                $(
                    pub fn [< set_source_ $i >] (self, state: &mut dyn State, source: &mut dyn Source< [< S $i >] >) {
                        let handler: [< BindingY $n Source $i Handler >] ::<P, $( [< S $j >] ,)* Y, T>  = [< BindingY $n Source $i Handler >] {
                            binding: self.0,
                            phantom: PhantomType::new()
                        };
                        let source = source.handle(
                            state,
                            Box::new(handler)
                        );
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingY $n NodeSources >] <P, $( [< S $j >] ),* , Y, T>>().unwrap();
                        if let Some(source) = sources. [< source_ $i >] .replace(source) {
                            source.handler_id.unhandle(state);
                        }
                    }
                )*

                pub fn set_sync_source(self, state: &mut dyn State, source: &mut dyn SyncSource<Y>) {
                    let handler: [< BindingY $n SyncSourceHandler >] ::<P, $( [< S $i >] ,)* Y, T>  = [< BindingY $n SyncSourceHandler >] {
                        binding: self.0,
                        phantom: PhantomType::new()
                    };
                    let source = source.handle(
                        state,
                        Box::new(handler)
                    );
                    let bindings: &mut Bindings = state.get_mut();
                    let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
                    let sources = node.sources.downcast_mut::< [< BindingY $n NodeSources >] <P, $( [< S $i >] ,)* Y, T>>().unwrap();
                    if let Some(source) = sources.sync_source.replace(source.handler_id) {
                        source.unhandle(state);
                    }
                    if let Some(value) = source.value {
                        [< knoke_target_y_ $n >] ::<P, $( [< S $i >] ,)* Y, T > (self.0, state, value);
                    }
                }
            }

            impl<
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                Y, T: Convenient
            > From< [< BindingY $n >] <P, $( [< S $i >] , )* Y, T> > for Binding<T> {
                fn from(v: [< BindingY $n >] <P, $( [< S $i >] , )* Y, T> ) -> Binding<T> {
                    Binding(v.0, PhantomType::new())
                }
            }

            impl<
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                Y, T: Convenient
            > From< [< BindingY $n >] <P, $( [< S $i >] , )* Y, T> > for AnyBinding {
                fn from(v: [< BindingY $n >] <P, $( [< S $i >] , )* Y, T> ) -> AnyBinding {
                    AnyBinding(v.0)
                }
            }

            $(
                #[derive(Educe)]
                #[educe(Debug, Clone)]
                struct [< BindingY $n Source $i Handler >] <
                    P: Convenient,
                    $( [< S $j >] : Convenient, )*
                    Y, T: Convenient
                > {
                    binding: Id<BoxedBindingNode>,
                    phantom: PhantomType<(P, $( [< S $j >] ,)* Y, T)>
                }

                impl<
                    P: Convenient,
                    $( [< S $j >] : Convenient, )*
                    Y: 'static, T: Convenient
                > AnyHandler for [< BindingY $n Source $i Handler >] <P, $( [< S $j >] , )* Y, T >  {
                    fn clear(&self, state: &mut dyn State) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingY $n NodeSources >] <P, $( [< S $j >] ,)* Y, T>>().unwrap();
                        sources. [< source_ $i >] .take();
                    }
                }

                impl<
                    P: Convenient,
                    $( [< S $j >] : Convenient, )*
                    Y: 'static, T: Convenient
                > Handler< [< S $i >] > for [< BindingY $n Source $i Handler >] <P, $( [< S $j >] , )* Y, T >  {
                    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                        self
                    }

                    fn execute(&self, state: &mut dyn State, value: [< S $i >] ) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< BindingY $n NodeSources >] <P, $( [< S $j >] ,)* Y, T>>().unwrap();
                        sources. [< source_ $i >] .as_mut().unwrap().value = value;
                    }
                }
            )*

            #[derive(Educe)]
            #[educe(Debug, Clone)]
            struct [< BindingY $n SyncSourceHandler >] <
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                Y, T: Convenient
            > {
                binding: Id<BoxedBindingNode>,
                phantom: PhantomType<(P, $( [< S $i >] ,)* Y, T)>
            }

            impl<
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                Y: 'static, T: Convenient
            > AnyHandler for [< BindingY $n SyncSourceHandler >] <P, $( [< S $i >] , )* Y, T >  {
                fn clear(&self, state: &mut dyn State) {
                    let bindings: &mut Bindings = state.get_mut();
                    let node = bindings.0[self.binding].0.downcast_mut::<BindingNode<T>>().unwrap();
                    let sources = node.sources.downcast_mut::< [< BindingY $n NodeSources >] <P, $( [< S $i >] ,)* Y, T>>().unwrap();
                    sources.sync_source.take();
                }
            }

            impl<
                P: Convenient,
                $( [< S $i >] : Convenient, )*
                Y: 'static, T: Convenient
            > SyncHandler<Y> for [< BindingY $n SyncSourceHandler >] <P, $( [< S $i >] , )* Y, T >  {
                fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                    self
                }

                fn execute(&self, state: &mut dyn State, value: &mut Y) {
                    [< knoke_target_y_ $n >] ::<P, $( [< S $i >] ,)* Y, T > (self.binding, state, value);
                }
            }
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
