use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use dep_obj::{dep_obj, DepTypeToken, Context, ContextExt};
use once_cell::sync::{self};
use either::{Left, Right};
use crate::view::base::*;

dep_obj! {
    #[derive(Debug)]
    pub struct BorderDecorator as View: BorderDecoratorType {
        tl: Option<Text> = None,
        tr: Option<Text> = None,
        bl: Option<Text> = None,
        br: Option<Text> = None,
        l: Option<Text> = None,
        t: Option<Text> = None,
        r: Option<Text> = None,
        b: Option<Text> = None,
    }
}

static BORDER_DECORATOR_TOKEN: sync::Lazy<DepTypeToken<BorderDecoratorType>> = sync::Lazy::new(||
    BorderDecoratorType::new_raw().expect("BorderDecoratorType builder locked")
);

pub fn border_decorator_type() -> &'static BorderDecoratorType { BORDER_DECORATOR_TOKEN.ty() }

impl BorderDecorator {
    const BEHAVIOR: BorderDecoratorBehavior = BorderDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_decorator(tree, BorderDecorator::new_raw(&BORDER_DECORATOR_TOKEN));
        view.decorator_on_changed(tree, border_decorator_type().tl(), Self::invalidate_tl);
        view.decorator_on_changed(tree, border_decorator_type().tr(), Self::invalidate_tr);
        view.decorator_on_changed(tree, border_decorator_type().bl(), Self::invalidate_bl);
        view.decorator_on_changed(tree, border_decorator_type().br(), Self::invalidate_br);
        view.decorator_on_changed(tree, border_decorator_type().l(), Self::invalidate_l);
        view.decorator_on_changed(tree, border_decorator_type().t(), Self::invalidate_t);
        view.decorator_on_changed(tree, border_decorator_type().r(), Self::invalidate_r);
        view.decorator_on_changed(tree, border_decorator_type().b(), Self::invalidate_b);
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

impl Decorator for BorderDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior { &Self::BEHAVIOR }
}

struct BorderDecoratorBehavior;

impl DecoratorBehavior for BorderDecoratorBehavior {
    fn children_measure_size(
        &self,
        view: View,
        tree: &mut ViewTree,
        measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        let tl = view.decorator_get(tree, border_decorator_type().tl()).is_some();
        let tr = view.decorator_get(tree, border_decorator_type().tr()).is_some();
        let bl = view.decorator_get(tree, border_decorator_type().bl()).is_some();
        let br = view.decorator_get(tree, border_decorator_type().br()).is_some();
        let children_measure_width = if let Some(measure_width) = measure_size.0 {
            let l = tl || bl || view.decorator_get(tree, border_decorator_type().l()).is_some();
            let r = tr || br || view.decorator_get(tree, border_decorator_type().r()).is_some();
            Some((measure_width as u16).saturating_sub(if l { 1 } else { 0 }).saturating_sub(if r { 1 } else { 0 }) as i16)
        } else {
            None
        };
        let children_measure_height = if let Some(measure_height) = measure_size.1 {
            let t = tl || tr || view.decorator_get(tree, border_decorator_type().t()).is_some();
            let b = bl || br || view.decorator_get(tree, border_decorator_type().b()).is_some();
            Some((measure_height as u16).saturating_sub(if t { 1 } else { 0 }).saturating_sub(if b { 1 } else { 0 }) as i16)
        } else {
            None
        };
        (children_measure_width, children_measure_height)
    }

    fn desired_size(&self, view: View, tree: &mut ViewTree, children_desired_size: Vector) -> Vector {
        let tl = view.decorator_get(tree, border_decorator_type().tl()).is_some();
        let tr = view.decorator_get(tree, border_decorator_type().tr()).is_some();
        let bl = view.decorator_get(tree, border_decorator_type().bl()).is_some();
        let br = view.decorator_get(tree, border_decorator_type().br()).is_some();
        let l = tl || bl || view.decorator_get(tree, border_decorator_type().l()).is_some();
        let r = tr || br || view.decorator_get(tree, border_decorator_type().r()).is_some();
        let desired_width = (children_desired_size.x as u16)
            .saturating_add(if l { 1 } else { 0 })
            .saturating_add(if r { 1 } else { 0 })
            as i16
        ;
        let t = tl || tr || view.decorator_get(tree, border_decorator_type().t()).is_some();
        let b = bl || br || view.decorator_get(tree, border_decorator_type().b()).is_some();
        let desired_height = (children_desired_size.y as u16)
            .saturating_add(if t { 1 } else { 0 })
            .saturating_add(if b { 1 } else { 0 })
            as i16
        ;
        Vector { x : desired_width, y: desired_height }
    }

    fn children_arrange_bounds(&self, view: View, tree: &mut ViewTree, arrange_size: Vector) -> Rect {
        let tl = view.decorator_get(tree, border_decorator_type().tl()).is_some();
        let tr = view.decorator_get(tree, border_decorator_type().tr()).is_some();
        let bl = view.decorator_get(tree, border_decorator_type().bl()).is_some();
        let br = view.decorator_get(tree, border_decorator_type().br()).is_some();
        let l = tl || bl || view.decorator_get(tree, border_decorator_type().l()).is_some();
        let t = tl || tr || view.decorator_get(tree, border_decorator_type().t()).is_some();
        let tl_offset = Point {
            x: if l { 1 } else { 0 },
            y: if t { 1 } else { 0 },
        };
        let r = tr || br || view.decorator_get(tree, border_decorator_type().r()).is_some();
        let b = bl || br || view.decorator_get(tree, border_decorator_type().b()).is_some();
        let br_offset = Vector {
            x: if r { -1 } else { 0 },
            y: if b { -1 } else { 0 },
        };
        Rect::with_tl_br(tl_offset, Point { x: 0, y: 0}.offset(arrange_size + br_offset))
            .intersect(Rect { tl: Point { x: 0, y: 0 }, size: arrange_size })
    }

    fn render_bounds(&self, view: View, tree: &mut ViewTree, children_render_bounds: Rect) -> Rect {
        let tl = view.decorator_get(tree, border_decorator_type().tl()).is_some();
        let tr = view.decorator_get(tree, border_decorator_type().tr()).is_some();
        let bl = view.decorator_get(tree, border_decorator_type().bl()).is_some();
        let br = view.decorator_get(tree, border_decorator_type().br()).is_some();
        let l = tl || bl || view.decorator_get(tree, border_decorator_type().l()).is_some();
        let t = tl || tr || view.decorator_get(tree, border_decorator_type().t()).is_some();
        let tl_offset = Vector {
            x: if l { -1 } else { 0 },
            y: if t { -1 } else { 0 },
        };
        let r = tr || br || view.decorator_get(tree, border_decorator_type().r()).is_some();
        let b = bl || br || view.decorator_get(tree, border_decorator_type().b()).is_some();
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
        let tl = view.decorator_get(tree, border_decorator_type().tl());
        let tr = view.decorator_get(tree, border_decorator_type().tr());
        let bl = view.decorator_get(tree, border_decorator_type().bl());
        let br = view.decorator_get(tree, border_decorator_type().br());
        let l = view.decorator_get(tree, border_decorator_type().l());
        let r = view.decorator_get(tree, border_decorator_type().r());
        let t = view.decorator_get(tree, border_decorator_type().t());
        let b = view.decorator_get(tree, border_decorator_type().b());
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
