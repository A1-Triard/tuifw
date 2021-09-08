use crate::view::base::*;
use dep_obj::{DepObjBaseBuilder, dep_type_with_builder};
use dep_obj::binding::{Binding, Binding1};
use dyn_context::state::{State, StateExt};
use either::{Left, Right};
use std::borrow::Cow;
use std::fmt::Debug;
use tuifw_screen_base::{Attr, Color, Point, Rect, Vector};
use tuifw_window::RenderPort;

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
        BorderDecorator::new(self.state_mut(), view);
        f(BorderDecoratorBuilder::new_priv(self)).base_priv()
    }
}

dep_type_with_builder! {
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

    type BaseBuilder<'a> = ViewBuilder<'a>;
}

impl BorderDecorator {
    const BEHAVIOR: BorderDecoratorBehavior = BorderDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        state: &mut dyn State,
        view: View,
    ) {
        view.set_decorator(state, BorderDecorator::new_priv());
    }
}

impl Decorator for BorderDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior { &Self::BEHAVIOR }
}

#[derive(Debug)]
struct BorderDecoratorBindings {
    fg: Binding<Color>,
    bg: Binding<Option<Color>>,
    attr: Binding<Attr>,
    tl: Binding<Cow<'static, str>>,
    tr: Binding<Cow<'static, str>>,
    bl: Binding<Cow<'static, str>>,
    br: Binding<Cow<'static, str>>,
    l: Binding<Cow<'static, str>>,
    t: Binding<Cow<'static, str>>,
    r: Binding<Cow<'static, str>>,
    b: Binding<Cow<'static, str>>,
}

impl DecoratorBindings for BorderDecoratorBindings { }

#[derive(Debug)]
struct BorderDecoratorBehavior;

impl DecoratorBehavior for BorderDecoratorBehavior {
    fn children_measure_size(
        &self,
        view: View,
        state: &mut dyn State,
        measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<BorderDecoratorBindings>().unwrap();
        let tl = !bindings.tl.get_value(state).map_or(false, |x| x.is_empty());
        let tr = !bindings.tr.get_value(state).map_or(false, |x| x.is_empty());
        let bl = !bindings.bl.get_value(state).map_or(false, |x| x.is_empty());
        let br = !bindings.br.get_value(state).map_or(false, |x| x.is_empty());
        let children_measure_width = if let Some(measure_width) = measure_size.0 {
            let l = tl || bl || !bindings.l.get_value(state).map_or(false, |x| x.is_empty());
            let r = tr || br || !bindings.r.get_value(state).map_or(false, |x| x.is_empty());
            Some((measure_width as u16).saturating_sub(if l { 1 } else { 0 }).saturating_sub(if r { 1 } else { 0 }) as i16)
        } else {
            None
        };
        let children_measure_height = if let Some(measure_height) = measure_size.1 {
            let t = tl || tr || !bindings.t.get_value(state).map_or(false, |x| x.is_empty());
            let b = bl || br || !bindings.b.get_value(state).map_or(false, |x| x.is_empty());
            Some((measure_height as u16).saturating_sub(if t { 1 } else { 0 }).saturating_sub(if b { 1 } else { 0 }) as i16)
        } else {
            None
        };
        (children_measure_width, children_measure_height)
    }

    fn desired_size(&self, view: View, state: &mut dyn State, children_desired_size: Vector) -> Vector {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<BorderDecoratorBindings>().unwrap();
        let tl = !bindings.tl.get_value(state).map_or(false, |x| x.is_empty());
        let tr = !bindings.tr.get_value(state).map_or(false, |x| x.is_empty());
        let bl = !bindings.bl.get_value(state).map_or(false, |x| x.is_empty());
        let br = !bindings.br.get_value(state).map_or(false, |x| x.is_empty());
        let l = tl || bl || !bindings.l.get_value(state).map_or(false, |x| x.is_empty());
        let r = tr || br || !bindings.r.get_value(state).map_or(false, |x| x.is_empty());
        let desired_width = (children_desired_size.x as u16)
            .saturating_add(if l { 1 } else { 0 })
            .saturating_add(if r { 1 } else { 0 })
            as i16
        ;
        let t = tl || tr || !bindings.t.get_value(state).map_or(false, |x| x.is_empty());
        let b = bl || br || !bindings.b.get_value(state).map_or(false, |x| x.is_empty());
        let desired_height = (children_desired_size.y as u16)
            .saturating_add(if t { 1 } else { 0 })
            .saturating_add(if b { 1 } else { 0 })
            as i16
        ;
        Vector { x : desired_width, y: desired_height }
    }

