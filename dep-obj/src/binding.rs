use crate::base::*;
use alloc::boxed::Box;
use components_arena::{Component, ComponentId, Id, Arena, RawId, NewtypeComponentId};
use core::fmt::Debug;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::{SelfState, State, StateExt};
use educe::Educe;
use macro_attr_2018::macro_attr;
use phantom_type::PhantomType;

pub trait Handler<T: Convenient>: Debug + DynClone {
    fn execute(&self, state: &mut dyn State, value: T);
}

clone_trait_object!(<T: Convenient> Handler<T>);

#[derive(Educe)]
#[educe(Debug, Clone)]
pub struct HandlerFn<Context: Debug + Clone, T: Convenient> {
    context: Context,
    #[educe(Debug(ignore))]
    execute: fn(state: &mut dyn State, context: Context, value: T)
}

impl<Context: Debug + Clone, T: Convenient> Handler<T> for HandlerFn<Context, T> {
    fn execute(&self, state: &mut dyn State, value: T) {
        (self.execute)(state, self.context.clone(), value);
    }
}

pub trait Source<T: Convenient>: Debug {
    fn handle(&self, state: &mut dyn State, handler: Box<dyn Handler<T>>) -> HandledSource<T>;
}

pub trait HandlerId: Debug {
    fn unhandle(&self, state: &mut dyn State);
}

#[derive(Debug)]
pub struct HandledSource<T: Convenient> {
    pub handler_id: Box<dyn HandlerId>,
    pub value: T,
}

trait AnyBindingNode: Debug + Downcast {
    fn unhandle_sources(&mut self, state: &mut dyn State);
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
    handler: Option<Box<dyn Handler<T>>>,
}

impl<T: Convenient> AnyBindingNode for BindingNode<T> {
    fn unhandle_sources(&mut self, state: &mut dyn State) {
        self.sources.unhandle(state);
    }
}

macro_attr! {
    #[derive(Educe, NewtypeComponentId!)]
    #[educe(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
    pub struct Binding<T: Convenient>(Id<BoxedBindingNode>, PhantomType<T>);
}

impl<T: Convenient> Binding<T> {
    pub fn handle(self, state: &mut dyn State, handler: Box<dyn Handler<T>>) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
        node.handler = Some(handler.clone());
        self.knoke_handler(state);
    }

    pub fn handle_fn<Context: Debug + Clone + 'static>(
        self,
        state: &mut dyn State,
        context: Context,
        execute: fn(state: &mut dyn State, context: Context, value: T)
    ) {
        self.handle(state, Box::new(HandlerFn { context, execute }));
    }

    pub fn drop_binding(self, state: &mut dyn State) {
        let bindings: &mut Bindings = state.get_mut();
        let node = bindings.0.remove(self.0);
        let mut node = node.0.downcast::<BindingNode<T>>().unwrap();
        node.sources.unhandle(state);
    }

    pub fn get_value(self, state: &dyn State) -> Option<T> {
        let bindings: &Bindings = state.get();
        let node = bindings.0[self.0].0.downcast_ref::<BindingNode<T>>().unwrap();
        node.sources.get_value()
    }

    fn knoke_handler(self, state: &mut dyn State) {
        let bindings: &Bindings = state.get();
        let node = bindings.0[self.0].0.downcast_ref::<BindingNode<T>>().unwrap();
        if let Some(value) = node.sources.get_value() {
            if let Some(handler) = node.handler.clone() {
                handler.execute(state, value);
            }
        }
    }
}

