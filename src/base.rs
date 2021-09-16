use crate::view::{View, ViewTree};
use components_arena::{Arena, Component, Id, NewtypeComponentId};
use debug_panic::debug_panic;
use dep_obj::{Change, DepObjId, DepType, dep_obj, dep_type, Convenient};
use dep_obj::binding::{Binding1, Bindings, BYield};
use downcast_rs::{Downcast, impl_downcast};
use dyn_context::state::{RequiresStateDrop, State, StateDrop, StateExt};
use macro_attr_2018::macro_attr;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::mem::replace;
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
        loaded: bool,
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
                loaded: false,
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
            assert!(!node.loaded, "Loaded widget dropped");
        }
    }

    fn drop_bindings(self, state: &mut dyn State) {
        let tree: &WidgetTree = state.get();
        let behavior = tree.0.get().widget_arena[self.0].obj.behavior();
        behavior.drop_bindings(self, state);
        self.drop_bindings_priv(state);
    }

    pub fn load<X: Convenient>(self, state: &mut dyn State, parent: View, init: impl FnOnce(&mut dyn State, View)) -> BYield<X> {
        {
            let tree: &mut WidgetTree = state.get_mut();
            assert!(!replace(&mut tree.0.get_mut().widget_arena[self.0].loaded, true), "Widget already loaded");
        }
        let view = View::new(state, parent, |view| (self, view));
        init(state, view);
        WidgetBase::VIEW.set(state, self.base(), Some(view))
    }

    pub fn unload<X: Convenient>(self, state: &mut dyn State) -> BYield<X> {
        {
            let tree: &mut WidgetTree = state.get_mut();
            assert!(replace(&mut tree.0.get_mut().widget_arena[self.0].loaded, false), "Widget is not loaded");
        }
        WidgetBase::VIEW.set(state, self.base(), None)
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
}

impl DepObjId for Widget { }

dep_type! {
    #[derive(Debug)]
    pub struct WidgetBase in Widget {
        view: Option<View> = None,
    }
}
