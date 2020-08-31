use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use crate::view::base::*;

pub struct BorderDecoratorType {
    token: DepTypeToken<BorderDecorator>,
    tl: DepProp<BorderDecorator, Reactive<View, Option<Text>>>,
    tr: DepProp<BorderDecorator, Reactive<View, Option<Text>>>,
    bl: DepProp<BorderDecorator, Reactive<View, Option<Text>>>,
    br: DepProp<BorderDecorator, Reactive<View, Option<Text>>>,
    l: DepProp<BorderDecorator, Reactive<View, Option<Text>>>,
    t: DepProp<BorderDecorator, Reactive<View, Option<Text>>>,
    r: DepProp<BorderDecorator, Reactive<View, Option<Text>>>,
    b: DepProp<BorderDecorator, Reactive<View, Option<Text>>>,
}

impl BorderDecoratorType {
    pub fn token(&self) -> &DepTypeToken<BorderDecorator> { &self.token }
    pub fn tl(&self) -> DepProp<BorderDecorator, Reactive<View, Option<Text>>> { self.tl }
    pub fn tr(&self) -> DepProp<BorderDecorator, Reactive<View, Option<Text>>> { self.tr }
    pub fn bl(&self) -> DepProp<BorderDecorator, Reactive<View, Option<Text>>> { self.bl }
    pub fn br(&self) -> DepProp<BorderDecorator, Reactive<View, Option<Text>>> { self.br }
    pub fn l(&self) -> DepProp<BorderDecorator, Reactive<View, Option<Text>>> { self.l }
    pub fn t(&self) -> DepProp<BorderDecorator, Reactive<View, Option<Text>>> { self.t }
    pub fn r(&self) -> DepProp<BorderDecorator, Reactive<View, Option<Text>>> { self.r }
    pub fn b(&self) -> DepProp<BorderDecorator, Reactive<View, Option<Text>>> { self.b }
}

pub static BORDER_DECORATOR_TYPE: sync::Lazy<BorderDecoratorType> = sync::Lazy::new(|| {
    let mut builder = DepTypeBuilder::new().expect("BorderDecoratorType builder locked");
    let tl = builder.prop(|| Reactive::new(None));
    let tr = builder.prop(|| Reactive::new(None));
    let bl = builder.prop(|| Reactive::new(None));
    let br = builder.prop(|| Reactive::new(None));
    let l = builder.prop(|| Reactive::new(None));
    let t = builder.prop(|| Reactive::new(None));
    let r = builder.prop(|| Reactive::new(None));
    let b = builder.prop(|| Reactive::new(None));
    let token = builder.build();
    BorderDecoratorType {
        token,
        tl, tr, bl, br,
        l, t, r, b,
    }
});

macro_attr! {
    #[derive(DepObjRaw!)]
    #[derive(Debug)]
    pub struct BorderDecorator {
        view: View,
        dep_props: DepObjProps<Self>,
    }
}

impl BorderDecorator {
    pub fn new(
        tree: &mut ViewTree,
        parent: View,
    ) -> View {
        let view = View::new(tree, parent, |view| {
            let decorator = BorderDecorator {
                view,
                dep_props: DepObjProps::new()
            };
            (Some(Box::new(decorator) as _), None, view)
        });
        view.decorator_on_changed(&mut tree, BORDER_DECORATOR_TYPE.tl(), Self::invalidate_tl);
        view.decorator_on_changed(&mut tree, BORDER_DECORATOR_TYPE.tr(), Self::invalidate_tr);
        view.decorator_on_changed(&mut tree, BORDER_DECORATOR_TYPE.bl(), Self::invalidate_bl);
        view.decorator_on_changed(&mut tree, BORDER_DECORATOR_TYPE.br(), Self::invalidate_br);
        view.decorator_on_changed(&mut tree, BORDER_DECORATOR_TYPE.l(), Self::invalidate_l);
        view.decorator_on_changed(&mut tree, BORDER_DECORATOR_TYPE.t(), Self::invalidate_t);
        view.decorator_on_changed(&mut tree, BORDER_DECORATOR_TYPE.r(), Self::invalidate_r);
        view.decorator_on_changed(&mut tree, BORDER_DECORATOR_TYPE.b(), Self::invalidate_b);
        view
    }

