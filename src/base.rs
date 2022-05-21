use crate::view::{Layout, View, ViewAlign, ViewBase, ViewInput, ViewTree, Decorator};
use alloc::boxed::Box;
use components_arena::{Arena, Component, ComponentStop, Id, NewtypeComponentId};
use components_arena::with_arena_in_state_part;
use core::any::{Any, TypeId};
use core::fmt::Debug;
use dep_obj::{Change, DepObjId, DepType, DetachedDepObjId, Convenient, DepProp};
use dep_obj::{dep_obj, dep_type, with_builder};
use dep_obj::binding::{Binding1, Bindings, Re, Binding};
use downcast_rs::{Downcast, impl_downcast};
use dyn_context::{State, StateExt, Stop};
use errno_no_std::Errno;
use macro_attr_2018::macro_attr;
use tuifw_screen_base::Screen;

pub trait WidgetBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State);
    fn drop_bindings(&self, widget: Widget, state: &mut dyn State);
}

pub enum WidgetObjKey { }

pub trait WidgetObj: Downcast + DepType<Id=Widget, DepObjKey=WidgetObjKey> {
    fn behavior(&self) -> &'static dyn WidgetBehavior;
}

impl_downcast!(WidgetObj);

macro_attr! {
    #[derive(NewtypeComponentId!)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Widget(Id<WidgetNode>);
}

macro_attr! {
    #[derive(Debug, Component!(stop=WidgetStop))]
    struct WidgetNode {
        view: Option<View>,
        base: WidgetBase,
        obj: Box<dyn WidgetObj>,
    }
}

#[derive(Stop)]
pub struct WidgetTree {
    #[stop]
    widget_arena: Arena<WidgetNode>,
    #[stop]
    view_tree: ViewTree,
}

impl ComponentStop for WidgetStop {
    with_arena_in_state_part!(WidgetTree { .widget_arena });

    fn stop(&self, state: &mut dyn State, id: Id<WidgetNode>) {
        Widget(id).drop_bindings(state);
    }
}

impl State for WidgetTree {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<WidgetTree>() { return Some(self); }
        self.view_tree.get_raw(ty)
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<WidgetTree>() { return Some(self); }
        self.view_tree.get_mut_raw(ty)
    }
}

impl WidgetTree {
    pub fn new(screen: Box<dyn Screen>, bindings: &mut Bindings) -> Self {
        let widget_arena = Arena::new();
        let view_tree = ViewTree::new(screen, bindings);
        WidgetTree {
            widget_arena,
            view_tree,
        }
    }

    pub fn root(&self) -> View {
        self.view_tree.root()
    }

    pub fn quit(state: &mut dyn State) {
        ViewTree::quit(state);
    }

    pub fn update(state: &mut dyn State, wait: bool) -> Result<bool, Errno> {
        ViewTree::update(state, wait)
    }
}

dep_obj! {
    impl Widget {
        fn<WidgetBase>(self as this, tree: WidgetTree) -> (WidgetBase) {
            if mut {
                &mut tree.widget_arena[this.0].base
            } else {
                &tree.widget_arena[this.0].base
            }
        }

        fn<WidgetObjKey>(self as this, tree: WidgetTree) -> dyn(WidgetObj) {
            if mut {
                tree.widget_arena[this.0].obj.as_mut()
            } else {
                tree.widget_arena[this.0].obj.as_ref()
            }
        }
    }
}

impl Widget {
    with_builder!();

    pub fn new<O: WidgetObj>(
        state: &mut dyn State,
        obj: O,
    ) -> Widget {
        let behavior = obj.behavior();
        let widget = {
            let tree: &mut WidgetTree = state.get_mut();
            tree.widget_arena.insert(|widget| (WidgetNode {
                view: None,
                base: WidgetBase::new_priv(),
                obj: Box::new(obj),
            }, Widget(widget)))
        };
        let drop_old_view = Binding1::new(state, (), |(), change: Option<Change<Option<View>>>|
            change.and_then(|change| change.old)
        );
        drop_old_view.set_target_fn(state, (), |state, (), view: View| view.drop_view(state));
        widget.add_binding::<WidgetBase, _>(state, drop_old_view);
        drop_old_view.set_source_1(state, &mut WidgetBase::VIEW.change_final_source(widget));
        behavior.init_bindings(widget, state);
        widget
    }

    pub fn drop_widget(self, state: &mut dyn State) {
        self.drop_bindings(state);
        {
            let tree: &mut WidgetTree = state.get_mut();
            let node = tree.widget_arena.remove(self.0);
            assert!(node.view.is_none(), "Loaded widget dropped");
        }
    }

    fn drop_bindings(self, state: &mut dyn State) {
        let tree: &WidgetTree = state.get();
        let node = &tree.widget_arena[self.0];
        let behavior = node.obj.behavior();
        behavior.drop_bindings(self, state);
        self.drop_bindings_priv(state);
    }

