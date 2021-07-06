use crate::base::{WidgetTemplate, Widget, WidgetTree, WidgetObj, WidgetBehavior};
use crate::view::View;
use crate::view::decorators::{ViewBuilderLabelDecoratorExt};
use dep_obj::{dep_type, Style};
use dyn_context::{State, StateExt};
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
    pub fn new(tree: &mut WidgetTree) -> Widget {
        Widget::new(tree, Label::new_priv())
    }

    pub fn template(style: Style<Label>) -> Box<dyn WidgetTemplate> {
        Box::new(LabelTemplate(style))
    }
}

#[derive(Debug, Clone)]
struct LabelTemplate(Style<Label>);

impl WidgetTemplate for LabelTemplate {
    fn load(&self, state: &mut dyn State) -> Widget {
        let tree: &mut WidgetTree = state.get_mut();
        let widget = Label::new(tree);
        widget.obj_mut(state).apply_style(Some(self.0.clone()));
        widget
    }
}

impl WidgetObj for Label {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}

struct LabelBehavior;

impl WidgetBehavior for LabelBehavior {
    fn load(&self, state: &mut dyn State, button: Widget, view: View) {
        let tree: &WidgetTree = state.get();
        let text = button.obj_ref(tree).get(Label::TEXT).clone();
        view.build(state, |view| view.label_decorator(|label| label.text(text)));
    }
}
