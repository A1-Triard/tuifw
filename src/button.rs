use crate::base::{WidgetTemplate, Widget, WidgetTree, WidgetObj, WidgetBehavior, ViewBuilderWidgetExt};
use crate::view::View;
use crate::view::decorators::{ViewBuilderLabelDecoratorExt};
use crate::view::panels::{ViewBuilderDockPanelExt};
use dep_obj::{dep_type, Style};
use dyn_context::{Context, ContextExt};
use either::Right;
use std::borrow::Cow;
use tuifw_screen_base::Side;

dep_type! {
    #[derive(Debug)]
    pub struct Button become obj in Widget {
        content: Option<Widget> = None,
    }
}

impl Button {
    const BEHAVIOR: ButtonBehavior = ButtonBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(tree: &mut WidgetTree) -> Widget {
        Widget::new(tree, Button::new_priv())
    }

    pub fn template(style: Style<Button>) -> Box<dyn WidgetTemplate> {
        Box::new(ButtonTemplate(style))
    }
}

#[derive(Debug, Clone)]
struct ButtonTemplate(Style<Button>);

impl WidgetTemplate for ButtonTemplate {
    fn load(&self, context: &mut dyn Context) -> Widget {
        let tree: &mut WidgetTree = context.get_mut();
        let widget = Button::new(tree);
        widget.obj_apply_style(context, Some(self.0.clone()));
        widget
    }
}

impl WidgetObj for Button {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}

struct ButtonBehavior;

impl WidgetBehavior for ButtonBehavior {
    fn load(&self, tree: &mut WidgetTree, button: Widget, view: View) {
        let &content = button.obj_get(tree, Button::CONTENT);
        view.build(tree, |view| view
            .dock_panel(|panel| {
                let panel = panel
                    .child(None, button, |layout| layout.dock(Right(Side::Left)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed("[ ")))
                    )
                    .child(None, button, |layout| layout.dock(Right(Side::Right)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed(" ]")))
                    )
                ;
                if let Some(content) = content {
                    panel.child(None, content, |layout| layout, |view| view.widget(content))
                } else {
                    panel
                }
            })
        );
    }
}
