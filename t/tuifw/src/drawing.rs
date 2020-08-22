use boow::Bow;
use tuifw_screen_base::{Color, Attr, Point, Vector, Rect};
use tuifw_property::Property;
use tuifw_property::context::{ContextMut};
use tuifw_window::{DrawingPort, Window, WindowTree, OptionWindowExt};

pub trait Drawing<Error> {
    fn draw(&self, tree: &WindowTree<Box<dyn Drawing<Error>>, Error>, port: &mut DrawingPort<Error>);
}

#[derive(Debug)]
pub struct Grapheme {
    pub fg: Color,
    pub bg: Option<Color>,
    pub attr: Attr,
    pub text: Bow<'static, &'static str>,
}

pub type DrawingContext<Error> = ContextMut<WindowTree<Box<dyn Drawing<Error>>, Error>>;

macro_attr! {
    #[derive(Debug)]
    #[derive(Component!(class=BorderComponent))]
    pub struct Border<Error> {
        window: Option<Window<Box<dyn Drawing<Error>>>>,
        tl: Property<Self, Option<Grapheme>, DrawingContext<Error>>,
        tr: Property<Self, Option<Grapheme>, DrawingContext<Error>>,
        bl: Property<Self, Option<Grapheme>, DrawingContext<Error>>,
        br: Property<Self, Option<Grapheme>, DrawingContext<Error>>,
        l: Property<Self, Option<Grapheme>, DrawingContext<Error>>,
        t: Property<Self, Option<Grapheme>, DrawingContext<Error>>,
        r: Property<Self, Option<Grapheme>, DrawingContext<Error>>,
        b: Property<Self, Option<Grapheme>, DrawingContext<Error>>
    }
}

impl<Error> Border<Error> {
    fn invalidate_tl(&mut self, context: &mut DrawingContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_tr(&mut self, context: &mut DrawingContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_bl(&mut self, context: &mut DrawingContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_br(&mut self, context: &mut DrawingContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        });
    }

    fn invalidate_l(&mut self, context: &mut DrawingContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: size.y }
        });
    }

    fn invalidate_t(&mut self, context: &mut DrawingContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: size.x, y: 1 }
        });
    }

    fn invalidate_r(&mut self, context: &mut DrawingContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: size.y }
        });
    }

    fn invalidate_b(&mut self, context: &mut DrawingContext<Error>, _old: &Option<Grapheme>) {
        let tree = context.get_1();
        let size = self.window.size(tree);
        self.window.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: size.x, y: 1 }
        });
    }

    pub fn new(
        tree: &mut WindowTree<Tag, Error>,
        parent_bounds: Option<(Option<Window<Box<dyn Drawing<Error>>>>, Rect)>,
    ) -> Self {
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

    property!(Option<Grapheme>, tl, set_tl, on_changed_tl, DrawingContext<Error>);
    property!(Option<Grapheme>, tr, set_tr, on_changed_tr, DrawingContext<Error>);
    property!(Option<Grapheme>, bl, set_bl, on_changed_bl, DrawingContext<Error>);
    property!(Option<Grapheme>, br, set_br, on_changed_br, DrawingContext<Error>);
    property!(Option<Grapheme>, l, set_l, on_changed_l, DrawingContext<Error>);
    property!(Option<Grapheme>, t, set_t, on_changed_t, DrawingContext<Error>);
    property!(Option<Grapheme>, r, set_r, on_changed_r, DrawingContext<Error>);
    property!(Option<Grapheme>, b, set_b, on_changed_b, DrawingContext<Error>);
}

impl<Error> Drawing<Error> for Border<Error> {
    fn draw(&self, tree: &WindowTree<Box<dyn Drawing<Error>>, Error>, port: &mut DrawingPort<Error>) {
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
