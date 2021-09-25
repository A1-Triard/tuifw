use crate::view::base::*;
use alloc::boxed::Box;
use dep_obj::{DepObjBaseBuilder, dep_type_with_builder};
use dep_obj::binding::{Binding, Binding1};
use dyn_context::state::{State, StateExt};
use alloc::borrow::Cow;
use core::fmt::Debug;
use core::num::NonZeroI16;
use tuifw_screen_base::{Attr, Color, Point, Rect, Vector};
use tuifw_window::RenderPort;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

pub trait ViewBuilderTextDecoratorExt {
    fn text_decorator(
        self,
        f: impl for<'a> FnOnce(TextDecoratorBuilder<'a>) -> TextDecoratorBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderTextDecoratorExt for ViewBuilder<'a> {
    fn text_decorator(
        mut self,
        f: impl for<'b> FnOnce(TextDecoratorBuilder<'b>) -> TextDecoratorBuilder<'b>
    ) -> Self {
        let view = self.id();
        TextDecorator::new(self.state_mut(), view);
        f(TextDecoratorBuilder::new_priv(self)).base_priv()
    }
}

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct TextDecorator become decorator in View {
        text: Cow<'static, str> = Cow::Borrowed(""),
    }

    type BaseBuilder<'a> = ViewBuilder<'a>;
}

impl TextDecorator {
    const BEHAVIOR: TextDecoratorBehavior = TextDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        state: &mut dyn State,
        view: View,
    ) {
        view.set_decorator(state, TextDecorator::new_priv());
    }
}

impl Decorator for TextDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior { &Self::BEHAVIOR }
}

#[derive(Debug)]
struct TextDecoratorBindings {
    fg: Binding<Color>,
    bg: Binding<Option<Color>>,
    attr: Binding<Attr>,
    text: Binding<Cow<'static, str>>,
}

impl DecoratorBindings for TextDecoratorBindings { }

struct TextDecoratorBehavior;

impl DecoratorBehavior for TextDecoratorBehavior {
    fn ty(&self) -> &'static str { "Text" }

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
        let bindings = view.decorator_bindings(tree).downcast_ref::<TextDecoratorBindings>().unwrap();
        let width = bindings.text.get_value(state).unwrap_or(Cow::Borrowed(""))
            .graphemes(true)
            .map(|g| g
                .chars()
                .find_map(|c| NonZeroI16::new(c.width().unwrap_or(1) as u16 as i16))
                .map_or(0, |x| x.get())
            )
            .sum();
        Vector { x: width, y: 1 }
    }

    fn children_arrange_bounds(&self, _view: View, _state: &mut dyn State, _arrange_size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0}, size: Vector::null() }
    }

    fn render_bounds(&self, view: View, state: &mut dyn State, _children_render_bounds: Rect) -> Rect {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<TextDecoratorBindings>().unwrap();
        let width = bindings.text.get_value(state).unwrap_or(Cow::Borrowed(""))
            .graphemes(true)
            .map(|g| g
                .chars()
                .find_map(|c| NonZeroI16::new(c.width().unwrap_or(1) as u16 as i16))
                .map_or(0, |x| x.get())
            )
            .sum();
        Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: width, y: 1 } }
    }

    fn render(&self, view: View, state: &dyn State, port: &mut RenderPort) {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<TextDecoratorBindings>().unwrap();
        let fg = bindings.fg.get_value(state).unwrap_or(Color::White);
        let bg = bindings.bg.get_value(state).unwrap_or_default();
        let attr = bindings.attr.get_value(state).unwrap_or_default();
        let text = &bindings.text.get_value(state).unwrap_or(Cow::Borrowed(""));
        port.out(Point { y: 0, x: 0 }, fg, bg, attr, text);
    }

    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn DecoratorBindings> {
        let fg = Binding1::new(state, (), |(), fg| Some(fg));
        let bg = Binding1::new(state, (), |(), bg| Some(bg));
        let attr = Binding1::new(state, (), |(), attr| Some(attr));
        let text = Binding1::new(state, (), |(), text| Some(text));
        bg.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        fg.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        attr.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        text.set_target_fn(state, view, |state, view, _| view.invalidate_measure_and_render(state));
        bg.set_source_1(state, &mut ViewBase::BG.value_source(view.base()));
        fg.set_source_1(state, &mut ViewBase::FG.value_source(view.base()));
        attr.set_source_1(state, &mut ViewBase::ATTR.value_source(view.base()));
        text.set_source_1(state, &mut TextDecorator::TEXT.value_source(view.decorator()));
        Box::new(TextDecoratorBindings {
            bg: bg.into(),
            fg: fg.into(),
            attr: attr.into(),
            text: text.into(),
        })
    }

    fn drop_bindings(&self, _view: View, state: &mut dyn State, bindings: Box<dyn DecoratorBindings>) {
        let bindings = bindings.downcast::<TextDecoratorBindings>().unwrap();
        bindings.bg.drop_binding(state);
        bindings.fg.drop_binding(state);
        bindings.attr.drop_binding(state);
        bindings.text.drop_binding(state);
    }
}
