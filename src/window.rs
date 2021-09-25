use crate::base::*;
use crate::view::{View, ViewAlign, ViewBase, ViewTree};
use crate::view::decorators::{BorderDecorator, TextDecorator};
use crate::view::decorators::ViewBuilderBorderDecoratorExt;
use crate::view::decorators::ViewBuilderTextDecoratorExt;
use crate::view::panels::{CanvasLayout, DockLayout, ViewBuilderDockPanelExt};
use dep_obj::{DepObjBaseBuilder, dep_type_with_builder, Change, Glob};
use dep_obj::binding::{BYield, Binding1, BindingExt3, b_continue, b_immediate};
use dyn_context::state::{State, StateExt};
use either::Right;
use alloc::borrow::Cow;
use tuifw_screen_base::*;

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct Window become obj in Widget {
        header: Cow<'static, str> = Cow::Borrowed(""),
        #[ref]
        content: Option<Widget> = None,
        bg: Option<Color> = Some(Color::Blue),
        bounds: Rect = Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: 0, y: 0 } },
    }

    type BaseBuilder<'a> = WidgetBuilder<'a>;
}

impl<'a> WindowBuilder<'a> {
    pub fn content<T: WidgetObjWithBuilder, F: for<'b> FnOnce(T::Builder<'b>)>(
        mut self,
        storage: Option<&mut Option<Widget>>,
        f: F
    ) -> Self {
        let window = self.base_priv_ref().id();
        let content = T::build(self.base_priv_mut().state_mut(), f);
        storage.map(|x| x.replace(content));
        b_immediate(Window::CONTENT.set(self.base_priv_mut().state_mut(), window.obj(), Some(content)));
        self
    }
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
                        .text_decorator(|decorator| decorator)
                    )
                )
            );
            header.unwrap().bind_decorator_to_widget(state, TextDecorator::TEXT, widget, Window::HEADER, |x| x);
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

        let content = BindingExt3::new(state, (None, None), |
            state,
            content_cache: Glob<(Option<Widget>, Option<View>)>,
            view: Option<View>,
            new_content: Option<Change<Option<Widget>>>,
            old_content: Option<Change<Option<Widget>>>,
        | -> BYield<!> {
            if let Some(content) = new_content {
                content_cache.get_mut(state).0 = content.new;
                if let Some(view) = view {
                    if let Some(content) = content.new {
                        let view_tree: &ViewTree = state.get();
                        let prev = view.last_child(view_tree);
                        return content.load(state, view, prev, |state, content_view|
                            DockLayout::new(state, content_view)
                        );
                    } else {
                        let view_tree: &ViewTree = state.get();
                        let prev = view.last_child(view_tree);
                        let content_view = View::new(state, view, prev);
                        DockLayout::new(state, content_view);
                        let ok = content_cache.get_mut(state).1.replace(content_view).is_none();
                        debug_assert!(ok);
                    }
                }
            } else if let Some(content) = old_content {
                if view.is_some() {
                    if let Some(content) = content.old {
                        return content.unload(state);
                    } else {
                        content_cache.get_mut(state).1.take().unwrap().drop_view(state);
                    }
                }
            } else {
                if let Some(content) = content_cache.get(state).0 {
                    if let Some(view) = view {
                        let view_tree: &ViewTree = state.get();
                        let prev = view.last_child(view_tree);
                        return content.load(state, view, prev, |state, content_view|
                            DockLayout::new(state, content_view)
                        );
                    } else {
                        return content.unload(state);
                    }
                } else {
                    if let Some(view) = view {
                        let view_tree: &ViewTree = state.get();
                        let prev = view.last_child(view_tree);
                        let content_view = View::new(state, view, prev);
                        DockLayout::new(state, content_view);
                        let ok = content_cache.get_mut(state).1.replace(content_view).is_none();
                        debug_assert!(ok);
                    } else {
                        content_cache.get_mut(state).1.take().unwrap().drop_view(state);
                    }
                }
            }
            b_continue()
        });
        widget.obj::<Window>().add_binding(state, content);
        content.set_source_1(state, &mut WidgetBase::VIEW.value_source(widget.base()));
        content.set_source_2(state, &mut Window::CONTENT.change_initial_source(widget.obj()));
        content.set_source_3(state, &mut Window::CONTENT.change_final_source(widget.obj()));
    }

    fn drop_bindings(&self, _widget: Widget, _state: &mut dyn State) { }
}

impl Window {
    const BEHAVIOR: WindowBehavior = WindowBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, Window::new_priv())
    }
}

impl WidgetObjWithBuilder for Window {
    type Builder<'a> = WindowBuilder<'a>;

    fn build<'a>(
        state: &'a mut dyn State,
        f: impl FnOnce(WindowBuilder<'a>)
    ) -> Widget {
        let window = Window::new(state);
        f(WindowBuilder::new_priv(WidgetBuilder { widget: window, state }));
        window
    }
}

impl WidgetObj for Window {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