    fn children_arrange_bounds(&self, view: View, state: &mut dyn State, arrange_size: Vector) -> Rect {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<BorderDecoratorBindings>().unwrap();
        let tl = !bindings.tl.get_value(state).map_or(false, |x| x.is_empty());
        let tr = !bindings.tr.get_value(state).map_or(false, |x| x.is_empty());
        let bl = !bindings.bl.get_value(state).map_or(false, |x| x.is_empty());
        let br = !bindings.br.get_value(state).map_or(false, |x| x.is_empty());
        let l = tl || bl || !bindings.l.get_value(state).map_or(false, |x| x.is_empty());
        let t = tl || tr || !bindings.t.get_value(state).map_or(false, |x| x.is_empty());
        let tl_offset = Point {
            x: if l { 1 } else { 0 },
            y: if t { 1 } else { 0 },
        };
        let r = tr || br || !bindings.r.get_value(state).map_or(false, |x| x.is_empty());
        let b = bl || br || !bindings.b.get_value(state).map_or(false, |x| x.is_empty());
        let br_offset = Vector {
            x: if r { -1 } else { 0 },
            y: if b { -1 } else { 0 },
        };
        Rect::from_tl_br(tl_offset, Point { x: 0, y: 0}.offset(arrange_size + br_offset))
            .intersect(Rect { tl: Point { x: 0, y: 0 }, size: arrange_size })
    }

