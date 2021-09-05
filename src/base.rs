use crate::view::{View, ViewTree, ViewBuilder, PanelTemplate, RootDecorator};
use components_arena::{Arena, Component, Id, NewtypeComponentId};
use dep_obj::{DepObjBaseBuilder, DepObjIdBase, DepType, Items, Style, dep_type, dep_obj};
use dep_obj::binding::{AnyBinding, Binding1, Bindings, EventBinding0};
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::state::{State, StateExt, StateRefMut};
use macro_attr_2018::macro_attr;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::iter::{self};
use std::mem::replace;
use tuifw_screen_base::Screen;

pub trait WidgetBehavior {
    fn load(&self, state: &mut dyn State, widget: Widget, view: View);
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
    #[derive(NewtypeComponentId!)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Model(Id<ModelNode>);
}

macro_attr! {
    #[derive(Debug, Component!)]
    struct WidgetNode {
        view: Option<View>,
        parent: Option<Widget>,
        last_child: Option<Widget>,
        next: Widget,
        obj: Box<dyn WidgetObj>,
        attached: bool,
    }
}

macro_attr! {
    #[derive(Debug, Component!)]
    struct ModelNode {
    }
}

pub struct WidgetTree {
    widget_arena: Arena<WidgetNode>,
    _model_arena: Arena<ModelNode>,
    view_tree: ViewTree,
    root: Widget,
    inserted_children: Option<AnyBinding>,
}

impl State for WidgetTree {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<WidgetTree>() {
            Some(self)
        } else if ty == TypeId::of::<ViewTree>() {
            Some(&self.view_tree)
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<WidgetTree>() {
            Some(self)
        } else if ty == TypeId::of::<ViewTree>() {
            Some(&mut self.view_tree)
        } else {
            None
        }
    }
}

impl WidgetTree {
    pub fn new(screen: Box<dyn Screen>, bindings: &mut Bindings) -> Self {
        let mut widget_arena = Arena::new();
        let model_arena = Arena::new();
        let (view_tree, root, root_view) = ViewTree::new(screen, bindings, |view| {
            let root = widget_arena.insert(|root| (WidgetNode {
                view: Some(view),
                parent: None,
                last_child: None,
                next: Widget(root),
                obj: Box::new(Root::new_priv()),
                attached: true,
            }, root));
            (root, move |view_tree| (view_tree, Widget(root), view))
        });
        let mut tree = WidgetTree {
            widget_arena,
            _model_arena: model_arena,
            view_tree,
            root,
            inserted_children: None,
        };
        (&mut tree).merge_mut_and_then(|state| {
            let decorator_style = Binding1::new(state, (), |(), (_, x)| Some(x));
            root.obj::<Root>().add_binding(state, decorator_style.into());
            decorator_style.set_source_1(state, &mut Root::DECORATOR_STYLE.source(root.obj()));
            decorator_style.set_target_fn(state, root_view, |state, root_view, decorator_style| {
                root_view.decorator().apply_style(state, decorator_style);
            });
            let children_changed = EventBinding0::new(state, root_view, |state, root_view, _: &mut ()| {
                root_view.invalidate_measure(state);
                Some(())
            });
            root.obj::<Root>().add_binding(state, children_changed.into());
            children_changed.set_event_source(state, &mut Root::CHILDREN.changed_source(root.obj()));
            let panel_template = Binding1::new(state, (), |(), (_, x)| Some(x));
            root.obj::<Root>().add_binding(state, panel_template.into());
            panel_template.set_source_1(state, &mut Root::PANEL_TEMPLATE.source(root.obj()));
            panel_template.set_target_fn(state, root_view, |state, root_view, panel_template| {
                panel_template.map(|x| x.apply_panel(state, root_view));
            });
            let inserted_children = Binding1::new(state, (), |(), (_, x)| Some(x));
            root.obj::<Root>().add_binding(state, inserted_children.into());
            inserted_children.set_source_1(state, &mut Root::PANEL_TEMPLATE.source(root.obj()));
            inserted_children.set_target_fn(state, (root, root_view), |state, (root, root_view), panel_template| {
                let inserted_children = EventBinding0::new(
                    state, (root, root_view, panel_template),
                    |
                        state,
                        (root, root_view, panel_template),
                        args: &mut Items<Widget>
                    | {
                        for &child in args.iter() {
                            child.attach(state, root);
                            let view = View::new(state, root_view, |view| (child, view));
                            panel_template.as_ref().map(|x| x.apply_layout(state, view));
                            child.load(state, view);
                        }
                        Some(())
                    }
                );
                {
                    let tree: &mut WidgetTree = state.get_mut();
                    tree.inserted_children.replace(inserted_children.into()).map(|x| x.drop_binding(state));
                }
                inserted_children.set_event_source(state, &mut Root::CHILDREN.inserted_items_source(root.obj()));
            });
            let removed_children = EventBinding0::new(state, (), |state, (), args: &mut Items<Widget>| {
                for &child in args.iter() {
                    child.detach(state);
                }
                Some(())
            });
            root.obj::<Root>().add_binding(state, removed_children.into());
            removed_children.set_event_source(state, &mut Root::CHILDREN.removed_items_source(root.obj()));
        }, bindings);
        tree
    }

