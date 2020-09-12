use crate::base::{Widget, WidgetTree};

pub struct Button(());

impl Button {
    fn load(tree: &mut WidgetTree, button: Widget, parent_view: View) -> View {
    }

    pub fn new(tree: &WidgetTree) -> Widget {
        Widget::new(tree, load)
    }
}
