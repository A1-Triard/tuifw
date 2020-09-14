use dep_obj::{dep_obj, DepTypeToken};
use once_cell::sync::{self};
use crate::base::{Widget, WidgetTree, WidgetObj, WidgetBehavior};
use crate::view::View;

dep_obj! {
    #[derive(Debug)]
    pub struct Button become obj in Widget {
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
    fn load(&self, _tree: &mut WidgetTree, _button: Widget, _view: View) {
        /*
        let content = 
        view.build(tree, |view| view
            .dock_panel(|panel| panel
                .child(button, |layout| layout.dock(Some(Side::Left)), |view| view
                    .label_decorator(|label| label.text(Cow::Borrowed("[ ")))
                )
                .child(button, |layout| layout.dock(Some(Side::Right)), |view| view
                    .label_decorator(|label| label.text(Cow::Borrowed(" ]")))
                )
                .child(button, |layout| layout, |view| {

                    view
                    .label_decorator(|label| label.text(Cow::Borrowed(" ]")))
                )
        */
    }
}
