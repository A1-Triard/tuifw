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

#[derive(Educe)]
#[educe(Debug, Clone)]
pub struct GlobalHandler<T: Convenient> {
    pub target: Global,
    #[educe(Debug(ignore))]
    pub execute: unsafe fn(state: &mut dyn State, target: Global, value: T),
}

macro_attr! {
    #[derive(Educe, Component!(class=HandlerComponent))]
    #[educe(Debug, Clone)]
    pub struct Handler<T: Convenient> {
        pub target_id: RawId,
        #[educe(Debug(ignore))]
        pub execute: fn(state: &mut dyn State, target_id: RawId, value: T),
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct HandledSource<T: Convenient> {
    pub source: Global,
    pub handler_id: Id<Handler<T>>,
    #[educe(Debug(ignore))]
    pub unhandle: unsafe fn(state: &mut dyn State, source: Global, handler_id: Id<Handler<T>>),
    pub value: T,
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
            struct [< Binding $n Node >] < $( [< S $i >] : Convenient, )+ T: Convenient> {
                $(
                    [< source_ $i >] : Option<HandledSource< [< S $i >] >>,
                )+
                #[educe(Debug(ignore))]
                map: fn( $( [< S $i >] ),+ ) -> T,
                handler: Option<GlobalHandler<T>>,
            }

            impl<
                $( [< S $i >] : Convenient, )+
                T: Convenient
            > AnyBindingNode for [< Binding $n Node >] <$( [< S $i >] , )+ T> { }

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
                    map: fn( $( [< S $i >] ),+ ) -> T
                ) -> Self {
                    let id = bindings.0.insert(|id| {
                        let node: [< Binding $n Node >] <$( [< S $i >] ),+ , T> = [< Binding $n Node >] {
                            $(
                                [< source_ $i >] : None,
                            )+
                            map,
                            handler: None,
                        };
                        (BoxedBindingNode(Box::new(node)), id)
                    });
                    [< Binding $n >] (id, PhantomType::new())
                }

                $(
                    pub fn [< set_source_ $i >] (self, source: &mut dyn Source<Value= [< S $i >] >) {
                        let (source, state) = source.handle(Handler {
                            target_id: self.into_raw(),
                            execute: [< execute_binding_ $n _source_ $i >] ::<$( [< S $j >] ),+ , T>
                        });
                        let bindings: &mut Bindings = state.get_mut();
                        let node = bindings.0[self.0].0.downcast_mut::< [< Binding $n Node >] <$( [< S $j >] ),+ , T>>().unwrap();
                        if let Some(source) = node. [< source_ $i >] .replace(source) {
                            unsafe { (source.unhandle)(state, source.source, source.handler_id); }
                        }
                        self.knoke_handler(state);
                    }
                )+


                fn get_value(self, bindings: &Bindings) -> Option<T> {
                    let node = bindings.0[self.0].0.downcast_ref::< [< Binding $n Node >] <$( [< S $i >] ),+ , T>>().unwrap();
                    $(
                        let [< value_ $i >] ;
                        if let Some(source) = node. [< source_ $i >] .as_ref() {
                            [< value_ $i >] = source.value.clone();
                        } else {
                            return None;
                        }
                    )+
                    Some((node.map)($( [< value_ $i >] ),+))
                }

                fn knoke_handler(self, state: &mut dyn State) {
                    let bindings: &Bindings = state.get();
                    if let Some(value) = self.get_value(bindings) {
                        let node = bindings.0[self.0].0.downcast_ref::< [< Binding $n Node >] <$( [< S $i >] ),+ , T>>().unwrap();
                        if let Some(handler) = node.handler.clone() {
                            unsafe { (handler.execute)(state, handler.target, value); }
                        }
                    }
                }
            }

            impl<
                $( [< S $i >] : Convenient, )+
                T: Convenient
            > Binding for [< Binding $n >] <$( [< S $i >] , )+ T> {
                type Value = T;

                fn handle(
                    &self,
                    state: &mut dyn State,
                    target: Global,
                    execute: unsafe fn(state: &mut dyn State, target: Global, value: T),
                ) {
                    let handler = GlobalHandler { target, execute };
                    let bindings: &mut Bindings = state.get_mut();
                    let node = bindings.0[self.0].0.downcast_mut::< [< Binding $n Node >] <$( [< S $i >] ),+ , T>>().unwrap();
                    node.handler = Some(handler.clone());
                    if let Some(value) = self.get_value(bindings) {
                        unsafe { (handler.execute)(state, handler.target, value); }
                    }
                }

                fn stop(&self, state: &mut dyn State) {
                    let bindings: &mut Bindings = state.get_mut();
                    let node = bindings.0.remove(self.0);
                    let mut node = node.0.downcast::< [< Binding $n Node >] <$( [< S $i >] ),+ , T>>().unwrap();
                    $(
                        if let Some(source) = node. [< source_ $i >] .take() {
                            unsafe { (source.unhandle)(state, source.source, source.handler_id); }
                        }
                    )+
                }
            }

            $(
                fn [< execute_binding_ $n _source_ $i >] <
                    $( [< S $j >] : Convenient, )+
                    T: Convenient
                >(
                    state: &mut dyn State,
                    target_id: RawId,
                    value: [< S $i >]
                ) {
                    let binding: [< Binding $n >] <$( [< S $j >] ),+ , T> = [< Binding $n >] ::from_raw(target_id);
                    let bindings: &mut Bindings = state.get_mut();
                    let node = bindings.0[binding.0].0.downcast_mut::< [< Binding $n Node >] <$( [< S $j >] ),+ , T>>().unwrap();
                    node. [< source_ $i >] .as_mut().unwrap().value = value.clone();
                    $(
                        let [< value_ $j >] ;
                        if let Some(source) = node. [< source_ $j >] .as_ref() {
                            [< value_ $j >] = source.value.clone();
                        } else {
                            return;
                        }
                    )+
                    if let Some(handler) = node.handler.clone() {
                        let value = (node.map)($( [< value_ $j >] ),+);
                        unsafe { (handler.execute)(state, handler.target, value); }
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

trait AnyBindingNode: Debug + Downcast { }

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
    pub fn new() -> Self { Bindings(Arena::new()) }
}

pub trait Source {
    type Value: Convenient;

    fn handle(&mut self, handler: Handler<Self::Value>) -> (HandledSource<Self::Value>, &mut dyn State);
}

pub trait Binding: Debug + DynClone + Send + Sync {
    type Value: Convenient;

    fn handle(
        &self,
        state: &mut dyn State,
        target: Global,
        execute: unsafe fn(state: &mut dyn State, target: Global, value: Self::Value),
    );

    fn stop(&self, state: &mut dyn State);
}

clone_trait_object!(<Value: Convenient> Binding<Value=Value>);
