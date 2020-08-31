use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use dep_obj::dep::{DepObjProps, DepTypeBuilder, DepProp, DepTypeToken, DepObj};
use dep_obj::reactive::{Context, ContextExt, Reactive};
use once_cell::sync::{self};
use either::{Left, Right};
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
                dep_props: DepObjProps::new(BORDER_DECORATOR_TYPE.token())
            };
            (Some(Box::new(decorator) as _), None, view)
        });
        view.decorator_on_changed(tree, BORDER_DECORATOR_TYPE.tl(), Self::invalidate_tl);
        view.decorator_on_changed(tree, BORDER_DECORATOR_TYPE.tr(), Self::invalidate_tr);
        view.decorator_on_changed(tree, BORDER_DECORATOR_TYPE.bl(), Self::invalidate_bl);
        view.decorator_on_changed(tree, BORDER_DECORATOR_TYPE.br(), Self::invalidate_br);
        view.decorator_on_changed(tree, BORDER_DECORATOR_TYPE.l(), Self::invalidate_l);
        view.decorator_on_changed(tree, BORDER_DECORATOR_TYPE.t(), Self::invalidate_t);
        view.decorator_on_changed(tree, BORDER_DECORATOR_TYPE.r(), Self::invalidate_r);
        view.decorator_on_changed(tree, BORDER_DECORATOR_TYPE.b(), Self::invalidate_b);
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
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_bl(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_br(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_l(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: size.y }
        }).unwrap();
    }

    fn invalidate_t(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: size.x, y: 1 }
        }).unwrap();
    }

    fn invalidate_r(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: size.y }
        }).unwrap();
    }

    fn invalidate_b(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: size.x, y: 1 }
        }).unwrap();
    }
}

impl DepObj for BorderDecorator {
    fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
    fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
}

impl Decorator for BorderDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior {
        static BEHAVIOR: BorderDecoratorBehavior = BorderDecoratorBehavior;
        &BEHAVIOR
    }
}

struct BorderDecoratorBehavior;

