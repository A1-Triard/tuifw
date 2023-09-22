#![windows_subsystem = "windows"]

#![deny(warnings)]

//use alloc::boxed::Box;
//use core::any::Any;
use tuifw_screen::{Bg, Fg};
use tuifw_window::{WindowTree};
use tuifw::{StackPanel, StaticText, widget_render, widget_measure, widget_arrange};

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut WindowTree::new(
        screen, widget_render, widget_measure, widget_arrange,
        StackPanel { vertical: true }.widget_tag()
    ).unwrap();
    let root = tree.root();
    StaticText { color: (Fg::Green, Bg::None), text: "Hello!".to_string() }.window(tree, root, None).unwrap();
    loop {
        tree.update(true, &mut ()).unwrap();
    }
}
