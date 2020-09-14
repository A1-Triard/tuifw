use crate::base::{Widget, WidgetTree};

pub struct Button(());

impl Button {
    fn load(tree: &mut WidgetTree, button: Widget, view: View) {
        view.build(tree, |view| view
            .dock_panel(|panel| panel
                .child(button, |layout| layout.dock(Some(Side::Left)), |view| view
                    .label_decorator(|label| label.text(Cow::Borrowed("[ ")))
                )
                .child(button, |layout| layout.dock(Some(Side::Right)), |view| view
                    .label_decorator(|label| label.text(Cow::Borrowed(" ]")))
                )
                .child(button, |layout| layout, |view| view
                    .label_decorator(|label| label.text(Cow::Borrowed(" ]")))
                )
    }

    pub fn new(tree: &WidgetTree) -> Widget {
        Widget::new(tree, load)
    }
}
