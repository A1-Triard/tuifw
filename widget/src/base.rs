use std::mem::replace;
use components_arena::{ComponentId, Id, Component, Arena, ComponentClassMutex};
use macro_attr_2018::macro_attr;
use educe::Educe;
use tuifw_screen_base::Screen;
use crate::view::{View, ViewTree};

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
    #[derive(Educe, Component!)]
    #[educe(Debug)]
    struct WidgetNode {
        view: Option<View>,
        parent: Option<Widget>,
        last_child: Option<Widget>,
        next: Widget,
        #[educe(Debug(ignore))]
        load: Option<fn(tree: &mut WidgetTree, widget: Widget, parent_view: View) -> View>,
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
                load: None,
            }, root));
            (root, move |view_tree| (view_tree, Widget(root)))
        });
        WidgetTree {
            widget_arena,
            _model_arena: model_arena,
            view_tree,
            _root: root,
        }
    }
}

impl Widget {
    pub fn new(
        tree: &mut WidgetTree, 
        load: fn(tree: &mut WidgetTree, widget: Widget, parent_view: View) -> View,
    ) -> Widget {
        tree.widget_arena.insert(|widget| (WidgetNode {
            view: None,
            parent: None,
            last_child: None,
            next: Widget(widget),
            load: Some(load),
        }, Widget(widget)))
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
    }

    fn parent(self, tree: &WidgetTree) -> Option<Widget> {
        tree.widget_arena[self.0].parent
    }

    pub fn load(self, tree: &mut WidgetTree, parent_view: View) -> View {
        let parent = self.parent(tree).expect("detached widget cannot be loaded");
        if parent_view.tag::<Widget>(&tree.view_tree) != parent {
            panic!("parent view/widget mismatch");
        }
        let node = &mut tree.widget_arena[self.0];
        if node.view.is_some() { panic!("widget already loaded"); }
        let load = node.load.unwrap();
        let view = load(tree, self, parent_view);
        if view.tag::<Widget>(&tree.view_tree) != self {
            panic!("widget/view tag mismatch");
        }
        let node = &mut tree.widget_arena[self.0];
        node.view = Some(view);
        view
    }
}
