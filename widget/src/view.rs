use std::any::Any;

pub trait Layout {
    fn measure(&self, tree: &mut ViewTree, view: View, w: Option<i16>, h: Option<i16>) -> Size;
    fn arrange(&self, tree: &mut ViewTree, view: View, size: Size) -> Size;
}

pub trait Draw {
    fn draw(&self, tree: &ViewTree, view: View, port: &mut DrawingPort);
}

struct ViewNode {
    properties: Box<dyn Any>,
    window: Option<(Box<dyn Draw>, Window<View>)>,
    layout: Option<Box<dyn Layout>>,
    next: View,
    last_child: Option<View>,
}

Component!(() struct ViewNode { ... });

pub struct ViewTree {
    arena: Arena<ViewNode>,
    window_tree: WindowTree<View>,
}

pub struct View(Id<ViewNode>);

pub type ViewContext = ContextMut<ViewTree>;

struct BorderDraw;

#[derive(Debug)]
pub struct Grapheme {
    pub fg: Color,
    pub bg: Option<Color>,
    pub attr: Attr,
    pub text: Bow<'static, &'static str>,
}

pub struct BorderView {
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
        let parent_window = tree.arena[parent].window.1;
        let mut properties = BorderView {
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
        View(tree.arena.insert(|this| {
            let window = Window::new(
                &mut tree.window_tree,
                parent_window,
                Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                |_| View(this)
            );
            ViewNode {
                properties,
                window: Some(Box::new(BorderDraw) as _, window),
                layout: None,
                next: this,
                last_child: None
            }
        }))
    }

    property!(Option<Grapheme>, tl, set_tl, on_changed_tl, ViewContext<Error>);
    property!(Option<Grapheme>, tr, set_tr, on_changed_tr, ViewContext<Error>);
    property!(Option<Grapheme>, bl, set_bl, on_changed_bl, ViewContext<Error>);
    property!(Option<Grapheme>, br, set_br, on_changed_br, ViewContext<Error>);
    property!(Option<Grapheme>, l, set_l, on_changed_l, ViewContext<Error>);
    property!(Option<Grapheme>, t, set_t, on_changed_t, ViewContext<Error>);
    property!(Option<Grapheme>, r, set_r, on_changed_r, ViewContext<Error>);
    property!(Option<Grapheme>, b, set_b, on_changed_b, ViewContext<Error>);

    fn invalidate_tl(&mut self, context: &mut ViewContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_tr(&mut self, context: &mut ViewContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_bl(&mut self, context: &mut ViewContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_br(&mut self, context: &mut ViewContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_l(&mut self, context: &mut ViewContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: size.y }
        });
    }

    fn invalidate_t(&mut self, context: &mut ViewContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: size.x, y: 1 }
        });
    }

    fn invalidate_r(&mut self, context: &mut ViewContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: size.y }
        });
    }

    fn invalidate_b(&mut self, context: &mut ViewContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: size.x, y: 1 }
        });
    }

}

impl Draw for BorderDraw {
    fn draw(&self, tree: &ViewTree, view: View, port: &mut DrawingPort) {
        let size = self.window.size(tree);
        if let Some(l) = self.l() {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: 0, y: y as i16 }, l.fg, l.bg, l.attr, &l.text);
            }
        }
        if let Some(t) = self.t() {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: 0 }, t.fg, t.bg, t.attr, &t.text);
            }
        }
        if let Some(r) = self.r() {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: size.x.overflowing_sub(1).0, y: y as i16 }, r.fg, r.bg, r.attr, &r.text);
            }
        }
        if let Some(b) = self.b() {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: size.y.overflowing_sub(1).0 }, b.fg, b.bg, b.attr, &b.text);
            }
        }
        if let Some(tl) = self.tl() {
            port.out(Point { x: 0, y: 0 }, tl.fg, tl.bg, tl.attr, &tl.text);
        }
        if let Some(tr) = self.tr() {
            port.out(Point { x: size.x.overflowing_sub(1).0, y: 0 }, tr.fg, tr.bg, tr.attr, &tr.text);
        }
        if let Some(bl) = self.bl() {
            port.out(Point { x: 0, y: size.y.overflowing_sub(1).0 }, bl.fg, bl.bg, bl.attr, &bl.text);
        }
        if let Some(br) = self.br() {
            let p = Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 };
            port.out(p, br.fg, br.bg, br.attr, &br.text);
        }
    }
}