    fn invalidate_tl(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_tr(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.size(tree).unwrap();
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_bl(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.size(tree).unwrap();
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_br(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.size(tree).unwrap();
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_l(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.size(tree).unwrap();
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: size.y }
        }).unwrap();
    }

    fn invalidate_t(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.size(tree).unwrap();
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: size.x, y: 1 }
        }).unwrap();
    }

    fn invalidate_r(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.size(tree).unwrap();
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: size.y }
        }).unwrap();
    }

    fn invalidate_b(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.size(tree).unwrap();
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: size.x, y: 1 }
        }).unwrap();
    }
}

impl ViewObj for BorderView {
    fn client_bounds(&self, _tree: &ViewTree, size: Vector) -> Rect {
        let tl = Point {
            x: if self.l().is_some() || self.tl().is_some() || self.bl().is_some() { 1 } else { 0 },
            y: if self.t().is_some() || self.tl().is_some() || self.tr().is_some() { 1 } else { 0 },
        };
        let br = Vector {
            x: if self.r().is_some() || self.tr().is_some() || self.br().is_some() { -1 } else { 0 },
            y: if self.t().is_some() || self.tl().is_some() || self.tr().is_some() { -1 } else { 0 },
        };
        Rect { tl, size: size + br }.intersect(Rect { tl: Point { x: 0, y: 0 }, size })
    }
}

impl Render for BorderRender {
    fn render(&self, tree: &ViewTree, view: View, port: &mut RenderPort) {
        let size = view.size(tree).unwrap();
        let obj = view.obj(tree).downcast_ref::<BorderView>().unwrap();
        let l = obj.l().as_ref().or_else(|| if obj.tl().is_some() || obj.bl().is_some() { Some(&Text::SPACE) } else { None });
        let t = obj.t().as_ref().or_else(|| if obj.tl().is_some() || obj.tr().is_some() { Some(&Text::SPACE) } else { None });
        let r = obj.r().as_ref().or_else(|| if obj.tr().is_some() || obj.br().is_some() { Some(&Text::SPACE) } else { None });
        let b = obj.b().as_ref().or_else(|| if obj.bl().is_some() || obj.br().is_some() { Some(&Text::SPACE) } else { None });
        if let Some(l) = l {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: 0, y: y as i16 }, l.fg, l.bg, l.attr, &l.value);
            }
        }
        if let Some(r) = r {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: size.x.overflowing_sub(1).0, y: y as i16 }, r.fg, r.bg, r.attr, &r.value);
            }
        }
        if let Some(t) = t {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: 0 }, t.fg, t.bg, t.attr, &t.value);
            }
        }
        if let Some(b) = b {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: size.y.overflowing_sub(1).0 }, b.fg, b.bg, b.attr, &b.value);
            }
        }
        if let Some(tl) = obj.tl() {
            port.out(Point { x: 0, y: 0 }, tl.fg, tl.bg, tl.attr, &tl.value);
        }
        if let Some(tr) = obj.tr() {
            port.out(Point { x: size.x.overflowing_sub(1).0, y: 0 }, tr.fg, tr.bg, tr.attr, &tr.value);
        }
        if let Some(bl) = obj.bl() {
            port.out(Point { x: 0, y: size.y.overflowing_sub(1).0 }, bl.fg, bl.bg, bl.attr, &bl.value);
        }
        if let Some(br) = obj.br() {
            let p = Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 };
            port.out(p, br.fg, br.bg, br.attr, &br.value);
        }
    }
}
