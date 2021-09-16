use crate::base::*;
use crate::view::{View, ViewAlign, ViewBase};
use crate::view::decorators::{ViewBuilderBorderDecoratorExt};
use crate::view::panels::CanvasLayout;
use dep_obj::{dep_type, Change};
use dep_obj::binding::{Binding1, Binding2};
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
        init_new_view.set_target_fn(state, (), |state, (), view: View| view.build(state, |view| view
            .border_decorator(|view| view
                .tl(Cow::Borrowed("╔"))
                .tr(Cow::Borrowed("╗"))
                .bl(Cow::Borrowed("╚"))
                .br(Cow::Borrowed("╝"))
                .l(Cow::Borrowed("║"))
                .t(Cow::Borrowed("═"))
                .r(Cow::Borrowed("║"))
                .b(Cow::Borrowed("═"))
                .fill(Cow::Borrowed(" "))
            )
        ));
        init_new_view.set_source_1(state, &mut WidgetBase::VIEW.change_initial_source(widget.base()));
        widget.obj::<Window>().add_binding(state, init_new_view);

        let bg = Binding2::new(state, (), |(), bg: Option<Color>, view: Option<View>| view.map(|view| (bg, view)));
        bg.dispatch(state, (), |state, (), (bg, view)| ViewBase::BG.set(state, view.base(), bg));
        bg.set_source_1(state, &mut Window::BG.value_source(widget.obj()));
        bg.set_source_2(state, &mut WidgetBase::VIEW.value_source(widget.base()));
        widget.obj::<Window>().add_binding(state, bg);

        let tl = Binding2::new(state, (), |(), bounds: Rect, view: Option<View>| view.map(|view| (bounds.tl, view)));
        tl.dispatch(state, (), |state, (), (tl, view)| CanvasLayout::TL.set(state, view.layout(), tl));
        tl.set_source_1(state, &mut Window::BOUNDS.value_source(widget.obj()));
        tl.set_source_2(state, &mut WidgetBase::VIEW.value_source(widget.base()));
        widget.obj::<Window>().add_binding(state, tl);

        let w = Binding2::new(state, (), |(), bounds: Rect, view: Option<View>| view.map(|view| (bounds.w(), view)));
        w.dispatch(state, (), |state, (), (w, view)| ViewAlign::W.set(state, view.align(), Some(w)));
        w.set_source_1(state, &mut Window::BOUNDS.value_source(widget.obj()));
        w.set_source_2(state, &mut WidgetBase::VIEW.value_source(widget.base()));
        widget.obj::<Window>().add_binding(state, w);

        let h = Binding2::new(state, (), |(), bounds: Rect, view: Option<View>| view.map(|view| (bounds.h(), view)));
        h.dispatch(state, (), |state, (), (h, view)| ViewAlign::H.set(state, view.align(), Some(h)));
        h.set_source_1(state, &mut Window::BOUNDS.value_source(widget.obj()));
        h.set_source_2(state, &mut WidgetBase::VIEW.value_source(widget.base()));
        widget.obj::<Window>().add_binding(state, h);
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
