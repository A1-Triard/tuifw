use crate::base::*;
use crate::view::{View, ViewBase};
use crate::view::decorators::TextDecorator;
use dep_obj::{Builder, Change, DepObjId, dep_type, ext_builder};
use dep_obj::binding::Binding1;
use dyn_context::State;
use alloc::borrow::Cow;
use tuifw_screen_base::*;

dep_type! {
    #[derive(Debug)]
    pub struct StaticText = Widget[WidgetObjKey] {
        text: Cow<'static, str> = Cow::Borrowed(""),
        bg: Option<Option<Color>> = None,
        fg: Option<Color> = None,
        attr: Option<Attr> = None,
    }
}

ext_builder!(<'a> Builder<'a, Widget> as BuilderWidgetStaticTextExt[Widget] {
    static_text -> (StaticText)
});

struct StaticTextBehavior;

impl WidgetBehavior for StaticTextBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State) {
        let init_new_view = Binding1::new(state, (), |(), change: Option<Change<Option<View>>>|
            change.and_then(|change| change.new)
        );
        init_new_view.set_target_fn(state, widget, |state, widget, view: View| {
            TextDecorator::new(state, view);
            view.bind_base_to_widget_option(state, ViewBase::BG, widget, StaticText::BG, |x| x);
            view.bind_base_to_widget_option(state, ViewBase::FG, widget, StaticText::FG, |x| x);
            view.bind_base_to_widget_option(state, ViewBase::ATTR, widget, StaticText::ATTR, |x| x);
            view.bind_decorator_to_widget(state, TextDecorator::TEXT, widget, StaticText::TEXT, |x| x);
        });
        widget.add_binding::<StaticText, _>(state, init_new_view);
        init_new_view.set_source_1(state, &mut WidgetBase::VIEW.change_initial_source(widget));
    }

    fn drop_bindings(&self, _widget: Widget, _state: &mut dyn State) { }
}

impl StaticText {
    const BEHAVIOR: StaticTextBehavior = StaticTextBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, StaticText::new_priv())
    }
}

/*
impl<B: DepObjBuilder<Id=Widget>> WidgetObjWithBuilder<B> for StaticText {
    type Builder = StaticTextBuilder<B>;

    fn build<'a>(
        state: &'a mut dyn State,
        f: impl FnOnce(StaticTextBuilder<B>)
    ) -> Widget {
        let static_text = StaticText::new(state);
        f(StaticTextBuilder::new_priv(Builder { id: static_text, state }));
        static_text
    }
}
*/

impl WidgetObj for StaticText {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
