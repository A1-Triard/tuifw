
pub trait Layout {
    fn measure<Error>(ViewTree
}

struct ViewNode {
    properties: Box<dyn Any>,
    drawing: Option<Box<dyn Drawing>>,
    layout: Option<Box<dyn Layout>>,
    next: View,
    last_child: Option<View>,
}

pub struct ViewTree<Error> {
    window_tree: WindowTree<View, Error>,
}

pub type ViewContext<Error> = ContextMut<ViewTree<Error>>;

#[derive(Debug)]
pub struct Grapheme {
    pub fg: Color,
    pub bg: Option<Color>,
    pub attr: Attr,
    pub text: Bow<'static, &'static str>,
}

pub struct BorderView<Error> {
    window: Window<View>,
    tl: Property<Self, Option<Grapheme>, ViewContext<Error>>,
    tr: Property<Self, Option<Grapheme>, ViewContext<Error>>,
    bl: Property<Self, Option<Grapheme>, ViewContext<Error>>,
    br: Property<Self, Option<Grapheme>, ViewContext<Error>>,
    l: Property<Self, Option<Grapheme>, ViewContext<Error>>,
    t: Property<Self, Option<Grapheme>, ViewContext<Error>>,
    r: Property<Self, Option<Grapheme>, ViewContext<Error>>,
    b: Property<Self, Option<Grapheme>, ViewContext<Error>>
}

impl<Error> BorderView<Error> {
    pub fn new(
        tree: &mut ViewTree<Error>,
        parent: View,
    ) -> View {
        let mut d = Border {
            None,
            tl: Property::new(None),
            tr: Property::new(None),
            bl: Property::new(None),
            br: Property::new(None),
            l: Property::new(None),
            t: Property::new(None),
            r: Property::new(None),
            b: Property::new(None),
        };
        let window = parent_bounds.map(|(parent, bounds)| Window::new(tree, parent, bounds, ))
        d.on_changed_tl(Self::invalidate_tl);
        d.on_changed_tr(Self::invalidate_tr);
        d.on_changed_bl(Self::invalidate_bl);
        d.on_changed_br(Self::invalidate_br);
        d.on_changed_l(Self::invalidate_l);
        d.on_changed_t(Self::invalidate_t);
        d.on_changed_r(Self::invalidate_r);
        d.on_changed_b(Self::invalidate_b);
        d
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
