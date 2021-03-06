use std::borrow::{Borrow, Cow};
use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use dep_obj::{dep_type, DepObjBuilderCore};
use dyn_context::{Context, ContextExt};
use either::{Left, Right};
use crate::view::base::*;

pub trait ViewBuilderBorderDecoratorExt {
    fn border_decorator(
        self,
        f: impl for<'a> FnOnce(BorderDecoratorBuilder<'a>) -> BorderDecoratorBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderBorderDecoratorExt for ViewBuilder<'a> {
    fn border_decorator(
        mut self,
        f: impl for<'b> FnOnce(BorderDecoratorBuilder<'b>) -> BorderDecoratorBuilder<'b>
    ) -> Self {
        let view = self.id();
        let tree: &mut ViewTree = self.context_mut().get_mut();
        BorderDecorator::new(tree, view);
        f(BorderDecoratorBuilder::new_priv(self)).core_priv()
    }
}

dep_type! {
    #[derive(Debug)]
    pub struct BorderDecorator become decorator in View {
        tl: Cow<'static, str> = Cow::Borrowed(""),
        tr: Cow<'static, str> = Cow::Borrowed(""),
        bl: Cow<'static, str> = Cow::Borrowed(""),
        br: Cow<'static, str> = Cow::Borrowed(""),
        l: Cow<'static, str> = Cow::Borrowed(""),
        t: Cow<'static, str> = Cow::Borrowed(""),
        r: Cow<'static, str> = Cow::Borrowed(""),
        b: Cow<'static, str> = Cow::Borrowed(""),
    }

    type BuilderCore<'a> = ViewBuilder<'a>;
}

