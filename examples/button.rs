#![deny(warnings)]
use dep_obj::Style;
use tuifw::*;
use tuifw::view::panels::DockPanel;


fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let mut tree = WidgetTree::new(screen);
    tree.root().obj_mut(&mut tree).set_uncond(Root::PANEL_TEMPLATE, Some(Box::new(<Style<DockPanel>>::new())));
    let button = Button::new(&mut tree);
    tree.root().obj_mut(&mut tree).push(Root::CHILDREN, button);
    while WidgetTree::update(&mut tree, true).unwrap() { }
}