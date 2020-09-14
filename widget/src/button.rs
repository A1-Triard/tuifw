use std::borrow::Cow;
use tuifw_screen_base::Side;
use dep_obj::{dep_obj, DepTypeToken, DepObjBuilderCore};
use once_cell::sync::{self};
use crate::base::{Widget, WidgetTree, WidgetObj, WidgetBehavior, ViewBuilderWidgetExt};
use crate::view::View;
use crate::view::decorators::{ViewBuilderLabelDecoratorExt};
use crate::view::panels::{ViewBuilderDockPanelExt};

dep_obj! {
    #[derive(Debug)]
    pub struct Button become obj in Widget where BuilderCore<'a> = DepObjBuilderCore<'a> {
        content: Option<Widget> = None,
    }
}

static BUTTON_TOKEN: sync::Lazy<DepTypeToken<ButtonType>> = sync::Lazy::new(||
    ButtonType::new_priv().expect("ButtonType builder locked")
);

pub fn button_type() -> &'static ButtonType { BUTTON_TOKEN.ty() }

impl Button {
    const BEHAVIOR: ButtonBehavior = ButtonBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(tree: &mut WidgetTree) -> Widget {
        Widget::new(tree, Button::new_priv(&BUTTON_TOKEN))
    }
}

impl WidgetObj for Button {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}

struct ButtonBehavior;

impl WidgetBehavior for ButtonBehavior {
    fn load(&self, tree: &mut WidgetTree, button: Widget, view: View) {
        let &content = button.obj_get(tree, button_type().content());
        view.build(tree, |view| view
            .dock_panel(|panel| {
                panel
                    .child(None, button, |layout| layout.dock(Some(Side::Left)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed("[ ")))
                    )
                    .child(None, button, |layout| layout.dock(Some(Side::Right)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed(" ]")))
                    )
                ;
                if let Some(content) = content {
                    panel.child(None, content, |layout| layout, |view| view.widget(content));
                }
                panel
            })
        );
    }
}