macro_rules! binding_n {
    ($n:tt; $($i:tt),+ $(,)?) => {
        binding_n! { @unwrap [] [$n] [$($i)+] [$($i)+] }
    };
    (@unwrap [$($r:tt)*] [$n:tt] [] [$($j:tt)+]) => {
        binding_n! { @done [$n] $($r)* }
    };
    (@unwrap [$($r:tt)*] [$n:tt] [$i0:tt $($i:tt)*] [$($j:tt)+]) => {
        binding_n! { @unwrap [$($r)* [$i0 $($j)+]] [$n] [$($i)*] [$($j)+] }
    };
    (@unwrap $($x:tt)*) => {
        compile_error!(stringify! { $($x)* });
    };
    (@done [$n:tt] $([$i:tt $($j:tt)+])+) => {
        $crate::paste_paste! {
            #[derive(Educe)]
            #[educe(Debug)]
            struct [< Binding $n NodeSources >] < $( [< S $i >] : Convenient, )+ T: Convenient> {
                $(
                    [< source_ $i >] : Option<HandledSource< [< S $i >] >>,
                )+
                #[educe(Debug(ignore))]
                filter_map: fn( $( [< S $i >] ),+ ) -> Option<T>,
            }

            impl<
                $( [< S $i >] : Convenient, )+
                T: Convenient
            > AnyBindingNodeSources for [< Binding $n NodeSources >] <$( [< S $i >] , )+ T> {
                type Value = T;

                fn unhandle(&mut self, state: &mut dyn State) {
                    $(
                        if let Some(source) = self. [< source_ $i >] .take() {
                            source.handler_id.unhandle(state);
                        }
                    )+
                }

                fn get_value(&self) -> Option<T> {
                    $(
                        let [< value_ $i >] ;
                        if let Some(source) = self. [< source_ $i >] .as_ref() {
                            [< value_ $i >] = source.value.clone();
                        } else {
                            return None;
                        }
                    )+
                    (self.filter_map)($( [< value_ $i >] ),+)
                }
            }

            macro_attr! {
                #[derive(Educe, NewtypeComponentId!)]
                #[educe(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd, Hash)]
                pub struct [< Binding $n >] < $( [< S $i >] : Convenient, )+ T: Convenient>(
                    Id<BoxedBindingNode>,
                    PhantomType<($( [< S $i >] ),+ , T)>
                );
            }

            impl<
                $( [< S $i >] : Convenient, )+
                T: Convenient
            > [< Binding $n >] < $( [< S $i >] , )+ T> {
                pub fn new(
                    bindings: &mut Bindings,
                    filter_map: fn( $( [< S $i >] ),+ ) -> Option<T>
                ) -> Self {
                    let id = bindings.0.insert(|id| {
                        let sources: [< Binding $n NodeSources >] <$( [< S $i >] ),+ , T> = [< Binding $n NodeSources >] {
                            $(
                                [< source_ $i >] : None,
                            )+
                            filter_map,
                        };
                        let node: BindingNode<T> = BindingNode {
                            sources: Box::new(sources),
                            handler: None,
                        };
                        (BoxedBindingNode(Box::new(node)), id)
                    });
                    [< Binding $n >] (id, PhantomType::new())
                }

                pub fn handle(self, state: &mut dyn State, handler: Box<dyn Handler<T>>) {
                    Binding::from(self).handle(state, handler);
                }

                pub fn handle_fn<Context: Debug + Clone + 'static>(
                    self,
                    state: &mut dyn State,
                    context: Context,
                    execute: fn(state: &mut dyn State, context: Context, value: T)
                ) {
                    Binding::from(self).handle_fn(state, context, execute);
                }

                pub fn get_value(self, state: &dyn State) -> Option<T> {
                    Binding::from(self).get_value(state)
                }

                $(
                    pub fn [< set_source_ $i >] (self, state: &mut dyn State, source: &mut dyn Source< [< S $i >] >) {
                        let handler: [< Binding $n Source $i Handler >] ::<$( [< S $j >] ),+ , T>  = [< Binding $n Source $i Handler >] {
                            binding: self.into_raw(),
                            phantom: PhantomType::new()
                        };
                        let source = source.handle(
                            state,
                            Box::new(handler)
                        );
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.0].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <$( [< S $j >] ),+ , T>>().unwrap();
                        if let Some(source) = sources. [< source_ $i >] .replace(source) {
                            source.handler_id.unhandle(state);
                        }
                        Binding::from(self).knoke_handler(state);
                    }
                )+
            }

            impl<
                $( [< S $i >] : Convenient, )+
                T: Convenient
            > From< [< Binding $n >] < $( [< S $i >] , )+ T> > for Binding<T> {
                fn from(v: [< Binding $n >] < $( [< S $i >] , )+ T> ) -> Binding<T> {
                    Binding(v.0, PhantomType::new())
                }
            }

            $(
                #[derive(Debug, Clone)]
                struct [< Binding $n Source $i Handler >] <
                    $( [< S $j >] : Convenient, )+
                    T: Convenient
                > {
                    binding: RawId,
                    phantom: PhantomType<($( [< S $j >] ),+ , T)>
                }

                impl<
                    $( [< S $j >] : Convenient, )+
                    T: Convenient
                > Handler< [< S $i >] > for [< Binding $n Source $i Handler >] < $( [< S $j >] , )+ T >  {
                    fn execute(&self, state: &mut dyn State, value: [< S $i >] ) {
                        let binding: [< Binding $n >] <$( [< S $j >] ),+ , T> = [< Binding $n >] ::from_raw(self.binding);
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[binding.0].0.downcast_mut::<BindingNode<T>>().unwrap();
                        let sources = node.sources.downcast_mut::< [< Binding $n NodeSources >] <$( [< S $j >] ),+ , T>>().unwrap();
                        sources. [< source_ $i >] .as_mut().unwrap().value = value.clone();
                        Binding::from(binding).knoke_handler(state);
                    }
                }
            )+
        }
    };
}

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

