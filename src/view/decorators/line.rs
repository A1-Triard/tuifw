use crate::view::base::*;
use dep_obj::{dep_type_with_builder, DepObjBuilderCore};
use dyn_context::{Context, ContextExt};
use std::borrow::Cow;
use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect, Orient};
use tuifw_window::{RenderPort};

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
        let tree: &mut ViewTree = self.context_mut().get_mut();
        LineDecorator::new(tree, view);
        f(LineDecoratorBuilder::new_priv(self)).core_priv()
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

    type BuilderCore<'a> = ViewBuilder<'a>;
}

impl LineDecorator {
    const BEHAVIOR: LineDecoratorBehavior = LineDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_decorator(tree, LineDecorator::new_priv());
        view.decorator(tree).on_changed(LineDecorator::ORIENT, Self::invalidate_measure);
        view.decorator(tree).on_changed(LineDecorator::LENGTH, Self::invalidate_measure);
        view.decorator(tree).on_changed(LineDecorator::NEAR, Self::invalidate_near);
        view.decorator(tree).on_changed(LineDecorator::STROKE, Self::invalidate_stroke);
        view.decorator(tree).on_changed(LineDecorator::FAR, Self::invalidate_far);
    }

    fn invalidate_measure<T>(context: &mut dyn Context, view: View, _old: &T) {
        let tree: &mut ViewTree = context.get_mut();
        view.invalidate_measure(tree);
    }

    fn invalidate_near(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let invalidated = Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: 1, y: 1 } };
        view.invalidate_rect(tree, invalidated).unwrap();
    }

    fn invalidate_stroke(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        view.invalidate_render(tree).unwrap();
    }

    fn invalidate_far(context: &mut dyn Context, view: View, _old: &Cow<'static, str>) {
        let tree: &mut ViewTree = context.get_mut();
        let &orient = view.decorator_ref(tree).get(LineDecorator::ORIENT);
        let size = view.render_bounds(tree).size;
        let invalidated = if orient == Orient::Vert {
            Rect { tl: Point { x: 0, y: size.y.wrapping_sub(1) }, size: Vector { x: 1, y: 1 } }
        } else {
            Rect { tl: Point { y: 0, x: size.x.wrapping_sub(1) }, size: Vector { x: 1, y: 1 } }
        };
        view.invalidate_rect(tree, invalidated).unwrap();
    }
}

impl Decorator for LineDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior { &Self::BEHAVIOR }
}

struct LineDecoratorBehavior;

impl DecoratorBehavior for LineDecoratorBehavior {
    fn children_measure_size(
        &self,
        _view: View,
        _tree: &mut ViewTree,
        _measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        (Some(0), Some(0))
    }

    fn desired_size(&self, view: View, tree: &mut ViewTree, _children_desired_size: Vector) -> Vector {
        let &orient = view.decorator_ref(tree).get(LineDecorator::ORIENT);
        let &length = view.decorator_ref(tree).get(LineDecorator::LENGTH);
        if orient == Orient::Vert {
            Vector {  x: 1, y: length }
        } else {
            Vector {  x: length, y: 1 }
        }
    }

    fn children_arrange_bounds(&self, _view: View, _tree: &mut ViewTree, _arrange_size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0}, size: Vector::null() }
    }

    fn render_bounds(&self, view: View, tree: &mut ViewTree, _children_render_bounds: Rect) -> Rect {
        let &orient = view.decorator_ref(tree).get(LineDecorator::ORIENT);
        let &length = view.decorator_ref(tree).get(LineDecorator::LENGTH);
        if orient == Orient::Vert {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: 1, y: length } }
        } else {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: length, y: 1 } }
        }
    }

    fn render(&self, view: View, tree: &ViewTree, port: &mut RenderPort) {
        let &orient = view.decorator_ref(tree).get(LineDecorator::ORIENT);
        let &length = view.decorator_ref(tree).get(LineDecorator::LENGTH);
        let near = view.decorator_ref(tree).get(LineDecorator::NEAR);
        let stroke = view.decorator_ref(tree).get(LineDecorator::STROKE);
        let far = view.decorator_ref(tree).get(LineDecorator::FAR);
        let fg = view.actual_fg(tree);
        let bg = view.actual_bg(tree);
        let attr = view.actual_attr(tree);
        if !stroke.is_empty() {
            for i in 0 .. length as u16 {
                if orient == Orient::Vert {
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
            if orient == Orient::Vert {
                port.out(Point { y: length.wrapping_sub(1), x: 0 }, fg, bg, attr, far);
            } else {
                port.out(Point { x: length.wrapping_sub(1), y: 0 }, fg, bg, attr, far);
            }
        }
    }
}
