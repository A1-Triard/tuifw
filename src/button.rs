use std::borrow::Cow;
use tuifw_screen_base::Side;
use dep_obj::{dep_type};
use either::Right;
use crate::base::{Widget, WidgetTree, WidgetObj, WidgetBehavior, ViewBuilderWidgetExt};
use crate::view::View;
use crate::view::decorators::{ViewBuilderLabelDecoratorExt};
use crate::view::panels::{ViewBuilderDockPanelExt};

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
