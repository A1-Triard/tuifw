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

pub trait RefTarget<T>: Debug + DynClone {
    fn execute(&self, state: &mut dyn State, value: &mut T);
    fn clear(&self, state: &mut dyn State);
}

clone_trait_object!(<T> RefTarget<T>);

#[derive(Educe)]
#[educe(Debug, Clone)]
struct FnRefTarget<Context: Debug + Clone, T> {
    context: Context,
    #[educe(Debug(ignore))]
    execute: fn(state: &mut dyn State, context: Context, value: &mut T)
}

impl<Context: Debug + Clone, T> RefTarget<T> for FnRefTarget<Context, T> {
    fn execute(&self, state: &mut dyn State, value: &mut T) {
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

trait AnyRefBindingNodeSources: Debug + Downcast {
    fn unhandle(&mut self, state: &mut dyn State);
}

impl_downcast!(AnyRefBindingNodeSources);

#[derive(Educe)]
#[educe(Debug)]
struct RefBindingNode<T> {
    sources: Box<dyn AnyRefBindingNodeSources>,
    target: Option<Box<dyn RefTarget<T>>>,
}

impl<T: 'static> AnyBindingNode for RefBindingNode<T> {
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
    pub struct RefBinding<T>(Id<BoxedBindingNode>, PhantomType<T>);
}

impl<T: 'static> RefBinding<T> {
    pub fn set_target(self, state: &mut dyn State, target: Box<dyn RefTarget<T>>) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0[self.0].0.downcast_mut::<RefBindingNode<T>>().unwrap();
        node.target = Some(target);
    }

    pub fn set_target_fn<Context: Debug + Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: &mut T)
    ) {
        self.set_target(state, Box::new(FnRefTarget { context, execute }));
    }

    pub fn drop_binding(self, state: &mut dyn State) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0.remove(self.0);
        let mut node = node.0.downcast::<RefBindingNode<T>>().unwrap();
        node.unhandle_sources_and_clear_target(state);
    }
}

