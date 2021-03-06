use components_arena::{ComponentId, Id, Component, Arena, ComponentClassMutex};
use core::hint::unreachable_unchecked;
use crate::view::{View, ViewTree, ViewBuilder, RootDecorator, PanelTemplate};
use dep_obj::{dep_type, dep_obj, Style, DepObjBuilderCore};
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dyn_context::{Context, ContextExt};
use macro_attr_2018::macro_attr;
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::iter::{self};
use std::mem::replace;
use tuifw_screen_base::Screen;

pub trait WidgetBehavior {
    fn load(&self, tree: &mut WidgetTree, widget: Widget, view: View);
}

pub trait WidgetObj: Downcast + Debug + Send + Sync {
    fn behavior(&self) -> &'static dyn WidgetBehavior;
}

impl_downcast!(WidgetObj);

macro_attr! {
    #[derive(ComponentId!)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Widget(Id<WidgetNode>);
}

macro_attr! {
    #[derive(ComponentId!)]
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
        obj: Option<Box<dyn WidgetObj>>,
        attached: bool,
    }
}

static WIDGET_NODE: ComponentClassMutex<WidgetNode> = ComponentClassMutex::new();

macro_attr! {
    #[derive(Debug, Component!)]
    struct ModelNode {
    }
}

static MODEL_NODE: ComponentClassMutex<ModelNode> = ComponentClassMutex::new();

pub struct WidgetTree {
    widget_arena: Arena<WidgetNode>,
    _model_arena: Arena<ModelNode>,
    view_tree: ViewTree,
    _root: Widget,
}

impl Context for WidgetTree {
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
    pub fn new(screen: Box<dyn Screen>) -> Self {
        let mut widget_arena = Arena::new(&mut WIDGET_NODE.lock().unwrap());
        let model_arena = Arena::new(&mut MODEL_NODE.lock().unwrap());
        let (view_tree, root) = ViewTree::new(screen, |view| {
            let root = widget_arena.insert(|root| (WidgetNode {
                view: Some(view),
                parent: None,
                last_child: None,
                next: Widget(root),
                obj: Some(Box::new(Root::new_priv())),
                attached: true,
            }, root));
            (root, move |view_tree| (view_tree, Widget(root)))
        });
        let mut tree = WidgetTree {
            widget_arena,
            _model_arena: model_arena,
            view_tree,
            _root: root,
        };
        root.obj(&mut tree).on_changed(Root::PANEL_TEMPLATE, |context, root, _old| {
            let tree: &WidgetTree = context.get();
            let root_view = tree.widget_arena[root.0].view.unwrap_or_else(|| unsafe { unreachable_unchecked() });
            let new = root.obj_ref(tree).get(Root::PANEL_TEMPLATE).clone();
            new.map(|x| x.apply_panel(context, root_view));
        });
        root.obj(&mut tree).on_changed(Root::DECORATOR_STYLE, |context, root, _old| {
            let tree: &WidgetTree = context.get();
            let root_view = tree.widget_arena[root.0].view.unwrap_or_else(|| unsafe { unreachable_unchecked() });
            let decorator_style = root.obj_ref(tree).get(Root::DECORATOR_STYLE).clone();
            root_view.decorator_mut(context).apply_style(decorator_style);
        });
        /*
        root.obj(&mut tree).on_vec_changed(Root::CHILDREN, |context, root, change| {
            match change {
                DepVecChange::Reset => {
                    if let Some(last_child) = 
                },
                _ => {}
            }
        });
        */
        tree
    }

    //pub fn update(context: &mut dyn Context, wait: bool) -> Result<bool, Box<dyn Any>> {
    //}
}

impl Widget {
    pub fn new<O: WidgetObj>(
        tree: &mut WidgetTree, 
        obj: O,
    ) -> Widget {
        tree.widget_arena.insert(|widget| (WidgetNode {
            view: None,
            parent: None,
            last_child: None,
            next: Widget(widget),
            obj: Some(Box::new(obj)),
            attached: false,
        }, Widget(widget)))
    }

    pub fn drop(self, tree: &mut WidgetTree) {
        if self.parent(tree).is_some() {
            self.detach(tree);
        }
        tree.widget_arena.remove(self.0);
    }