    pub fn load<X: Convenient>(
        self,
        state: &mut dyn State,
        parent: View,
        prev: Option<View>,
        init: impl FnOnce(&mut dyn State, View),
    ) -> Re<X> {
        let view = View::new(state, parent, prev);
        view.set_tag(state, self);
        {
            let tree: &mut WidgetTree = state.get_mut();
            assert!(
                tree.widget_arena[self.0].view.replace(view).is_none(),
                "Widget already loaded"
            );
        }
        init(state, view);

        let input_binding = Binding1::new(state, (), |(), input: Option<ViewInput>| input);
        input_binding.dispatch(state, self, |state, widget, input|
            WidgetBase::VIEW_INPUT.raise(state, widget, input)
        );
        self.add_binding::<WidgetBase, _>(state, input_binding);
        input_binding.set_source_1(state, &mut ViewBase::INPUT.source(view));

        WidgetBase::VIEW.set(state, self, Some(view))
    }

    pub fn unload<X: Convenient>(self, state: &mut dyn State) -> Re<X> {
        {
            let tree: &mut WidgetTree = state.get_mut();
            assert!(
                tree.widget_arena[self.0].view.take().is_some(),
                "Widget is not loaded"
            );
        }
        WidgetBase::VIEW.set(state, self, None)
    }

    pub fn view(self, tree: &WidgetTree) -> Option<View> {
        tree.widget_arena[self.0].view
    }

    pub fn focus(self, state: &mut dyn State) {
        let tree: &WidgetTree = state.get();
        self.view(tree).map(|view| view.focus(state));
    }

    fn bind_view<O: WidgetObj, M: Clone + 'static, P: Clone + 'static, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        widget_prop: DepProp<O, T>,
        map_param: M,
        map: fn(M, T) -> U,
        param: P,
        bind: fn(&mut dyn State, P, Binding<U>)
    ) {
        let binding = Binding1::new(state, (map_param, map), |(map_param, map), value: T|
            Some(map(map_param, value))
        );
        bind(state, param, binding.into());
        binding.set_source_1(state, &mut widget_prop.value_source(self));
    }
}

impl DetachedDepObjId for Widget { }

pub trait ViewWidgetExt {
    fn bind_base_to_widget_option<O: WidgetObj, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        view_base_prop: DepProp<ViewBase, U>,
        widget: Widget,
        widget_prop: DepProp<O, Option<T>>,
        map: fn(T) -> U,
    );

    fn bind_base_to_widget<O: WidgetObj, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        view_base_prop: DepProp<ViewBase, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U,
    );

    fn bind_align_to_widget<O: WidgetObj, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        view_align_prop: DepProp<ViewAlign, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U,
    );

    fn bind_layout_to_widget<O: WidgetObj, L: Layout, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        layout_prop: DepProp<L, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U,
    );

    fn bind_decorator_to_widget<O: WidgetObj, D: Decorator, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        decorator_prop: DepProp<D, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U,
    );
}

impl ViewWidgetExt for View {
    fn bind_base_to_widget_option<O: WidgetObj, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        view_base_prop: DepProp<ViewBase, U>,
        widget: Widget,
        widget_prop: DepProp<O, Option<T>>,
        map: fn(T) -> U,
    ) {
        widget.bind_view(state, widget_prop, map, |map, x| x.map(map), (view_base_prop, self),
            |state, (view_base_prop, view), binding| {
                binding.dispatch(state, (view_base_prop, view), |state, (view_base_prop, view), value| {
                    if let Some(value) = value {
                        view_base_prop.set(state, view, value)
                    } else {
                        view_base_prop.unset(state, view)
                    }
                });
                view.add_binding::<ViewBase, _>(state, binding);
            }
        );
    }

    fn bind_base_to_widget<O: WidgetObj, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        view_base_prop: DepProp<ViewBase, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U
    ) {
        widget.bind_view(state, widget_prop, map, |map, x| map(x), (view_base_prop, self),
            |state, (view_base_prop, view), binding|
                view_base_prop.bind(state, view, binding)
        );
    }

    fn bind_align_to_widget<O: WidgetObj, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        view_align_prop: DepProp<ViewAlign, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U,
    ) {
        widget.bind_view(state, widget_prop, map, |map, x| map(x), (view_align_prop, self),
            |state, (view_align_prop, view), binding|
                view_align_prop.bind(state, view, binding)
        );
    }

    fn bind_layout_to_widget<O: WidgetObj, L: Layout, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        layout_prop: DepProp<L, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U,
    ) {
        widget.bind_view(state, widget_prop, map, |map, x| map(x), (layout_prop, self),
            |state, (layout_prop, view), binding|
                layout_prop.bind(state, view, binding)
        );
    }

    fn bind_decorator_to_widget<O: WidgetObj, D: Decorator, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        decorator_prop: DepProp<D, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U,
    ) {
        widget.bind_view(state, widget_prop, map, |map, x| map(x),
            (decorator_prop, self), |state, (decorator_prop, view), binding|
                decorator_prop.bind(state, view, binding)
        );
    }
}

dep_type! {
    #[derive(Debug)]
    pub struct WidgetBase = Widget[WidgetBase] {
        view: Option<View> = None,
        view_input yield ViewInput,
    }
}