impl BorderDecorator {
    const BEHAVIOR: BorderDecoratorBehavior = BorderDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_decorator(tree, BorderDecorator::new_priv());
        view.decorator(tree).on_changed(BorderDecorator::TL, Self::invalidate_tl);
        view.decorator(tree).on_changed(BorderDecorator::TR, Self::invalidate_tr);
        view.decorator(tree).on_changed(BorderDecorator::BL, Self::invalidate_bl);
        view.decorator(tree).on_changed(BorderDecorator::BR, Self::invalidate_br);
        view.decorator(tree).on_changed(BorderDecorator::L, Self::invalidate_l);
        view.decorator(tree).on_changed(BorderDecorator::T, Self::invalidate_t);
        view.decorator(tree).on_changed(BorderDecorator::R, Self::invalidate_r);
        view.decorator(tree).on_changed(BorderDecorator::B, Self::invalidate_b);
    }

    fn invalidate_tl(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_tr(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.wrapping_sub(1), y: 0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_bl(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.wrapping_sub(1) },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_br(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.wrapping_sub(1), y: size.y.wrapping_sub(1) },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_l(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: size.y }
        }).unwrap();
    }

    fn invalidate_t(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: size.x, y: 1 }
        }).unwrap();
    }

    fn invalidate_r(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.wrapping_sub(1), y: 0 },
            size: Vector { x: 1, y: size.y }
        }).unwrap();
    }

    fn invalidate_b(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let size = view.render_bounds(tree).size;
        view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.wrapping_sub(1) },
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
        let tl = !view.decorator_ref(tree).get(BorderDecorator::TL).is_empty();
        let tr = !view.decorator_ref(tree).get(BorderDecorator::TR).is_empty();
        let bl = !view.decorator_ref(tree).get(BorderDecorator::BL).is_empty();
        let br = !view.decorator_ref(tree).get(BorderDecorator::BR).is_empty();
        let children_measure_width = if let Some(measure_width) = measure_size.0 {
            let l = tl || bl || !view.decorator_ref(tree).get(BorderDecorator::L).is_empty();
            let r = tr || br || !view.decorator_ref(tree).get(BorderDecorator::R).is_empty();
            Some((measure_width as u16).saturating_sub(if l { 1 } else { 0 }).saturating_sub(if r { 1 } else { 0 }) as i16)
        } else {
            None
        };
        let children_measure_height = if let Some(measure_height) = measure_size.1 {
            let t = tl || tr || !view.decorator_ref(tree).get(BorderDecorator::T).is_empty();
            let b = bl || br || !view.decorator_ref(tree).get(BorderDecorator::B).is_empty();
            Some((measure_height as u16).saturating_sub(if t { 1 } else { 0 }).saturating_sub(if b { 1 } else { 0 }) as i16)
        } else {
            None
        };
        (children_measure_width, children_measure_height)
    }

    fn desired_size(&self, view: View, tree: &mut ViewTree, children_desired_size: Vector) -> Vector {
        let tl = !view.decorator_ref(tree).get(BorderDecorator::TL).is_empty();
        let tr = !view.decorator_ref(tree).get(BorderDecorator::TR).is_empty();
        let bl = !view.decorator_ref(tree).get(BorderDecorator::BL).is_empty();
        let br = !view.decorator_ref(tree).get(BorderDecorator::BR).is_empty();
        let l = tl || bl || !view.decorator_ref(tree).get(BorderDecorator::L).is_empty();
        let r = tr || br || !view.decorator_ref(tree).get(BorderDecorator::R).is_empty();
        let desired_width = (children_desired_size.x as u16)
            .saturating_add(if l { 1 } else { 0 })
            .saturating_add(if r { 1 } else { 0 })
            as i16
        ;
        let t = tl || tr || !view.decorator_ref(tree).get(BorderDecorator::T).is_empty();
        let b = bl || br || !view.decorator_ref(tree).get(BorderDecorator::B).is_empty();
        let desired_height = (children_desired_size.y as u16)
            .saturating_add(if t { 1 } else { 0 })
            .saturating_add(if b { 1 } else { 0 })
            as i16
        ;
        Vector { x : desired_width, y: desired_height }
    }

    fn children_arrange_bounds(&self, view: View, tree: &mut ViewTree, arrange_size: Vector) -> Rect {
        let tl = !view.decorator_ref(tree).get(BorderDecorator::TL).is_empty();
        let tr = !view.decorator_ref(tree).get(BorderDecorator::TR).is_empty();
        let bl = !view.decorator_ref(tree).get(BorderDecorator::BL).is_empty();
        let br = !view.decorator_ref(tree).get(BorderDecorator::BR).is_empty();
        let l = tl || bl || !view.decorator_ref(tree).get(BorderDecorator::L).is_empty();
        let t = tl || tr || !view.decorator_ref(tree).get(BorderDecorator::T).is_empty();
        let tl_offset = Point {
            x: if l { 1 } else { 0 },
            y: if t { 1 } else { 0 },
        };
        let r = tr || br || !view.decorator_ref(tree).get(BorderDecorator::R).is_empty();
        let b = bl || br || !view.decorator_ref(tree).get(BorderDecorator::B).is_empty();
        let br_offset = Vector {
            x: if r { -1 } else { 0 },
            y: if b { -1 } else { 0 },
        };
        Rect::from_tl_br(tl_offset, Point { x: 0, y: 0}.offset(arrange_size + br_offset))
            .intersect(Rect { tl: Point { x: 0, y: 0 }, size: arrange_size })
    }

    fn render_bounds(&self, view: View, tree: &mut ViewTree, children_render_bounds: Rect) -> Rect {
        let tl = !view.decorator_ref(tree).get(BorderDecorator::TL).is_empty();
        let tr = !view.decorator_ref(tree).get(BorderDecorator::TR).is_empty();
        let bl = !view.decorator_ref(tree).get(BorderDecorator::BL).is_empty();
        let br = !view.decorator_ref(tree).get(BorderDecorator::BR).is_empty();
        let l = tl || bl || !view.decorator_ref(tree).get(BorderDecorator::L).is_empty();
        let t = tl || tr || !view.decorator_ref(tree).get(BorderDecorator::T).is_empty();
        let tl_offset = Vector {
            x: if l { -1 } else { 0 },
            y: if t { -1 } else { 0 },
        };
        let r = tr || br || !view.decorator_ref(tree).get(BorderDecorator::R).is_empty();
        let b = bl || br || !view.decorator_ref(tree).get(BorderDecorator::B).is_empty();
        let br_offset = Vector {
            x: if r { 1 } else { 0 },
            y: if b { 1 } else { 0 },
        };
        let render_bounds = Rect::from_tl_br(
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
        let tl = view.decorator_ref(tree).get(BorderDecorator::TL);
        let tr = view.decorator_ref(tree).get(BorderDecorator::TR);
        let bl = view.decorator_ref(tree).get(BorderDecorator::BL);
        let br = view.decorator_ref(tree).get(BorderDecorator::BR);
        let l: &str = view.decorator_ref(tree).get(BorderDecorator::L).borrow();
        let r: &str = view.decorator_ref(tree).get(BorderDecorator::R).borrow();
        let t: &str = view.decorator_ref(tree).get(BorderDecorator::T).borrow();
        let b: &str = view.decorator_ref(tree).get(BorderDecorator::B).borrow();
        let l = if !l.is_empty() { l } else if !tl.is_empty() || !bl.is_empty() { " " } else { "" };
        let t = if !t.is_empty() { t } else if !tl.is_empty() || !tr.is_empty() { " " } else { "" };
        let r = if !r.is_empty() { r } else if !tr.is_empty() || !br.is_empty() { " " } else { "" };
        let b = if !b.is_empty() { b } else if !bl.is_empty() || !br.is_empty() { " " } else { "" };
        let fg = view.actual_fg(tree);
        let bg = view.actual_bg(tree);
        let attr = view.actual_attr(tree);
        if !l.is_empty() {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: 0, y: y as i16 }, fg, bg, attr, l);
            }
        }
        if !r.is_empty() {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: size.x.wrapping_sub(1), y: y as i16 }, fg, bg, attr, r);
            }
        }
        if !t.is_empty() {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: 0 }, fg, bg, attr, t);
            }
        }
        if !b.is_empty() {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: size.y.wrapping_sub(1) }, fg, bg, attr, b);
            }
        }
        if !tl.is_empty() {
            port.out(Point { x: 0, y: 0 }, fg, bg, attr, tl);
        }
        if !tr.is_empty() {
            port.out(Point { x: size.x.wrapping_sub(1), y: 0 }, fg, bg, attr, tr);
        }
        if !bl.is_empty() {
            port.out(Point { x: 0, y: size.y.wrapping_sub(1) }, fg, bg, attr, bl);
        }
        if !br.is_empty() {
            let p = Point { x: size.x.wrapping_sub(1), y: size.y.wrapping_sub(1) };
            port.out(p, fg, bg, attr, br);
        }
    }
}
