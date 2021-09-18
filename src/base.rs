use crate::view::{Layout, View, ViewAlign, ViewBase, ViewInput, ViewTree, Decorator};
use components_arena::{Arena, Component, Id, NewtypeComponentId};
use debug_panic::debug_panic;
use dep_obj::{Change, DepObjId, DepType, dep_obj, dep_type, Convenient, DepProp, DepObjBaseBuilder};
use dep_obj::binding::{Binding1, Bindings, BYield, Binding};
use downcast_rs::{Downcast, impl_downcast};
use dyn_context::state::{RequiresStateDrop, State, StateDrop, StateExt};
use macro_attr_2018::macro_attr;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use tuifw_screen_base::Screen;

pub trait WidgetBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State);
    fn drop_bindings(&self, widget: Widget, state: &mut dyn State);
}

pub trait WidgetObj: Downcast + DepType<Id=Widget> + Send + Sync {
    fn behavior(&self) -> &'static dyn WidgetBehavior;
}

impl_downcast!(WidgetObj);

macro_attr! {
    #[derive(NewtypeComponentId!)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Widget(Id<WidgetNode>);
}

macro_attr! {
    #[derive(Debug, Component!)]
    struct WidgetNode {
        view: Option<View>,
        base: WidgetBase,
        obj: Box<dyn WidgetObj>,
    }
}

pub struct WidgetTree(StateDrop<WidgetTreeImpl>);

struct WidgetTreeImpl {
    widget_arena: Arena<WidgetNode>,
    view_tree: ViewTree,
}

impl RequiresStateDrop for WidgetTreeImpl {
    fn get(state: &dyn State) -> &StateDrop<Self> {
        let tree: &WidgetTree = state.get();
        &tree.0
    }

    fn get_mut(state: &mut dyn State) -> &mut StateDrop<Self> {
        let tree: &mut WidgetTree = state.get_mut();
        &mut tree.0
    }

    fn before_drop(state: &mut dyn State) {
        let tree: &WidgetTree = state.get();
        let widgets = tree.0.get().widget_arena.items().ids().collect::<Vec<_>>();
        for widget in widgets {
            Widget(widget).drop_bindings(state);
        }
        ViewTree::drop_self(state);
    }

    fn drop_incorrectly(self) {
        debug_panic!("WidgetTree should be dropped with the drop_self method");
    }
}

impl State for WidgetTree {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<WidgetTree>() { return Some(self); }
        self.0.get().view_tree.get_raw(ty)
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<WidgetTree>() { return Some(self); }
        self.0.get_mut().view_tree.get_mut_raw(ty)
    }
}

impl WidgetTree {
    pub fn new(screen: Box<dyn Screen>, bindings: &mut Bindings) -> Self {
        let widget_arena = Arena::new();
        let view_tree = ViewTree::new(screen, bindings, |_| ((), |view_tree| view_tree));
        WidgetTree(StateDrop::new(WidgetTreeImpl {
            widget_arena,
            view_tree,
        }))
    }

    pub fn drop_self(state: &mut dyn State) {
        <StateDrop<WidgetTreeImpl>>::drop_self(state);
    }

    pub fn root(&self) -> View {
        self.0.get().view_tree.root()
    }

    pub fn quit(state: &mut dyn State) {
        ViewTree::quit(state);
    }

    pub fn update(state: &mut dyn State, wait: bool) -> Result<bool, Box<dyn Any>> {
        ViewTree::update(state, wait)
    }
}

impl Widget {
    pub fn new<O: WidgetObj>(
        state: &mut dyn State,
        obj: O,
    ) -> Widget {
        let behavior = obj.behavior();
        let widget = {
            let tree: &mut WidgetTree = state.get_mut();
            tree.0.get_mut().widget_arena.insert(|widget| (WidgetNode {
                view: None,
                base: WidgetBase::new_priv(),
                obj: Box::new(obj),
            }, Widget(widget)))
        };
        let drop_old_view = Binding1::new(state, (), |(), change: Option<Change<Option<View>>>|
            change.and_then(|change| change.old)
        );
        drop_old_view.set_target_fn(state, (), |state, (), view: View| view.drop_view(state));
        drop_old_view.set_source_1(state, &mut WidgetBase::VIEW.change_final_source(widget.base()));
        widget.base().add_binding(state, drop_old_view);
        behavior.init_bindings(widget, state);
        widget
    }