impl<T: Convenient> From<RefBinding<T>> for AnyBinding {
    fn from(v: RefBinding<T>) -> AnyBinding {
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
            fn [< knoke_ref_target_y_ $n >] < $( [< S $i >] : Convenient, )* Y: 'static, T: 'static>(binding: Id<BoxedBindingNode>, state: &mut dyn State, sync_value: &mut Y) {
                let bindings: &Bindings = state.get();
                let node = bindings.0[binding].0.downcast_ref::<RefBindingNode<T>>().unwrap();
                let sources = node.sources.downcast_ref::< [< RefBindingY $n NodeSources >] <$( [< S $i >] ,)* Y, T>>().unwrap();
                if let Some(value) = sources.get_value_priv(sync_value) {
                    if let Some(target) = node.target.clone() {
                        target.execute(state, value);
                    }
                }
            }

            #[derive(Educe)]
            #[educe(Debug)]
            struct [< RefBindingY $n NodeSources >] < $( [< S $i >] : Convenient, )* Y, T> {
                $(
                    [< source_ $i >] : Option<HandledSource< [< S $i >] >>,
                )*
                sync_source: Option<Box<dyn HandlerId>>,
                #[educe(Debug(ignore))]
                filter_map: fn( $( [< S $i >] ,)* &mut Y) -> Option<&mut T>,
            }

            impl<
                $( [< S $i >] : Convenient, )*
                Y: 'static, T
            > [< RefBindingY $n NodeSources >] <$( [< S $i >] , )* Y, T> {
                fn get_value_priv<'a>(&self, sync_value: &'a mut Y) -> Option<&'a mut T> {
                    $(
                        let [< value_ $i >] ;
                        if let Some(source) = self. [< source_ $i >] .as_ref() {
                            [< value_ $i >] = source.value.clone();
                        } else {
                            return None;
                        }
                    )*
                    (self.filter_map)($( [< value_ $i >] ,)* sync_value)
                }

            }

            impl<
                $( [< S $i >] : Convenient, )*
                Y: 'static, T: 'static
            > AnyRefBindingNodeSources for [< RefBindingY $n NodeSources >] <$( [< S $i >] , )* Y, T> {
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
            }

            macro_attr! {
                #[derive(Educe, NewtypeComponentId!)]
                #[educe(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
                pub struct [< RefBindingY $n >] < $( [< S $i >] : Convenient, )* Y, T>(
                    Id<BoxedBindingNode>,
                    PhantomType<(($( [< S $i >] ,)* ), Y, T)>
                );
            }

            impl<
                $( [< S $i >] : Convenient, )*
                Y: 'static, T: 'static
            > [< RefBindingY $n >] < $( [< S $i >] , )* Y, T> {
                pub fn new(
                    state: &mut dyn State,
                    filter_map: fn( $( [< S $i >] ,)* &mut Y) -> Option<&mut T>
                ) -> Self {
                    let bindings: &mut Bindings = state.get_mut();
                    let id = bindings.0.insert(|id| {
                        let sources: [< RefBindingY $n NodeSources >] <$( [< S $i >] ,)* Y, T> = [< RefBindingY $n NodeSources >] {
                            $(
                                [< source_ $i >] : None,
                            )*
                            sync_source: None,
                            filter_map,
                        };
                        let node: RefBindingNode<T> = RefBindingNode {
                            sources: Box::new(sources),
                            target: None,
                        };
                        (BoxedBindingNode(Box::new(node)), id)
                    });
                    [< RefBindingY $n >] (id, PhantomType::new())
                }

                pub fn set_target(self, state: &mut dyn State, target: Box<dyn RefTarget<T>>) {
                    RefBinding::from(self).set_target(state, target);
                }

                pub fn set_target_fn<Context: Debug + Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: &mut T)
                ) {
                    RefBinding::from(self).set_target_fn(state, context, execute);
                }

                pub fn drop_binding(self, state: &mut dyn State) {
                    RefBinding::from(self).drop_binding(state);
                }

                $(
                    pub fn [< set_source_ $i >] (self, state: &mut dyn State, source: &mut dyn Source< [< S $i >] >) {
                        let handler: [< RefBindingY $n Source $i Handler >] ::<$( [< S $j >] ,)* Y, T>  = [< RefBindingY $n Source $i Handler >] {
                            binding: self.0,
                            phantom: PhantomType::new()
                        };
                        let source = source.handle(
                            state,
                            Box::new(handler)
                        );
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.0].0.downcast_mut::<RefBindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< RefBindingY $n NodeSources >] <$( [< S $j >] ),* , Y, T>>().unwrap();
                        if let Some(source) = sources. [< source_ $i >] .replace(source) {
                            source.handler_id.unhandle(state);
                        }
                    }
                )*

                pub fn set_sync_source(self, state: &mut dyn State, source: &mut dyn SyncSource<Y>) {
                    let handler: [< RefBindingY $n SyncSourceHandler >] ::<$( [< S $i >] ,)* Y, T>  = [< RefBindingY $n SyncSourceHandler >] {
                        binding: self.0,
                        phantom: PhantomType::new()
                    };
                    let source = source.handle(
                        state,
                        Box::new(handler)
                    );
                    let bindings: &mut Bindings = state.get_mut();
                    let node = bindings.0[self.0].0.downcast_mut::<RefBindingNode<T>>().unwrap();
                    let sources = node.sources.downcast_mut::< [< RefBindingY $n NodeSources >] <$( [< S $i >] ,)* Y, T>>().unwrap();
                    if let Some(source) = sources.sync_source.replace(source.handler_id) {
                        source.unhandle(state);
                    }
                    if let Some(value) = source.value {
                        [< knoke_ref_target_y_ $n >] ::< $( [< S $i >] ,)* Y, T > (self.0, state, value);
                    }
                }
            }

            impl<
                $( [< S $i >] : Convenient, )*
                Y, T
            > From< [< RefBindingY $n >] < $( [< S $i >] , )* Y, T> > for RefBinding<T> {
                fn from(v: [< RefBindingY $n >] < $( [< S $i >] , )* Y, T> ) -> RefBinding<T> {
                    RefBinding(v.0, PhantomType::new())
                }
            }

            impl<
                $( [< S $i >] : Convenient, )*
                Y, T
            > From< [< RefBindingY $n >] < $( [< S $i >] , )* Y, T> > for AnyBinding {
                fn from(v: [< RefBindingY $n >] < $( [< S $i >] , )* Y, T> ) -> AnyBinding {
                    AnyBinding(v.0)
                }
            }

            $(
                #[derive(Educe)]
                #[educe(Debug, Clone)]
                struct [< RefBindingY $n Source $i Handler >] <
                    $( [< S $j >] : Convenient, )*
                    Y, T
                > {
                    binding: Id<BoxedBindingNode>,
                    phantom: PhantomType<($( [< S $j >] ,)* Y, T)>
                }

                impl<
                    $( [< S $j >] : Convenient, )*
                    Y: 'static, T: 'static
                > AnyHandler for [< RefBindingY $n Source $i Handler >] < $( [< S $j >] , )* Y, T >  {
                    fn clear(&self, state: &mut dyn State) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<RefBindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< RefBindingY $n NodeSources >] <$( [< S $j >] ,)* Y, T>>().unwrap();
                        sources. [< source_ $i >] .take();
                    }
                }

                impl<
                    $( [< S $j >] : Convenient, )*
                    Y: 'static, T: 'static
                > Handler< [< S $i >] > for [< RefBindingY $n Source $i Handler >] < $( [< S $j >] , )* Y, T >  {
                    fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                        self
                    }

                    fn execute(&self, state: &mut dyn State, value: [< S $i >] ) {
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.binding].0.downcast_mut::<RefBindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< RefBindingY $n NodeSources >] <$( [< S $j >] ,)* Y, T>>().unwrap();
                        sources. [< source_ $i >] .as_mut().unwrap().value = value;
                    }
                }
            )*

            #[derive(Educe)]
            #[educe(Debug, Clone)]
            struct [< RefBindingY $n SyncSourceHandler >] <
                $( [< S $i >] : Convenient, )*
                Y, T
            > {
                binding: Id<BoxedBindingNode>,
                phantom: PhantomType<($( [< S $i >] ,)* Y, T)>
            }

            impl<
                $( [< S $i >] : Convenient, )*
                Y: 'static, T: 'static
            > AnyHandler for [< RefBindingY $n SyncSourceHandler >] < $( [< S $i >] , )* Y, T >  {
                fn clear(&self, state: &mut dyn State) {
                    let bindings: &mut Bindings = state.get_mut();
                    let node = bindings.0[self.binding].0.downcast_mut::<RefBindingNode<T>>().unwrap();
                    let sources = node.sources.downcast_mut::< [< RefBindingY $n NodeSources >] <$( [< S $i >] ,)* Y, T>>().unwrap();
                    sources.sync_source.take();
                }
            }

            impl<
                $( [< S $i >] : Convenient, )*
                Y: 'static, T: 'static
            > SyncHandler<Y> for [< RefBindingY $n SyncSourceHandler >] < $( [< S $i >] , )* Y, T >  {
                fn into_any(self: Box<Self>) -> Box<dyn AnyHandler> {
                    self
                }

                fn execute(&self, state: &mut dyn State, value: &mut Y) {
                    [< knoke_ref_target_y_ $n >] ::< $( [< S $i >] ,)* Y, T > (self.binding, state, value);
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
