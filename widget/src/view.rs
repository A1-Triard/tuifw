use std::any::Any;
use std::fmt::Debug;
use std::iter::{self};
use std::mem::{replace};
use boow::Bow;
use components_arena::{Id, Arena, ComponentClassMutex};
use tuifw_screen_base::{Vector, Point, Rect, Attr, Color};
use tuifw_window::{DrawingPort, WindowTree, Window};
use crate::context::ContextMut;
use crate::property::Property;

pub trait Layout: Debug {
    fn measure(&self, tree: &mut ViewTree, view: View, w: Option<i16>, h: Option<i16>) -> Vector;
    fn arrange(&self, tree: &mut ViewTree, view: View, size: Vector) -> Vector;
}

pub trait Draw: Debug {
    fn draw(&self, tree: &ViewTree, view: View, port: &mut DrawingPort);
}

pub trait ViewProperties: Any + Debug { }

macro_attr! {
    #[derive(Debug)]
    #[derive(Component!)]
    struct ViewNode {
        properties: Box<dyn ViewProperties>,
        window: Option<(Box<dyn Draw>, Window<View>)>,
        layout: Option<Box<dyn Layout>>,
        parent: Option<View>,
        next: View,
        last_child: Option<View>,
    }
}

static VIEW_NODE: ComponentClassMutex<ViewNode> = ComponentClassMutex::new();

#[derive(Debug)]
pub struct ViewTree {
    arena: Arena<ViewNode>,
    window_tree: WindowTree<View>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct View(Id<ViewNode>);

impl View {
    pub fn parent(self, tree: &ViewTree) -> Option<View> { tree.arena[self.0].parent }

    pub fn self_and_parents<'a>(self, tree: &'a ViewTree) -> impl Iterator<Item=View> + 'a {
        let mut view = Some(self);
        iter::from_fn(move || {
            let parent = view.and_then(|view| view.parent(tree));
            replace(&mut view, parent)
        })
    }
}

pub type ViewContext = ContextMut<ViewTree>;

#[derive(Debug)]
struct BorderDraw;

#[derive(Debug)]
pub struct Grapheme {
    pub fg: Color,
    pub bg: Option<Color>,
    pub attr: Attr,
    pub text: Bow<'static, &'static str>,
}

#[derive(Debug)]
pub struct BorderView {
    this: View,
    tl: Property<Self, Option<Grapheme>, ViewContext>,
    tr: Property<Self, Option<Grapheme>, ViewContext>,
    bl: Property<Self, Option<Grapheme>, ViewContext>,
    br: Property<Self, Option<Grapheme>, ViewContext>,
    l: Property<Self, Option<Grapheme>, ViewContext>,
    t: Property<Self, Option<Grapheme>, ViewContext>,
    r: Property<Self, Option<Grapheme>, ViewContext>,
    b: Property<Self, Option<Grapheme>, ViewContext>
}

impl BorderView {
    pub fn new(
        tree: &mut ViewTree,
        parent: View,
    ) -> View {
        let parent_window = parent
            .self_and_parents(tree)
            .find_map(|view| tree.arena[view.0].window.map(|x| x.1))
        ;
        View(tree.arena.insert(|this| {
            let mut properties = BorderView {
                this: View(this),
                tl: Property::new(None),
                tr: Property::new(None),
                bl: Property::new(None),
                br: Property::new(None),
                l: Property::new(None),
                t: Property::new(None),
                r: Property::new(None),
                b: Property::new(None),
            };
            properties.on_changed_tl(Self::invalidate_tl);
            properties.on_changed_tr(Self::invalidate_tr);
            properties.on_changed_bl(Self::invalidate_bl);
            properties.on_changed_br(Self::invalidate_br);
            properties.on_changed_l(Self::invalidate_l);
            properties.on_changed_t(Self::invalidate_t);
            properties.on_changed_r(Self::invalidate_r);
            properties.on_changed_b(Self::invalidate_b);
            let window = Window::new(
                &mut tree.window_tree,
                parent_window,
                Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                |_| View(this)
            );
            ViewNode {
                properties: Box::new(properties) as _,
                window: Some((Box::new(BorderDraw) as _, window)),
                layout: None,
                parent: Some(parent),
                next: View(this),
                last_child: None
            }
        }))
    }