    pub fn drop_widget(self, state: &mut dyn State) {
        self.drop_bindings(state);
        {
            let tree: &mut WidgetTree = state.get_mut();
            let node = tree.0.get_mut().widget_arena.remove(self.0);
            assert!(node.view.is_none(), "Loaded widget dropped");
        }
    }

    fn drop_bindings(self, state: &mut dyn State) {
        let tree: &WidgetTree = state.get();
        let behavior = tree.0.get().widget_arena[self.0].obj.behavior();
        behavior.drop_bindings(self, state);
        self.drop_bindings_priv(state);
    }

    pub fn load<X: Convenient>(self, state: &mut dyn State, parent: View, init: impl FnOnce(&mut dyn State, View)) -> BYield<X> {
        let view = View::new(state, parent, |view| (self, view));
        {
            let tree: &mut WidgetTree = state.get_mut();
            assert!(tree.0.get_mut().widget_arena[self.0].view.replace(view).is_none(), "Widget already loaded");
        }
        init(state, view);

        let input_binding = Binding1::new(state, (), |(), input: Option<ViewInput>| input);
        input_binding.dispatch(state, self, |state, widget, input|
            WidgetBase::VIEW_INPUT.raise(state, widget.base(), input)
        );
        input_binding.set_source_1(state, &mut ViewBase::INPUT.source(view.base()));

        WidgetBase::VIEW.set(state, self.base(), Some(view))
    }

    pub fn unload<X: Convenient>(self, state: &mut dyn State) -> BYield<X> {
        {
            let tree: &mut WidgetTree = state.get_mut();
            assert!(tree.0.get_mut().widget_arena[self.0].view.take().is_some(), "Widget is not loaded");
        }
        WidgetBase::VIEW.set(state, self.base(), None)
    }

    fn view(self, tree: &WidgetTree) -> Option<View> {
        tree.0.get().widget_arena[self.0].view
    }

    pub fn focus(self, state: &mut dyn State) {
        let tree: &WidgetTree = state.get();
        self.view(tree).map(|view| view.focus(state));
    }

    dep_obj! {
        pub fn base(self as this, tree: WidgetTree) -> (WidgetBase) {
            if mut {
                &mut tree.0.get_mut().widget_arena[this.0].base
            } else {
                &tree.0.get().widget_arena[this.0].base
            }
        }

        pub fn obj(self as this, tree: WidgetTree) -> (trait WidgetObj) {
            if mut {
                tree.0.get_mut().widget_arena[this.0].obj.as_mut()
            } else {
                tree.0.get().widget_arena[this.0].obj.as_ref()
            }
        }
    }

    fn bind_view<O: WidgetObj, P: Clone + 'static, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U,
        param: P,
        bind: fn(&mut dyn State, P, Binding<U>)
    ) {
        let binding = Binding1::new(state, map, |map, value: T| Some(map(value)));
        bind(state, param, binding.into());
        binding.set_source_1(state, &mut widget_prop.value_source(self.obj()));
    }
}

impl DepObjId for Widget { }

pub struct WidgetBuilder<'a> {
    pub widget: Widget,
    pub state: &'a mut dyn State,
}

impl<'a> DepObjBaseBuilder<Widget> for WidgetBuilder<'a> {
    fn id(&self) -> Widget { self.widget }
    fn state(&self) -> &dyn State { self.state }
    fn state_mut(&mut self) -> &mut dyn State { self.state }
}

pub trait ViewWidgetExt {
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
    fn bind_base_to_widget<O: WidgetObj, T: Convenient, U: Convenient>(
        self,
        state: &mut dyn State,
        view_base_prop: DepProp<ViewBase, U>,
        widget: Widget,
        widget_prop: DepProp<O, T>,
        map: fn(T) -> U
    ) {
        widget.bind_view(state, widget_prop, map, (view_base_prop, self), |state, (view_base_prop, view), binding|
            view_base_prop.bind(state, view.base(), binding)
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
        widget.bind_view(state, widget_prop, map, (view_align_prop, self), |state, (view_align_prop, view), binding|
            view_align_prop.bind(state, view.align(), binding)
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
        widget.bind_view(state, widget_prop, map, (layout_prop, self), |state, (layout_prop, view), binding|
            layout_prop.bind(state, view.layout(), binding)
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
        widget.bind_view(state, widget_prop, map, (decorator_prop, self), |state, (decorator_prop, view), binding|
            decorator_prop.bind(state, view.decorator(), binding)
        );
    }
}

dep_type! {
    #[derive(Debug)]
    pub struct WidgetBase in Widget {
        view: Option<View> = None,
        view_input yield ViewInput,
    }
}
