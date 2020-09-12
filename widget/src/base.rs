use components_arena::{Id, Component, Arena, ComponentClassMutex};
use macro_attr_2018::macro_attr;
use educe::Educe;
use tuifw_screen_base::Screen;
use crate::view::{View, ViewTree};

pub struct Widget(Id<WidgetNode>);

pub struct Model(Id<ModelNode>);

macro_attr! {
    #[derive(Educe, Component!)]
    #[educe(Debug)]
    struct WidgetNode {
        view: Option<View>,
        #[educe(Debug(ignore))]
        template: Option<Box<dyn Fn(&mut WidgetTree, Widget, View) -> View>>,
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
    model_arena: Arena<ModelNode>,
    view_tree: ViewTree,
    root: Widget,
}

impl WidgetTree {
    pub fn new(screen: Box<dyn Screen>) -> Self {
        let mut widget_arena = Arena::new(&mut WIDGET_NODE.lock().unwrap());
        let model_arena = Arena::new(&mut MODEL_NODE.lock().unwrap());
        let (view_tree, root) = ViewTree::new(screen, |view| {
            let root = widget_arena.insert(|root| (WidgetNode { view: Some(view), template: None }, root));
            (root, move |view_tree| (view_tree, Widget(root)))
        });
        WidgetTree {
            widget_arena,
            model_arena,
            view_tree,
            root,
        }
    }
}