    pub fn root(&self) -> Widget { self.root }

    pub fn update(state: &mut dyn State, wait: bool) -> Result<bool, Box<dyn Any>> {
        ViewTree::update(state, wait)
    }
}

impl Widget {
    pub fn new<O: WidgetObj>(
        state: &mut dyn State,
        obj: O,
    ) -> Widget {
        let tree: &mut WidgetTree = state.get_mut();
        tree.widget_arena.insert(|widget| (WidgetNode {
            view: None,
            parent: None,
            last_child: None,
            next: Widget(widget),
            obj: Box::new(obj),
            attached: false,
        }, Widget(widget)))
    }

    pub fn drop_widget(self, state: &mut dyn State) {
        let tree: &WidgetTree = state.get();
        if self.parent(tree).is_some() {
            self.detach(state);
        }
        {
            let tree: &mut WidgetTree = state.get_mut();
            tree.widget_arena.remove(self.0);
        }
    }

    pub fn attach(self, state: &mut dyn State, parent: Widget) {
        let attached = {
            let tree: &mut WidgetTree = state.get_mut();
            let node = &mut tree.widget_arena[self.0];
            if let Some(parent) = node.parent.replace(parent) {
                node.parent = Some(parent);
                panic!("widget already attached");
            }
            if let Some(prev) = tree.widget_arena[parent.0].last_child.replace(self) {
                let next = replace(&mut tree.widget_arena[prev.0].next, self);
                tree.widget_arena[self.0].next = next;
            }
            tree.widget_arena[parent.0].attached
        };
        if attached {
            self.set_attached(state, true);
        }
    }

    pub fn detach(self, state: &mut dyn State) {
        let view = {
            let tree: &mut WidgetTree = state.get_mut();
            let node = &mut tree.widget_arena[self.0];
            node.view.take()
        };
        if let Some(view) = view {
            view.drop_view(state);
        }
        let attached = {
            let tree: &mut WidgetTree = state.get_mut();
            if let Some(parent) = tree.widget_arena[self.0].parent.take() {
                let last_child = tree.widget_arena[parent.0].last_child.unwrap();
                let mut child = last_child;
                loop {
                    child = child.next(tree);
                    if child.next(tree) == self {
                        tree.widget_arena[child.0].next = replace(&mut tree.widget_arena[self.0].next, self);
                        break;
                    }
                    assert_ne!(child, last_child);
                }
                tree.widget_arena[self.0].attached
            } else {
                panic!("widget already detached");
            }
        };
        if attached {
            self.set_attached(state, false);
        }
    }

    //fn view(self, tree: &WidgetTree) -> Option<View> { tree.widget_arena[self.0].view }

    pub fn parent(self, tree: &WidgetTree) -> Option<Widget> { tree.widget_arena[self.0].parent }

