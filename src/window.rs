use crate::base::*;
use crate::view::{View, ViewAlign, ViewBase};
use crate::view::ViewBuilderViewAlignExt;
use crate::view::decorators::{BorderDecorator, LabelDecorator};
use crate::view::decorators::ViewBuilderBorderDecoratorExt;
use crate::view::decorators::ViewBuilderLabelDecoratorExt;
use crate::view::panels::{CanvasLayout, ViewBuilderDockPanelExt};
use dep_obj::{dep_type_with_builder, Change};
use dep_obj::binding::{Binding1};
use dyn_context::state::State;
use either::Right;
use std::borrow::Cow;
use tuifw_screen_base::*;

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct Window become obj in Widget {
        header: Cow<'static, str> = Cow::Borrowed(""),
        bg: Option<Color> = Some(Color::Blue),
        bounds: Rect = Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: 0, y: 0 } },
    }

    type BaseBuilder<'a> = WidgetBuilder<'a>;
}

struct WindowBehavior;

impl WidgetBehavior for WindowBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State) {
        let init_new_view = Binding1::new(state, (), |(), change: Option<Change<Option<View>>>|
            change.and_then(|change| change.new)
        );
        init_new_view.set_target_fn(state, widget, |state, widget, view: View| {
            let mut header = None;
            view.build(state, |view| view
                .border_decorator(|decorator| decorator
                    .fill(Cow::Borrowed(" "))
                    .enable_t_padding(false)
                )
                .dock_panel(|panel| panel
                    .child(Some(&mut header), (), |layout| layout.dock(Right(Side::Top)), |child| child
                        .label_decorator(|decorator| decorator)
                        .align(|align| align.h_align(HAlign::Center))
                    )
                )
            );
            header.unwrap().bind_decorator_to_widget(state, LabelDecorator::TEXT, widget, Window::HEADER, |x| x);
            view.bind_base_to_widget(state, ViewBase::BG, widget, Window::BG, |x| x);
            view.bind_layout_to_widget(state, CanvasLayout::TL, widget, Window::BOUNDS, |x| x.tl);
            view.bind_align_to_widget(state, ViewAlign::W, widget, Window::BOUNDS, |x| Some(x.w()));
            view.bind_align_to_widget(state, ViewAlign::H, widget, Window::BOUNDS, |x| Some(x.h()));
            view.bind_decorator_to_base(state, BorderDecorator::TL, ViewBase::IS_FOCUSED, |focused|
                Cow::Borrowed(if focused { "╔" } else { "┌" })
            );
            view.bind_decorator_to_base(state, BorderDecorator::TR, ViewBase::IS_FOCUSED, |focused|
                Cow::Borrowed(if focused { "╗" } else { "┐" })
            );
            view.bind_decorator_to_base(state, BorderDecorator::BL, ViewBase::IS_FOCUSED, |focused|
                Cow::Borrowed(if focused { "╚" } else { "└" })
            );
            view.bind_decorator_to_base(state, BorderDecorator::BR, ViewBase::IS_FOCUSED, |focused|
                Cow::Borrowed(if focused { "╝" } else { "┘" })
            );
            view.bind_decorator_to_base(state, BorderDecorator::L, ViewBase::IS_FOCUSED, |focused|
                Cow::Borrowed(if focused { "║" } else { "│" })
            );
            view.bind_decorator_to_base(state, BorderDecorator::T, ViewBase::IS_FOCUSED, |focused|
                Cow::Borrowed(if focused { "═" } else { "─" })
            );
            view.bind_decorator_to_base(state, BorderDecorator::R, ViewBase::IS_FOCUSED, |focused|
                Cow::Borrowed(if focused { "║" } else { "│" })
            );
            view.bind_decorator_to_base(state, BorderDecorator::B, ViewBase::IS_FOCUSED, |focused|
                Cow::Borrowed(if focused { "═" } else { "─" })
            );
        });
        widget.obj::<Window>().add_binding(state, init_new_view);
        init_new_view.set_source_1(state, &mut WidgetBase::VIEW.change_initial_source(widget.base()));
    }

    fn drop_bindings(&self, _widget: Widget, _state: &mut dyn State) { }
}

impl Window {
    const BEHAVIOR: WindowBehavior = WindowBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, Window::new_priv())
    }

    pub fn build<'a>(
        state: &'a mut dyn State,
        f: impl FnOnce(WindowBuilder<'a>) -> WindowBuilder<'a>
    ) -> Widget {
        let window = Window::new(state);
        f(WindowBuilder::new_priv(WidgetBuilder { widget: window, state }));
        window
    }
}

impl WidgetObj for Window {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
