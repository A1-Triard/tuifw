use crate::view::base::*;
use dep_obj::{DepObjBaseBuilder, dep_type_with_builder};
use dep_obj::binding::{Binding, Binding1, Binding2};
use dyn_context::state::{State, StateExt};
use std::borrow::Cow;
use std::fmt::Debug;
use tuifw_screen_base::{Attr, Color, Orient, Point, Rect, Vector};
use tuifw_window::RenderPort;

pub trait ViewBuilderLineDecoratorExt {
    fn line_decorator(
        self,
        f: impl for<'a> FnOnce(LineDecoratorBuilder<'a>) -> LineDecoratorBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderLineDecoratorExt for ViewBuilder<'a> {
    fn line_decorator(
        mut self,
        f: impl for<'b> FnOnce(LineDecoratorBuilder<'b>) -> LineDecoratorBuilder<'b>
    ) -> Self {
        let view = self.id();
        LineDecorator::new(self.state_mut(), view);
        f(LineDecoratorBuilder::new_priv(self)).base_priv()
    }
}

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct LineDecorator become decorator in View {
        orient: Orient = Orient::Hor,
        length: i16 = 3,
        near: Cow<'static, str> = Cow::Borrowed(""),
        stroke: Cow<'static, str> = Cow::Borrowed(""),
        far: Cow<'static, str> = Cow::Borrowed(""),
    }

    type BaseBuilder<'a> = ViewBuilder<'a>;
}

impl LineDecorator {
    const BEHAVIOR: LineDecoratorBehavior = LineDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        state: &mut dyn State,
        view: View,
    ) {
        view.set_decorator(state, LineDecorator::new_priv());
    }
}

impl Decorator for LineDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior { &Self::BEHAVIOR }
}

#[derive(Debug)]
struct LineDecoratorBindings {
    fg: Binding<Color>,
    bg: Binding<Option<Color>>,
    attr: Binding<Attr>,
    length: Binding<i16>,
    near: Binding<Cow<'static, str>>,
    stroke: Binding<Cow<'static, str>>,
    orient: Binding<Orient>,
    far_orient: Binding<(Cow<'static, str>, Orient)>,
}

impl DecoratorBindings for LineDecoratorBindings { }

struct LineDecoratorBehavior;

impl DecoratorBehavior for LineDecoratorBehavior {
    fn ty(&self) -> &'static str { "Line" }

    fn children_measure_size(
        &self,
        _view: View,
        _state: &mut dyn State,
        _measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        (Some(0), Some(0))
    }

    fn desired_size(&self, view: View, state: &mut dyn State, _children_desired_size: Vector) -> Vector {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<LineDecoratorBindings>().unwrap();
        let length = bindings.length.get_value(state).unwrap_or(3);
        if bindings.orient.get_value(state).unwrap_or(Orient::Hor) == Orient::Vert {
            Vector {  x: 1, y: length }
        } else {
            Vector {  x: length, y: 1 }
        }
    }

