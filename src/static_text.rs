use crate::base::*;
use crate::view::{View, ViewBase};
use crate::view::decorators::LabelDecorator;
use dep_obj::{dep_type_with_builder, Change};
use dep_obj::binding::Binding1;
use dyn_context::state::State;
use std::borrow::Cow;
use tuifw_screen_base::*;

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct StaticText become obj in Widget {
        text: Cow<'static, str> = Cow::Borrowed(""),
        bg: Option<Option<Color>> = None,
        fg: Option<Color> = None,
        attr: Option<Attr> = None,
    }

    type BaseBuilder<'a> = WidgetBuilder<'a>;
}

struct StaticTextBehavior;

impl WidgetBehavior for StaticTextBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State) {
        let init_new_view = Binding1::new(state, (), |(), change: Option<Change<Option<View>>>|
            change.and_then(|change| change.new)
        );
        init_new_view.set_target_fn(state, widget, |state, widget, view: View| {
            LabelDecorator::new(state, view);
            view.bind_base_to_widget_option(state, ViewBase::BG, widget, StaticText::BG, |x| x);
            view.bind_base_to_widget_option(state, ViewBase::FG, widget, StaticText::FG, |x| x);
            view.bind_base_to_widget_option(state, ViewBase::ATTR, widget, StaticText::ATTR, |x| x);
            view.bind_decorator_to_widget(state, LabelDecorator::TEXT, widget, StaticText::TEXT, |x| x);
        });
        widget.obj::<StaticText>().add_binding(state, init_new_view);
        init_new_view.set_source_1(state, &mut WidgetBase::VIEW.change_initial_source(widget.base()));
    }

    fn drop_bindings(&self, _widget: Widget, _state: &mut dyn State) { }
}

impl StaticText {
    const BEHAVIOR: StaticTextBehavior = StaticTextBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, StaticText::new_priv())
    }

    pub fn build<'a>(
        state: &'a mut dyn State,
        f: impl FnOnce(StaticTextBuilder<'a>) -> StaticTextBuilder<'a>
    ) -> Widget {
        let static_text = StaticText::new(state);
        f(StaticTextBuilder::new_priv(WidgetBuilder { widget: static_text, state }));
        static_text
    }
}

impl WidgetObj for StaticText {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
