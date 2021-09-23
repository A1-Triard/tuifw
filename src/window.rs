use crate::base::*;
use crate::view::{View, ViewAlign, ViewBase};
use crate::view::ViewBuilderViewAlignExt;
use crate::view::decorators::{BorderDecorator, LabelDecorator};
use crate::view::decorators::ViewBuilderBorderDecoratorExt;
use crate::view::decorators::ViewBuilderLabelDecoratorExt;
use crate::view::panels::{CanvasLayout, DockLayout, ViewBuilderDockPanelExt};
use dep_obj::{dep_type_with_builder, Change, Glob};
use dep_obj::binding::{Binding1, BindingExt2, b_continue, BYield};
use dyn_context::state::State;
use either::Right;
use std::borrow::Cow;
use tuifw_screen_base::*;

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct Window become obj in Widget {
        header: Cow<'static, str> = Cow::Borrowed(""),
        content: Option<Widget> = None,
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

        let load_content = BindingExt2::new(state, None, |
            state,
            content_cache: Glob<Option<Widget>>,
            view: Option<View>,
            content: Option<Change<Option<Widget>>>
        | -> BYield<!> {
            if let Some(content) = content {
                *content_cache.get_mut(state) = content.new;
                if let Some(view) = view {
                    if let Some(content) = content.new {
                        return content.load(state, view, |state, content_view| DockLayout::new(state, content_view));
                    }
                }
            } else {
                if let Some(view) = view {
                    if let Some(content) = *content_cache.get(state) {
                        return content.load(state, view, |state, content_view| DockLayout::new(state, content_view));
                    }
                }
            }
            b_continue()
        });
        widget.obj::<Window>().add_binding(state, load_content);
        load_content.set_source_2(state, &mut Window::CONTENT.change_initial_source(widget.obj()));
        load_content.set_source_1(state, &mut WidgetBase::VIEW.value_source(widget.base()));

        let unload_content = BindingExt2::new(state, None, |
            state,
            content_cache: Glob<Option<Widget>>,
            view: Option<View>,
            content: Option<Change<Option<Widget>>>
        | -> BYield<!> {
            if let Some(content) = content {
                *content_cache.get_mut(state) = content.new;
                if view.is_some() {
                    if let Some(content) = content.old {
                        return content.unload(state);
                    }
                }
            } else {
                if view.is_none() {
                    if let Some(content) = *content_cache.get(state) {
                        return content.unload(state);
                    }
                }
            }
            b_continue()
        });
        widget.obj::<Window>().add_binding(state, unload_content);
        unload_content.set_source_2(state, &mut Window::CONTENT.change_final_source(widget.obj()));
        unload_content.set_source_1(state, &mut WidgetBase::VIEW.value_source(widget.base()));
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
