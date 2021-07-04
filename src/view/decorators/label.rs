use crate::view::base::*;
use dep_obj::{dep_type_with_builder, DepObjBuilderCore};
use dyn_context::{Context, ContextExt};
use std::borrow::{Borrow, Cow};
use std::fmt::Debug;
use std::num::NonZeroI16;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

pub trait ViewBuilderLabelDecoratorExt {
    fn label_decorator(
        self,
        f: impl for<'a> FnOnce(LabelDecoratorBuilder<'a>) -> LabelDecoratorBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderLabelDecoratorExt for ViewBuilder<'a> {
    fn label_decorator(
        mut self,
        f: impl for<'b> FnOnce(LabelDecoratorBuilder<'b>) -> LabelDecoratorBuilder<'b>
    ) -> Self {
        let view = self.id();
        let tree: &mut ViewTree = self.context_mut().get_mut();
        LabelDecorator::new(tree, view);
        f(LabelDecoratorBuilder::new_priv(self)).core_priv()
    }
}

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct LabelDecorator become decorator in View {
        text: Cow<'static, str> = Cow::Borrowed(""),
    }

    type BuilderCore<'a> = ViewBuilder<'a>;
}

impl LabelDecorator {
    const BEHAVIOR: LabelDecoratorBehavior = LabelDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_decorator(tree, LabelDecorator::new_priv());
        view.decorator(tree).on_changed(LabelDecorator::TEXT, Self::invalidate_measure);
    }

    fn invalidate_measure<T>(context: &mut dyn Context, view: View, _old: &T) {
        let tree: &mut ViewTree = context.get_mut();
        view.invalidate_measure(tree);
    }
}

impl Decorator for LabelDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior { &Self::BEHAVIOR }
}

struct LabelDecoratorBehavior;

impl DecoratorBehavior for LabelDecoratorBehavior {
    fn children_measure_size(
        &self,
        _view: View,
        _tree: &mut ViewTree,
        _measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        (Some(0), Some(0))
    }

    fn desired_size(&self, view: View, tree: &mut ViewTree, _children_desired_size: Vector) -> Vector {
        let text: &str = view.decorator_ref(tree).get(LabelDecorator::TEXT).borrow();
        let width = text
            .graphemes(true)
            .map(|g| g
                .chars()
                .find_map(|c| NonZeroI16::new(c.width().unwrap_or(1) as u16 as i16))
                .map_or(0, |x| x.get())
            )
            .sum();
        Vector { x: width, y: 1 }
    }

    fn children_arrange_bounds(&self, _view: View, _tree: &mut ViewTree, _arrange_size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0}, size: Vector::null() }
    }

    fn render_bounds(&self, view: View, tree: &mut ViewTree, _children_render_bounds: Rect) -> Rect {
        let text: &str = view.decorator_ref(tree).get(LabelDecorator::TEXT).borrow();
        let width = text
            .graphemes(true)
            .map(|g| g
                .chars()
                .find_map(|c| NonZeroI16::new(c.width().unwrap_or(1) as u16 as i16))
                .map_or(0, |x| x.get())
            )
            .sum();
        Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: width, y: 1 } }
    }

    fn render(&self, view: View, tree: &ViewTree, port: &mut RenderPort) {
        let text: &str = view.decorator_ref(tree).get(LabelDecorator::TEXT).borrow();
        let fg = view.actual_fg(tree);
        let bg = view.actual_bg(tree);
        let attr = view.actual_attr(tree);
        port.out(Point { y: 0, x: 0 }, fg, bg, attr, text);
    }
}
