#![deny(warnings)]
use dep_obj::{Style, Dispatcher};
use dyn_context::{StateExt, StateRefMut};
use std::borrow::Cow;
use tuifw::*;
use tuifw::view::panels::DockPanel;

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let mut tree = WidgetTree::new(screen);
    let mut dispatcher = Dispatcher::new();
    (&mut tree).merge_mut_and_then(|state| {
        let tree: &mut WidgetTree = state.get_mut();
        let label = Label::new(tree);
        let button = Button::new(tree);
        let root = tree.root();
        let panel_template = DockPanel::template(Style::new(), Style::new());
        root.obj_mut(state).set_uncond(Root::PANEL_TEMPLATE, Some(panel_template));
        label.obj_mut(state).set_uncond(Label::TEXT, Cow::Borrowed("Press me!"));
        button.obj_mut(state).set_uncond(Button::CONTENT, Some(label));
        root.obj_mut(state).push(Root::CHILDREN, button);
        Dispatcher::dispatch(state);
        while WidgetTree::update(state, true).unwrap() {
            Dispatcher::dispatch(state);
        }
        while Dispatcher::dispatch(state) { }
    }, &mut dispatcher);
}