    pub fn attach(self, tree: &mut WidgetTree, parent: Widget) {
        let node = &mut tree.widget_arena[self.0];
        if let Some(parent) = node.parent.replace(parent) {
            node.parent = Some(parent);
            panic!("widget already attached");
        }
        if let Some(prev) = tree.widget_arena[parent.0].last_child.replace(self) {
            let next = replace(&mut tree.widget_arena[prev.0].next, self);
            tree.widget_arena[self.0].next = next;
        }
        if tree.widget_arena[parent.0].attached {
            self.set_attached(tree, true);
        }
    }

    pub fn detach(self, tree: &mut WidgetTree) {
        let node = &mut tree.widget_arena[self.0];
        if let Some(view) = node.view.take() {
            view.drop(&mut tree.view_tree);
        }
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
            if tree.widget_arena[self.0].attached {
                self.set_attached(tree, false);
            }
        } else {
            panic!("widget already detached");
        }
    }

    pub fn parent(self, tree: &WidgetTree) -> Option<Widget> { tree.widget_arena[self.0].parent }

    pub fn self_and_parents<'a>(self, tree: &'a WidgetTree) -> impl Iterator<Item=Widget> + 'a {
        let mut view = Some(self);
        iter::from_fn(move || {
            let parent = view.and_then(|view| view.parent(tree));
            replace(&mut view, parent)
        })
    }

    pub fn last_child(self, tree: &WidgetTree) -> Option<Widget> { tree.widget_arena[self.0].last_child }

    pub fn next(self, tree: &WidgetTree) -> Widget { tree.widget_arena[self.0].next }

    pub fn children<'a>(self, tree: &'a WidgetTree) -> impl Iterator<Item=Widget> + 'a {
        let last_child = self.last_child(tree);
        let mut view = last_child;
        iter::from_fn(move || {
            let item = view.map(|view| view.next(tree));
            view = if item == last_child { None } else { item };
            item
        })
    }

    fn set_attached(self, tree: &mut WidgetTree, attached: bool) {
        let node = &mut tree.widget_arena[self.0];
        node.attached = attached;
        if let Some(last_child) = node.last_child {
            let mut child = last_child;
            loop {
                child = child.next(tree);
                child.set_attached(tree, attached);
                if child == last_child { break; }
            }
        }
    }

    pub fn load(self, tree: &mut WidgetTree, view: View) {
        if view.tag::<Widget>(&tree.view_tree) != self {
            panic!("view/widget tag mismatch");
        }
        let node = &mut tree.widget_arena[self.0];
        if !node.attached {
            panic!("detached widget cannot be loaded");
        }
        if let Some(view) = node.view.replace(view) {
            node.view = Some(view);
            panic!("widget already loaded");
        }
        let behavior = node.obj.as_ref().unwrap().behavior();
        behavior.load(tree, self, view);
    }

    dep_obj! {
        pub dyn fn obj(self as this, tree: WidgetTree) -> WidgetObj {
            if mut {
                tree.widget_arena[this.0].obj.as_mut().expect("root widget does not have obj")
            } else {
                tree.widget_arena[this.0].obj.as_ref().expect("root widget does not have obj")
            }
        }
    }
}

pub trait ViewBuilderWidgetExt {
    fn widget(self, widget: Widget) -> Self;
}

impl<'a> ViewBuilderWidgetExt for ViewBuilder<'a> {
    fn widget(mut self, widget: Widget) -> Self {
        let view = self.id();
        let tree: &mut WidgetTree = self.context_mut().get_mut();
        widget.load(tree, view);
        self
    }
}

pub trait WidgetTemplate: Debug + DynClone + Send + Sync {
    fn load(&self, context: &mut dyn Context) -> Widget;
}

clone_trait_object!(WidgetTemplate);

dep_type! {
    #[derive(Debug)]
    pub struct Root become obj in Widget {
        panel_template: Option<Box<dyn PanelTemplate>> = None,
        decorator_style: Option<Style<RootDecorator>> = None,
        children [Widget],
    }
}

struct RootBehavior;

impl WidgetBehavior for RootBehavior {
    fn load(&self, _tree: &mut WidgetTree, _widget: Widget, _view: View) {
        panic!("root widget always loaded");
    }
}

impl Root {
    const BEHAVIOR: &'static dyn WidgetBehavior = &RootBehavior;
}

impl WidgetObj for Root {
    fn behavior(&self) -> &'static dyn WidgetBehavior { Root::BEHAVIOR }
}