impl DecoratorBehavior for BorderDecoratorBehavior {
    fn children_measure_size(
        &self,
        view: View,
        tree: &mut ViewTree,
        measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        let tl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tl()).is_some();
        let tr = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tr()).is_some();
        let bl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.bl()).is_some();
        let br = view.decorator_get(tree, BORDER_DECORATOR_TYPE.br()).is_some();
        let children_measure_width = if let Some(measure_width) = measure_size.0 {
            let l = tl || bl || view.decorator_get(tree, BORDER_DECORATOR_TYPE.l()).is_some();
            let r = tr || br || view.decorator_get(tree, BORDER_DECORATOR_TYPE.r()).is_some();
            Some((measure_width as u16).saturating_sub(if l { 1 } else { 0 }).saturating_sub(if r { 1 } else { 0 }) as i16)
        } else {
            None
        };
        let children_measure_height = if let Some(measure_height) = measure_size.1 {
            let t = tl || tr || view.decorator_get(tree, BORDER_DECORATOR_TYPE.t()).is_some();
            let b = bl || br || view.decorator_get(tree, BORDER_DECORATOR_TYPE.b()).is_some();
            Some((measure_height as u16).saturating_sub(if t { 1 } else { 0 }).saturating_sub(if b { 1 } else { 0 }) as i16)
        } else {
            None
        };
        (children_measure_width, children_measure_height)
    }

    fn desired_size(&self, view: View, tree: &mut ViewTree, children_desired_size: Vector) -> Vector {
        let tl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tl()).is_some();
        let tr = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tr()).is_some();
        let bl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.bl()).is_some();
        let br = view.decorator_get(tree, BORDER_DECORATOR_TYPE.br()).is_some();
        let l = tl || bl || view.decorator_get(tree, BORDER_DECORATOR_TYPE.l()).is_some();
        let r = tr || br || view.decorator_get(tree, BORDER_DECORATOR_TYPE.r()).is_some();
        let desired_width = (children_desired_size.x as u16)
            .saturating_add(if l { 1 } else { 0 })
            .saturating_add(if r { 1 } else { 0 })
            as i16
        ;
        let t = tl || tr || view.decorator_get(tree, BORDER_DECORATOR_TYPE.t()).is_some();
        let b = bl || br || view.decorator_get(tree, BORDER_DECORATOR_TYPE.b()).is_some();
        let desired_height = (children_desired_size.y as u16)
            .saturating_add(if t { 1 } else { 0 })
            .saturating_add(if b { 1 } else { 0 })
            as i16
        ;
        Vector { x : desired_width, y: desired_height }
    }

    fn children_arrange_bounds(&self, view: View, tree: &mut ViewTree, arrange_size: Vector) -> Rect {
        let tl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tl()).is_some();
        let tr = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tr()).is_some();
        let bl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.bl()).is_some();
        let br = view.decorator_get(tree, BORDER_DECORATOR_TYPE.br()).is_some();
        let l = tl || bl || view.decorator_get(tree, BORDER_DECORATOR_TYPE.l()).is_some();
        let t = tl || tr || view.decorator_get(tree, BORDER_DECORATOR_TYPE.t()).is_some();
        let tl_offset = Point {
            x: if l { 1 } else { 0 },
            y: if t { 1 } else { 0 },
        };
        let r = tr || br || view.decorator_get(tree, BORDER_DECORATOR_TYPE.r()).is_some();
        let b = bl || br || view.decorator_get(tree, BORDER_DECORATOR_TYPE.b()).is_some();
        let br_offset = Vector {
            x: if r { -1 } else { 0 },
            y: if b { -1 } else { 0 },
        };
        Rect::with_tl_br(tl_offset, Point { x: 0, y: 0}.offset(arrange_size + br_offset))
            .intersect(Rect { tl: Point { x: 0, y: 0 }, size: arrange_size })
    }

    fn render_bounds(&self, view: View, tree: &mut ViewTree, children_render_bounds: Rect) -> Rect {
        let tl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tl()).is_some();
        let tr = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tr()).is_some();
        let bl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.bl()).is_some();
        let br = view.decorator_get(tree, BORDER_DECORATOR_TYPE.br()).is_some();
        let l = tl || bl || view.decorator_get(tree, BORDER_DECORATOR_TYPE.l()).is_some();
        let t = tl || tr || view.decorator_get(tree, BORDER_DECORATOR_TYPE.t()).is_some();
        let tl_offset = Vector {
            x: if l { -1 } else { 0 },
            y: if t { -1 } else { 0 },
        };
        let r = tr || br || view.decorator_get(tree, BORDER_DECORATOR_TYPE.r()).is_some();
        let b = bl || br || view.decorator_get(tree, BORDER_DECORATOR_TYPE.b()).is_some();
        let br_offset = Vector {
            x: if r { 1 } else { 0 },
            y: if b { 1 } else { 0 },
        };
        let render_bounds = Rect::with_tl_br(
            children_render_bounds.tl.offset(tl_offset),
            children_render_bounds.br().offset(br_offset)
        ).union(children_render_bounds);
        match render_bounds {
            Some(Right(rect)) => rect,
            Some(Left(Left(h_band))) => Rect {
                tl: Point { x: children_render_bounds.l(), y: h_band.t },
                size: Vector { x: children_render_bounds.w(), y: h_band.h.get() }
            },
            Some(Left(Right(v_band))) => Rect {
                tl: Point { y: children_render_bounds.t(), x: v_band.l },
                size: Vector { y: children_render_bounds.h(), x: v_band.w.get() }
            },
            None => children_render_bounds
        }
    }

    fn render(&self, view: View, tree: &ViewTree, port: &mut RenderPort) {
        let size = view.render_bounds(tree).size;
        let tl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tl());
        let tr = view.decorator_get(tree, BORDER_DECORATOR_TYPE.tr());
        let bl = view.decorator_get(tree, BORDER_DECORATOR_TYPE.bl());
        let br = view.decorator_get(tree, BORDER_DECORATOR_TYPE.br());
        let l = view.decorator_get(tree, BORDER_DECORATOR_TYPE.l());
        let r = view.decorator_get(tree, BORDER_DECORATOR_TYPE.r());
        let t = view.decorator_get(tree, BORDER_DECORATOR_TYPE.t());
        let b = view.decorator_get(tree, BORDER_DECORATOR_TYPE.b());
        let l = l.as_ref().or_else(|| if tl.is_some() || bl.is_some() { Some(&Text::SPACE) } else { None });
        let t = t.as_ref().or_else(|| if tl.is_some() || tr.is_some() { Some(&Text::SPACE) } else { None });
        let r = r.as_ref().or_else(|| if tr.is_some() || br.is_some() { Some(&Text::SPACE) } else { None });
        let b = b.as_ref().or_else(|| if bl.is_some() || br.is_some() { Some(&Text::SPACE) } else { None });
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
        if let Some(tl) = tl.as_ref() {
            port.out(Point { x: 0, y: 0 }, tl.fg, tl.bg, tl.attr, &tl.value);
        }
        if let Some(tr) = tr.as_ref() {
            port.out(Point { x: size.x.overflowing_sub(1).0, y: 0 }, tr.fg, tr.bg, tr.attr, &tr.value);
        }
        if let Some(bl) = bl.as_ref() {
            port.out(Point { x: 0, y: size.y.overflowing_sub(1).0 }, bl.fg, bl.bg, bl.attr, &bl.value);
        }
        if let Some(br) = br.as_ref() {
            let p = Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 };
            port.out(p, br.fg, br.bg, br.attr, &br.value);
        }
    }
}