    property!(Option<Grapheme>, tl, set_tl, on_changed_tl, ViewContext);
    property!(Option<Grapheme>, tr, set_tr, on_changed_tr, ViewContext);
    property!(Option<Grapheme>, bl, set_bl, on_changed_bl, ViewContext);
    property!(Option<Grapheme>, br, set_br, on_changed_br, ViewContext);
    property!(Option<Grapheme>, l, set_l, on_changed_l, ViewContext);
    property!(Option<Grapheme>, t, set_t, on_changed_t, ViewContext);
    property!(Option<Grapheme>, r, set_r, on_changed_r, ViewContext);
    property!(Option<Grapheme>, b, set_b, on_changed_b, ViewContext);

    fn invalidate_tl(&mut self, context: &mut ViewContext, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let window = tree.arena[self.this.0].window.as_ref().unwrap().1;
        window.invalidate_rect(&mut tree.window_tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_tr(&mut self, context: &mut ViewContext, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let window = tree.arena[self.this.0].window.as_ref().unwrap().1;
        let size = window.size(&tree.window_tree);
        window.invalidate_rect(&mut tree.window_tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_bl(&mut self, context: &mut ViewContext, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let window = tree.arena[self.this.0].window.as_ref().unwrap().1;
        let size = window.size(&tree.window_tree);
        window.invalidate_rect(&mut tree.window_tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_br(&mut self, context: &mut ViewContext, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let window = tree.arena[self.this.0].window.as_ref().unwrap().1;
        let size = window.size(&tree.window_tree);
        window.invalidate_rect(&mut tree.window_tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_l(&mut self, context: &mut ViewContext, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let window = tree.arena[self.this.0].window.as_ref().unwrap().1;
        let size = window.size(&tree.window_tree);
        window.invalidate_rect(&mut tree.window_tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: size.y }
        });
    }

    fn invalidate_t(&mut self, context: &mut ViewContext, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let window = tree.arena[self.this.0].window.as_ref().unwrap().1;
        let size = window.size(&tree.window_tree);
        window.invalidate_rect(&mut tree.window_tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: size.x, y: 1 }
        });
    }

    fn invalidate_r(&mut self, context: &mut ViewContext, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let window = tree.arena[self.this.0].window.as_ref().unwrap().1;
        let size = window.size(&tree.window_tree);
        window.invalidate_rect(&mut tree.window_tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: size.y }
        });
    }

    fn invalidate_b(&mut self, context: &mut ViewContext, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let window = tree.arena[self.this.0].window.as_ref().unwrap().1;
        let size = window.size(&tree.window_tree);
        window.invalidate_rect(&mut tree.window_tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: size.x, y: 1 }
        });
    }
}

impl ViewProperties for BorderView { }

impl Draw for BorderDraw {
    fn draw(&self, tree: &ViewTree, view: View, port: &mut DrawingPort) {
        let node = tree.arena[view.0];
        let size = node.window.as_ref().unwrap().1.size(&tree.window_tree);
        let properties = node.properties.as_ref().downcast_ref::<BorderView>().unwrap();
        if let Some(l) = properties.l() {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: 0, y: y as i16 }, l.fg, l.bg, l.attr, &l.text);
            }
        }
        if let Some(t) = properties.t() {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: 0 }, t.fg, t.bg, t.attr, &t.text);
            }
        }
        if let Some(r) = properties.r() {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: size.x.overflowing_sub(1).0, y: y as i16 }, r.fg, r.bg, r.attr, &r.text);
            }
        }
        if let Some(b) = properties.b() {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: size.y.overflowing_sub(1).0 }, b.fg, b.bg, b.attr, &b.text);
            }
        }
        if let Some(tl) = properties.tl() {
            port.out(Point { x: 0, y: 0 }, tl.fg, tl.bg, tl.attr, &tl.text);
        }
        if let Some(tr) = properties.tr() {
            port.out(Point { x: size.x.overflowing_sub(1).0, y: 0 }, tr.fg, tr.bg, tr.attr, &tr.text);
        }
        if let Some(bl) = properties.bl() {
            port.out(Point { x: 0, y: size.y.overflowing_sub(1).0 }, bl.fg, bl.bg, bl.attr, &bl.text);
        }
        if let Some(br) = properties.br() {
            let p = Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 };
            port.out(p, br.fg, br.bg, br.attr, &br.text);
        }
    }
}