    fn render_bounds(&self, view: View, state: &mut dyn State, children_render_bounds: Rect) -> Rect {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<BorderDecoratorBindings>().unwrap();
        let tl = !bindings.tl.get_value(state).map_or(false, |x| x.is_empty());
        let tr = !bindings.tr.get_value(state).map_or(false, |x| x.is_empty());
        let bl = !bindings.bl.get_value(state).map_or(false, |x| x.is_empty());
        let br = !bindings.br.get_value(state).map_or(false, |x| x.is_empty());
        let l = tl || bl || !bindings.l.get_value(state).map_or(false, |x| x.is_empty());
        let t = tl || tr || !bindings.t.get_value(state).map_or(false, |x| x.is_empty());
        let tl_offset = Vector {
            x: if l { -1 } else { 0 },
            y: if t { -1 } else { 0 },
        };
        let r = tr || br || !bindings.r.get_value(state).map_or(false, |x| x.is_empty());
        let b = bl || br || !bindings.b.get_value(state).map_or(false, |x| x.is_empty());
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

    fn render(&self, view: View, state: &dyn State, port: &mut RenderPort) {
        let tree: &ViewTree = state.get();
        let size = view.render_bounds(tree).size;
        let bindings = view.decorator_bindings(tree).downcast_ref::<BorderDecoratorBindings>().unwrap();
        let tl: &str = &bindings.tl.get_value(state).unwrap_or(Cow::Borrowed(""));
        let tr: &str = &bindings.tr.get_value(state).unwrap_or(Cow::Borrowed(""));
        let bl: &str = &bindings.bl.get_value(state).unwrap_or(Cow::Borrowed(""));
        let br: &str = &bindings.br.get_value(state).unwrap_or(Cow::Borrowed(""));
        let l: &str = &bindings.l.get_value(state).unwrap_or(Cow::Borrowed(""));
        let r: &str = &bindings.r.get_value(state).unwrap_or(Cow::Borrowed(""));
        let t: &str = &bindings.t.get_value(state).unwrap_or(Cow::Borrowed(""));
        let b: &str = &bindings.b.get_value(state).unwrap_or(Cow::Borrowed(""));
        let l = if !l.is_empty() { l } else if !tl.is_empty() || !bl.is_empty() { " " } else { "" };
        let t = if !t.is_empty() { t } else if !tl.is_empty() || !tr.is_empty() { " " } else { "" };
        let r = if !r.is_empty() { r } else if !tr.is_empty() || !br.is_empty() { " " } else { "" };
        let b = if !b.is_empty() { b } else if !bl.is_empty() || !br.is_empty() { " " } else { "" };
        let fg = bindings.fg.get_value(state).unwrap_or(Color::White);
        let bg = bindings.bg.get_value(state).unwrap_or_default();
        let attr = bindings.attr.get_value(state).unwrap_or_default();
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

    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn DecoratorBindings> {
        let bg = Binding1::new(state, (), |(), bg| Some(bg));
        let fg = Binding1::new(state, (), |(), fg| Some(fg));
        let attr = Binding1::new(state, (), |(), attr| Some(attr));
        let tl = Binding1::new(state, (), |(), tl| Some(tl));
        let tr = Binding1::new(state, (), |(), tr| Some(tr));
        let bl = Binding1::new(state, (), |(), bl| Some(bl));
        let br = Binding1::new(state, (), |(), br| Some(br));
        let l = Binding1::new(state, (), |(), l| Some(l));
        let t = Binding1::new(state, (), |(), t| Some(t));
        let r = Binding1::new(state, (), |(), r| Some(r));
        let b = Binding1::new(state, (), |(), b| Some(b));
        bg.set_source_1(state, &mut ViewBase::BG.value_source(view.base()));
        fg.set_source_1(state, &mut ViewBase::FG.value_source(view.base()));
        attr.set_source_1(state, &mut ViewBase::ATTR.value_source(view.base()));
        tl.set_source_1(state, &mut BorderDecorator::TL.value_source(view.decorator()));
        tr.set_source_1(state, &mut BorderDecorator::TR.value_source(view.decorator()));
        bl.set_source_1(state, &mut BorderDecorator::BL.value_source(view.decorator()));
        br.set_source_1(state, &mut BorderDecorator::BR.value_source(view.decorator()));
        l.set_source_1(state, &mut BorderDecorator::L.value_source(view.decorator()));
        t.set_source_1(state, &mut BorderDecorator::T.value_source(view.decorator()));
        r.set_source_1(state, &mut BorderDecorator::R.value_source(view.decorator()));
        b.set_source_1(state, &mut BorderDecorator::B.value_source(view.decorator()));
        bg.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            view.invalidate_render(tree).expect("invalidate_render failed");
        });
        fg.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            view.invalidate_render(tree).expect("invalidate_render failed");
        });
        attr.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            view.invalidate_render(tree).expect("invalidate_render failed");
        });
        tl.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            view.invalidate_rect(tree, Rect {
                tl: Point { x: 0, y: 0 },
                size: Vector { x: 1, y: 1 }
            }).unwrap();
        });
        tr.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            let size = view.render_bounds(tree).size;
            view.invalidate_rect(tree, Rect {
                tl: Point { x: size.x.wrapping_sub(1), y: 0 },
                size: Vector { x: 1, y: 1 }
            }).unwrap();
        });
        bl.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            let size = view.render_bounds(tree).size;
            view.invalidate_rect(tree, Rect {
                tl: Point { x: 0, y: size.y.wrapping_sub(1) },
                size: Vector { x: 1, y: 1 }
            }).unwrap();
        });
        br.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            let size = view.render_bounds(tree).size;
            view.invalidate_rect(tree, Rect {
                tl: Point { x: size.x.wrapping_sub(1), y: size.y.wrapping_sub(1) },
                size: Vector { x: 1, y: 1 }
            }).unwrap();
        });
        l.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            let size = view.render_bounds(tree).size;
            view.invalidate_rect(tree, Rect {
                tl: Point { x: 0, y: 0 },
                size: Vector { x: 1, y: size.y }
            }).unwrap();
        });
        t.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            let size = view.render_bounds(tree).size;
            view.invalidate_rect(tree, Rect {
                tl: Point { x: 0, y: 0 },
                size: Vector { x: size.x, y: 1 }
            }).unwrap();
        });
        r.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            let size = view.render_bounds(tree).size;
            view.invalidate_rect(tree, Rect {
                tl: Point { x: size.x.wrapping_sub(1), y: 0 },
                size: Vector { x: 1, y: size.y }
            }).unwrap();
        });
        b.set_target_fn(state, view, |state, view, _| {
            let tree: &mut ViewTree = state.get_mut();
            let size = view.render_bounds(tree).size;
            view.invalidate_rect(tree, Rect {
                tl: Point { x: 0, y: size.y.wrapping_sub(1) },
                size: Vector { x: size.x, y: 1 }
            }).unwrap();
        });
        Box::new(BorderDecoratorBindings {
            bg: bg.into(),
            fg: fg.into(),
            attr: attr.into(),
            tl: tl.into(),
            tr: tr.into(),
            bl: bl.into(),
            br: br.into(),
            l: l.into(),
            t: t.into(),
            r: r.into(),
            b: b.into(),
        })
    }

    fn drop_bindings(&self, _view: View, state: &mut dyn State, bindings: Box<dyn DecoratorBindings>) {
        let bindings = bindings.downcast::<BorderDecoratorBindings>().unwrap();
        bindings.bg.drop_binding(state);
        bindings.fg.drop_binding(state);
        bindings.attr.drop_binding(state);
        bindings.tl.drop_binding(state);
        bindings.tr.drop_binding(state);
        bindings.bl.drop_binding(state);
        bindings.br.drop_binding(state);
        bindings.l.drop_binding(state);
        bindings.t.drop_binding(state);
        bindings.r.drop_binding(state);
        bindings.b.drop_binding(state);
    }
}
