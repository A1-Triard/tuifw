use crate::base::{WidgetTemplate, Widget, WidgetObj, WidgetBehavior};
use crate::view::View;
use crate::view::decorators::LabelDecorator;
use dep_obj::{dep_type, Style};
use dep_obj::binding::Binding1;
use dyn_context::state::State;
use std::borrow::Cow;

dep_type! {
    #[derive(Debug)]
    pub struct Label in Widget {
        text: Cow<'static, str> = Cow::Borrowed(""),
    }
}

impl Label {
    const BEHAVIOR: LabelBehavior = LabelBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, Label::new_priv())
    }

    pub fn template(style: Style<Label>) -> Box<dyn WidgetTemplate> {
        Box::new(LabelTemplate(style))
    }
}

#[derive(Debug, Clone)]
struct LabelTemplate(Style<Label>);

impl WidgetTemplate for LabelTemplate {
    fn load(&self, state: &mut dyn State) -> Widget {
        let widget = Label::new(state);
        widget.obj().apply_style(state, Some(self.0.clone()));
        widget
    }
}

impl WidgetObj for Label {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}

struct LabelBehavior;

impl WidgetBehavior for LabelBehavior {
    fn load(&self, state: &mut dyn State, button: Widget, view: View) {
        LabelDecorator::new(state, view);
        let text = Binding1::new(state, (), |(), (_, x)| Some(x));
        text.set_source_1(state, &mut Label::TEXT.source(button.obj()));
        LabelDecorator::TEXT.bind_uncond(state, view.decorator(), text);
    }
}