    pub fn self_and_parents<'a>(self, tree: &'a WidgetTree) -> impl Iterator<Item=Widget> + 'a {
        let mut widget = Some(self);
        iter::from_fn(move || {
            let parent = widget.and_then(|view| view.parent(tree));
            replace(&mut widget, parent)
        })
    }

    pub fn last_child(self, tree: &WidgetTree) -> Option<Widget> { tree.widget_arena[self.0].last_child }

    pub fn next(self, tree: &WidgetTree) -> Widget { tree.widget_arena[self.0].next }

    pub fn children<'a>(self, tree: &'a WidgetTree) -> impl Iterator<Item=Widget> + 'a {
        let last_child = self.last_child(tree);
        let mut widget = last_child;
        iter::from_fn(move || {
            let item = widget.map(|widget| widget.next(tree));
            widget = if item == last_child { None } else { item };
            item
        })
    }

    fn set_attached(self, state: &mut dyn State, attached: bool) {
        let last_child = {
            let tree: &mut WidgetTree = state.get_mut();
            let node = &mut tree.widget_arena[self.0];
            node.attached = attached;
            node.last_child
        };
        if let Some(last_child) = last_child {
            let mut child = last_child;
            loop {
                let tree: &WidgetTree = state.get();
                child = child.next(tree);
                child.set_attached(state, attached);
                if child == last_child { break; }
            }
        }
    }

    pub fn load(self, state: &mut dyn State, view: View) {
        let tree: &WidgetTree = state.get();
        if view.tag::<Widget>(&tree.view_tree) != self {
            panic!("view/widget tag mismatch");
        }
        let behavior = {
            let tree: &mut WidgetTree = state.get_mut();
            let node = &mut tree.widget_arena[self.0];
            if !node.attached {
                panic!("detached widget cannot be loaded");
            }
            if let Some(view) = node.view.replace(view) {
                node.view = Some(view);
                panic!("widget already loaded");
            }
            node.obj.behavior()
        };
        behavior.load(state, self, view);
    }

    dep_obj! {
        pub fn obj(self as this, tree: WidgetTree) -> (trait WidgetObj) {
            if mut {
                tree.widget_arena[this.0].obj.as_mut()
            } else {
                tree.widget_arena[this.0].obj.as_ref()
            }
        }
    }
}

impl DepObjIdBase for Widget {
    fn parent(self, state: &dyn State) -> Option<Self> {
        let tree: &WidgetTree = state.get();
        self.parent(tree)
    }

    fn next(self, state: &dyn State) -> Self {
        let tree: &WidgetTree = state.get();
        self.next(tree)
    }

    fn last_child(self, state: &dyn State) -> Option<Self> {
        let tree: &WidgetTree = state.get();
        self.last_child(tree)
    }
}

pub trait ViewBuilderWidgetExt {
    fn widget(self, widget: Widget) -> Self;
}

impl<'a> ViewBuilderWidgetExt for ViewBuilder<'a> {
    fn widget(mut self, widget: Widget) -> Self {
        let view = self.id();
        widget.load(self.state_mut(), view);
        self
    }
}

pub trait WidgetTemplate: Debug + DynClone + Send + Sync {
    fn load(&self, state: &mut dyn State) -> Widget;
}

clone_trait_object!(WidgetTemplate);

dep_type! {
    #[derive(Debug)]
    pub struct Root in Widget {
        panel_template: Option<Box<dyn PanelTemplate>> = None,
        decorator_style: Option<Style<RootDecorator>> = None,
        children [Widget],
    }
}

struct RootBehavior;

impl WidgetBehavior for RootBehavior {
    fn load(&self, _state: &mut dyn State, _widget: Widget, _view: View) {
        panic!("root widget always loaded");
    }
}

impl Root {
    const BEHAVIOR: &'static dyn WidgetBehavior = &RootBehavior;
}

impl WidgetObj for Root {
    fn behavior(&self) -> &'static dyn WidgetBehavior { Root::BEHAVIOR }
}
