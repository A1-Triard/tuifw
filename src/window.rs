use crate::base::*;
use crate::view::{View, ViewAlign, ViewBase};
use crate::view::decorators::{BorderDecorator, ViewBuilderBorderDecoratorExt};
use crate::view::panels::CanvasLayout;
use dep_obj::{dep_type, Change};
use dep_obj::binding::{Binding1};
use dyn_context::state::State;
use tuifw_screen_base::*;
use std::borrow::Cow;

dep_type! {
    #[derive(Debug)]
    pub struct Window in Widget {
        bg: Option<Color> = Some(Color::Blue),
        bounds: Rect = Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: 0, y: 0 } },
    }
}

struct WindowBehavior;

impl WidgetBehavior for WindowBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State) {
        let init_new_view = Binding1::new(state, (), |(), change: Option<Change<Option<View>>>|
            change.and_then(|change| change.new)
        );
        init_new_view.set_target_fn(state, (), |state, (), view: View| {
            view.build(state, |view| view
                .border_decorator(|decorator| decorator
                    .fill(Cow::Borrowed(" "))
                )
            );
            view.bind_decorator(state, ViewBase::IS_FOCUSED, BorderDecorator::TL, |focused|
                Cow::Borrowed(if focused { "╔" } else { "┌" })
            );
            view.bind_decorator(state, ViewBase::IS_FOCUSED, BorderDecorator::TR, |focused|
                Cow::Borrowed(if focused { "╗" } else { "┐" })
            );
            view.bind_decorator(state, ViewBase::IS_FOCUSED, BorderDecorator::BL, |focused|
                Cow::Borrowed(if focused { "╚" } else { "└" })
            );
            view.bind_decorator(state, ViewBase::IS_FOCUSED, BorderDecorator::BR, |focused|
                Cow::Borrowed(if focused { "╝" } else { "┘" })
            );
            view.bind_decorator(state, ViewBase::IS_FOCUSED, BorderDecorator::L, |focused|
                Cow::Borrowed(if focused { "║" } else { "│" })
            );
            view.bind_decorator(state, ViewBase::IS_FOCUSED, BorderDecorator::T, |focused|
                Cow::Borrowed(if focused { "═" } else { "─" })
            );
            view.bind_decorator(state, ViewBase::IS_FOCUSED, BorderDecorator::R, |focused|
                Cow::Borrowed(if focused { "║" } else { "│" })
            );
            view.bind_decorator(state, ViewBase::IS_FOCUSED, BorderDecorator::B, |focused|
                Cow::Borrowed(if focused { "═" } else { "─" })
            );
        });
        init_new_view.set_source_1(state, &mut WidgetBase::VIEW.change_initial_source(widget.base()));
        widget.obj::<Window>().add_binding(state, init_new_view);

        widget.bind_base(state, Window::BG, ViewBase::BG, |x| x);
        widget.bind_layout(state, Window::BOUNDS, CanvasLayout::TL, |x| x.tl);
        widget.bind_align(state, Window::BOUNDS, ViewAlign::W, |x| Some(x.w()));
        widget.bind_align(state, Window::BOUNDS, ViewAlign::H, |x| Some(x.h()));
    }

    fn drop_bindings(&self, _widget: Widget, _state: &mut dyn State) { }
}

impl Window {
    const BEHAVIOR: WindowBehavior = WindowBehavior;

    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, Window::new_priv())
    }
}

impl WidgetObj for Window {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
