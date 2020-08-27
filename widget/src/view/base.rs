use std::fmt::Debug;
use std::iter::{self};
use std::mem::{replace};
use boow::Bow;
use components_arena::{Id, Arena, ComponentClassMutex};
use downcast::Any;
use tuifw_screen_base::{Event, Screen, Vector, Point, Rect, Attr, Color};
use tuifw_window::{RenderPort, WindowTree, Window};
use crate::context::{Context, ContextRef, ContextMut};
use crate::property::Property;

pub trait Layout: Debug + Send + Sync {
    fn measure(&self, tree: &mut ViewTree, view: View, w: Option<i16>, h: Option<i16>) -> Vector;
    fn arrange(&self, tree: &mut ViewTree, view: View, size: Vector) -> Vector;
}

pub trait Render: Debug + Send + Sync {
    fn render(&self, tree: &ViewTree, view: View, port: &mut RenderPort);
}

pub trait ViewObj: Any + Debug + Sync + Send {
    fn client_bounds(&self, tree: &ViewTree, size: Vector) -> Rect;
}

downcast!(dyn ViewObj);

type RenderContext = ContextRef<ViewTree>;

macro_attr! {
    #[derive(Debug)]
    #[derive(Component!)]
    struct ViewNode {
        obj: Box<dyn ViewObj>,
        render: Option<(Box<dyn Render>, Window<View, RenderContext>, Vector)>,
        layout: Option<Box<dyn Layout>>,
        parent: Option<View>,
        next: View,
        last_child: Option<View>,
        measure_size: Option<(Option<i16>, Option<i16>)>,
        desired_size: Vector,
    }
}

static VIEW_NODE: ComponentClassMutex<ViewNode> = ComponentClassMutex::new();

#[derive(Debug)]
pub struct ViewTree {
    arena: Arena<ViewNode>,
    window_tree: Option<WindowTree<View, RenderContext>>,
    screen_size: Vector,
    root: View,
}

impl ViewTree {
    pub fn new(screen: Box<dyn Screen>) -> Self {
        let mut arena = Arena::new(&mut VIEW_NODE.lock().unwrap());
        let (window_tree, root) = arena.insert(|view| {
            let window_tree = WindowTree::new(screen, render_view, View(view));
            let mut root = RootView { view: View(view), bg: Property::new(Text::SPACE.clone()) };
            root.on_bg_changed(RootView::invalidate_bg);
            let screen_size = window_tree.screen_size();
            (ViewNode {
                obj: Box::new(root) as _,
                render: None,
                layout: None,
                parent: None,
                next: View(view),
                last_child: None,
                measure_size: Some((Some(screen_size.x), Some(screen_size.y))),
                desired_size: screen_size,
            }, (window_tree, View(view)))
        });
        let screen_size = window_tree.screen_size();
        ViewTree {
            arena,
            window_tree: Some(window_tree),
            screen_size,
            root
        }
    }

    fn window_tree(&mut self) -> &mut WindowTree<View, RenderContext> {
        self.window_tree.as_mut().expect("ViewTree is in invalid state")
    }

    pub fn root(&self) -> View { self.root }

    pub fn update(&mut self, wait: bool) -> Result<Option<Event>, Box<dyn std::any::Any>> {
        self.root.measure(self, Some(self.screen_size.x), Some(self.screen_size.y));
        let mut window_tree = self.window_tree.take().expect("ViewTree is in invalid state");
        let result = Context::with_ref(self, |render_context| window_tree.update(wait, render_context));
        if let Ok(result) = &result {
            if result == &Some(Event::Resize) {
                self.screen_size = window_tree.screen_size();
            }
        }
        self.window_tree.replace(window_tree);
        result
    }
}

