use std::borrow::Cow;
use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect, Orient};
use tuifw_window::{RenderPort};
use dep_obj::{dep_obj, DepTypeToken};
use dyn_context::{Context, ContextExt};
use once_cell::sync::{self};
use crate::view::base::*;

pub trait ViewBuilderLineDecoratorExt {
    fn line_decorator(
        &mut self,
        f: impl FnOnce(&mut LineDecoratorBuilder) -> &mut LineDecoratorBuilder
    ) -> &mut Self;
}

impl<'a> ViewBuilderLineDecoratorExt for ViewBuilder<'a> {
    fn line_decorator(
        &mut self,
        f: impl FnOnce(&mut LineDecoratorBuilder) -> &mut LineDecoratorBuilder
    ) -> &mut Self {
        let mut builder = LineDecoratorBuilder::new_priv();
        f(&mut builder);
        let view = self.view();
        builder.build_priv(self.context(), view, line_decorator_type());
        self
    }
}

dep_obj! {
    #[derive(Debug)]
    pub struct LineDecorator become decorator in View {
        orient: Orient = Orient::Hor,
        length: i16 = 3,
        near: Cow<'static, str> = Cow::Borrowed(""),
        stroke: Cow<'static, str> = Cow::Borrowed(""),
        far: Cow<'static, str> = Cow::Borrowed(""),
    }
}

static LINE_DECORATOR_TOKEN: sync::Lazy<DepTypeToken<LineDecoratorType>> = sync::Lazy::new(||
    LineDecoratorType::new_priv().expect("LineDecoratorType builder locked")
);

pub fn line_decorator_type() -> &'static LineDecoratorType { LINE_DECORATOR_TOKEN.ty() }

impl LineDecorator {
    const BEHAVIOR: LineDecoratorBehavior = LineDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_decorator(tree, LineDecorator::new_priv(&LINE_DECORATOR_TOKEN));
        view.decorator_on_changed(tree, line_decorator_type().orient(), Self::invalidate_measure);
        view.decorator_on_changed(tree, line_decorator_type().length(), Self::invalidate_measure);
        view.decorator_on_changed(tree, line_decorator_type().near(), Self::invalidate_near);
        view.decorator_on_changed(tree, line_decorator_type().stroke(), Self::invalidate_stroke);
        view.decorator_on_changed(tree, line_decorator_type().far(), Self::invalidate_far);
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
        let &orient = view.decorator_get(tree, line_decorator_type().orient());
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
        let &orient = view.decorator_get(tree, line_decorator_type().orient());
        let &length = view.decorator_get(tree, line_decorator_type().length());
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
        let &orient = view.decorator_get(tree, line_decorator_type().orient());
        let &length = view.decorator_get(tree, line_decorator_type().length());
        if orient == Orient::Vert {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: 1, y: length } }
        } else {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: length, y: 1 } }
        }
    }

    fn render(&self, view: View, tree: &ViewTree, port: &mut RenderPort) {
        let &orient = view.decorator_get(tree, line_decorator_type().orient());
        let &length = view.decorator_get(tree, line_decorator_type().length());
        let near = view.decorator_get(tree, line_decorator_type().near());
        let stroke = view.decorator_get(tree, line_decorator_type().stroke());
        let far = view.decorator_get(tree, line_decorator_type().far());
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
