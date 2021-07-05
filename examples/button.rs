#![deny(warnings)]
use dep_obj::{Style, Dispatcher};
use tuifw::*;
use tuifw::view::panels::DockPanel;
use dyn_context::{StateExt, StateRefMut};

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let mut tree = WidgetTree::new(screen);
    let mut dispatcher = Dispatcher::new();
    (&mut tree).merge_mut_and_then(|state| {
        let tree: &WidgetTree = state.get();
        let root = tree.root();
        root.obj_mut(state).set_uncond(Root::PANEL_TEMPLATE, Some(Box::new(<Style<DockPanel>>::new())));
        let tree: &mut WidgetTree = state.get_mut();
        let button = Button::new(tree);
        root.obj_mut(state).push(Root::CHILDREN, button);
        while WidgetTree::update(state, true).unwrap() {
            Dispatcher::dispatch(state);
        }
        while Dispatcher::dispatch(state) { }
    }, &mut dispatcher);
}