fn render_view(
    _tree: &WindowTree<View, RenderContext>,
    _window: Option<Window<View, RenderContext>>,
    port: &mut RenderPort,
    tag: &View,
    context: &mut RenderContext
) {
    let view_tree = context.get_1();
    if *tag == view_tree.root {
        let root = view_tree.arena[tag.0].obj.downcast_ref::<RootView>();
        let bg = root.as_ref().unwrap().bg();
        port.fill(|port, p| port.out(p, bg.fg, bg.bg, bg.attr, &bg.value));
    } else {
        view_tree.arena[tag.0].render.as_ref().unwrap().0.render(view_tree, *tag, port);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct View(Id<ViewNode>);

impl View {
    pub fn new<T>(
        tree: &mut ViewTree,
        parent: View,
        render: Option<Box<dyn Render>>,
        obj: impl FnOnce(View) -> (Box<dyn ViewObj>, T)
    ) -> T {
        let render_and_parent_window = render.map(|render| (
            render,
            parent
                .self_and_parents(tree)
                .find_map(|view| tree.arena[view.0].render.as_ref().map(|x| x.1))
        ));
        let arena = &mut tree.arena;
        let window_tree = tree.window_tree.as_mut().expect("ViewTree is in invalid state");
        let (view, result) = arena.insert(|view| {
            let (obj, result) = obj(View(view));
            let render = render_and_parent_window.map(|(render, parent_window)| (render, Window::new(
                window_tree,
                parent_window,
                Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                |window| (View(view), window)
            ), Vector::null()));
            (ViewNode {
                obj,
                render,
                layout: None,
                parent: Some(parent),
                next: View(view),
                last_child: None,
                measure_size: None,
                desired_size: Vector::null(),
            }, (view, result))
        });
        View(view).invalidate_measure(tree);
        result
    }

    pub fn parent(self, tree: &ViewTree) -> Option<View> { tree.arena[self.0].parent }

    pub fn self_and_parents<'a>(self, tree: &'a ViewTree) -> impl Iterator<Item=View> + 'a {
        let mut view = Some(self);
        iter::from_fn(move || {
            let parent = view.and_then(|view| view.parent(tree));
            replace(&mut view, parent)
        })
    }

    pub fn obj(self, tree: &ViewTree) -> &dyn ViewObj { tree.arena[self.0].obj.as_ref() }

    pub fn obj_mut(self, tree: &mut ViewTree) -> &mut dyn ViewObj { tree.arena[self.0].obj.as_mut() }

    pub fn layout(self, tree: &ViewTree) -> Option<&dyn Layout> { tree.arena[self.0].layout.as_deref() }

    pub fn set_layout(self, tree: &mut ViewTree, value: Option<Box<dyn Layout>>) -> Option<Box<dyn Layout>> {
        replace(&mut tree.arena[self.0].layout, value)
    }

    pub fn size(self, tree: &ViewTree) -> Option<Vector> {
        if self == tree.root { return Some(tree.screen_size); }
        tree.arena[self.0].render.as_ref().map(|x| x.2)
    }

    #[must_use]
    pub fn invalidate_rect(self, tree: &mut ViewTree, rect: Rect) -> Option<()> {
        if self == tree.root { return Some(tree.window_tree().invalidate_rect(rect)); }
        let window = tree.arena[self.0].render.as_ref().map(|x| x.1);
        window.map(|window| window.invalidate_rect(&mut tree.window_tree(), rect))
    }

    #[must_use]
    pub fn invalidate_render(self, tree: &mut ViewTree) -> Option<()> {
        if self == tree.root { return Some(tree.window_tree().invalidate_screen()); }
        let window = tree.arena[self.0].render.as_ref().map(|x| x.1);
        window.map(|window| window.invalidate(&mut tree.window_tree()))
    }
    
    pub fn invalidate_measure(self, tree: &mut ViewTree) {
        let mut view = self;
        loop {
            tree.arena[view.0].measure_size = None;
            if let Some(parent) = view.parent(tree) {
                view = parent;
            } else {
                break;
            }
        }
    }

    pub fn measure(self, tree: &mut ViewTree, w: Option<i16>, h: Option<i16>) {
        let node = &mut tree.arena[self.0];
        if node.measure_size == Some((w, h)) { return; }
        node.measure_size = Some((w, h));
        let layout = node.layout.take();
        let desired_size = if let Some(layout) = &layout {
            layout.measure(tree, self, w, h)
        } else {
            Vector::null()
        };
        let node = &mut tree.arena[self.0];
        node.layout = layout;
        node.desired_size = desired_size;
    }
}

pub type ViewContext = ContextMut<ViewTree>;

#[derive(Debug)]
pub struct RootView {
    view: View,
    bg: Property<Self, Text, ViewContext>,
}

impl ViewObj for RootView {
    fn client_bounds(&self, _tree: &ViewTree, size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0 }, size }
    }
}

impl RootView {
    property!(Text, bg, set_bg, on_bg_changed, ViewContext);

    fn invalidate_bg(&mut self, context: &mut ViewContext, _old: &Text) {
        let tree = context.get_1();
        tree.window_tree().invalidate_screen();
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    pub fg: Color,
    pub bg: Option<Color>,
    pub attr: Attr,
    pub value: Bow<'static, &'static str>,
}

impl Text {
    pub const SPACE: Text = Text {
        fg: Color::Black,
        bg: None,
        attr: Attr::empty(),
        value: Bow::Borrowed(&" ")
    };
}