    fn children_arrange_bounds(&self, _view: View, _state: &mut dyn State, _arrange_size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0}, size: Vector::null() }
    }

    fn render_bounds(&self, view: View, state: &mut dyn State, _children_render_bounds: Rect) -> Rect {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<LineDecoratorBindings>().unwrap();
        let length = bindings.length.get_value(state).unwrap_or(3);
        if bindings.orient.get_value(state).unwrap_or(Orient::Hor) == Orient::Vert {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: 1, y: length } }
        } else {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: length, y: 1 } }
        }
    }

    fn render(&self, view: View, state: &dyn State, port: &mut RenderPort) {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<LineDecoratorBindings>().unwrap();
        let fg = bindings.fg.get_value(state).unwrap_or(Color::White);
        let bg = bindings.bg.get_value(state).unwrap_or_default();
        let attr = bindings.attr.get_value(state).unwrap_or_default();
        let stroke = &bindings.stroke.get_value(state).unwrap_or(Cow::Borrowed(""));
        let near = &bindings.near.get_value(state).unwrap_or(Cow::Borrowed(""));
        let far = &bindings.far_orient.get_value(state).map_or(Cow::Borrowed(""), |x| x.0);
        let length = bindings.length.get_value(state).unwrap_or(3);
        if !bindings.stroke.get_value(state).map_or(true, |x| x.is_empty()) {
            for i in 0 .. length as u16 {
                if bindings.orient.get_value(state).unwrap_or(Orient::Hor) == Orient::Vert {
                    port.out(Point { x: 0, y: i as i16 }, fg, bg, attr, stroke);
                } else {
                    port.out(Point { y: 0, x: i as i16 }, fg, bg, attr, stroke);
                }
            }
        }
        if !near.is_empty() {
            port.out(Point { x: 0, y: 0 }, fg, bg, attr, near);
        }
        if !far.is_empty() {
            if bindings.orient.get_value(state).unwrap_or(Orient::Hor) == Orient::Vert {
                port.out(Point { y: length.wrapping_sub(1), x: 0 }, fg, bg, attr, far);
            } else {
                port.out(Point { x: length.wrapping_sub(1), y: 0 }, fg, bg, attr, far);
            }
        }
    }

    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn DecoratorBindings> {
        let fg = Binding1::new(state, (), |(), fg| Some(fg));
        let bg = Binding1::new(state, (), |(), bg| Some(bg));
        let attr = Binding1::new(state, (), |(), attr| Some(attr));
        let length = Binding1::new(state, (), |(), length| Some(length));
        let near = Binding1::new(state, (), |(), near| Some(near));
        let stroke = Binding1::new(state, (), |(), stroke| Some(stroke));
        let orient = Binding1::new(state, (), |(), orient| Some(orient));
        let far_orient = Binding2::new(state, (), |(), far, orient| Some((far, orient)));
        bg.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        fg.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        attr.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        length.set_target_fn(state, view, |state, view, _| view.invalidate_measure(state));
        near.set_target_fn(state, view, |state, view, _| view.invalidate_rect(state, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: 1 }
        }));
        stroke.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        orient.set_target_fn(state, view, |state, view, _| view.invalidate_measure(state));
        far_orient.set_target_fn(state, view, |state, view, (_far, orient)| {
            let tree: &ViewTree = state.get();
            let size = view.render_bounds(tree).size;
            let invalidated = if orient == Orient::Vert {
                Rect { tl: Point { x: 0, y: size.y.wrapping_sub(1) }, size: Vector { x: 1, y: 1 } }
            } else {
                Rect { tl: Point { y: 0, x: size.x.wrapping_sub(1) }, size: Vector { x: 1, y: 1 } }
            };
            view.invalidate_rect(state, invalidated);
        });
        bg.set_source_1(state, &mut ViewBase::BG.value_source(view.base()));
        fg.set_source_1(state, &mut ViewBase::FG.value_source(view.base()));
        attr.set_source_1(state, &mut ViewBase::ATTR.value_source(view.base()));
        length.set_source_1(state, &mut LineDecorator::LENGTH.value_source(view.decorator()));
        near.set_source_1(state, &mut LineDecorator::NEAR.value_source(view.decorator()));
        stroke.set_source_1(state, &mut LineDecorator::STROKE.value_source(view.decorator()));
        orient.set_source_1(state, &mut LineDecorator::ORIENT.value_source(view.decorator()));
        far_orient.set_source_1(state, &mut LineDecorator::FAR.value_source(view.decorator()));
        far_orient.set_source_2(state, &mut LineDecorator::ORIENT.value_source(view.decorator()));
        Box::new(LineDecoratorBindings {
            fg: fg.into(),
            bg: bg.into(),
            attr: attr.into(),
            length: length.into(),
            near: near.into(),
            stroke: stroke.into(),
            orient: orient.into(),
            far_orient: far_orient.into(),
        })
    }

    fn drop_bindings(&self, _view: View, state: &mut dyn State, bindings: Box<dyn DecoratorBindings>) {
        let bindings = bindings.downcast::<LineDecoratorBindings>().unwrap();
        bindings.bg.drop_binding(state);
        bindings.fg.drop_binding(state);
        bindings.attr.drop_binding(state);
        bindings.length.drop_binding(state);
        bindings.near.drop_binding(state);
        bindings.stroke.drop_binding(state);
        bindings.orient.drop_binding(state);
        bindings.far_orient.drop_binding(state);
    }
}
