#![windows_subsystem = "windows"]

#![deny(warnings)]

//use alloc::boxed::Box;
//use core::any::Any;
use tuifw_screen::{Bg, Fg};
use tuifw::{StackPanel, StaticText};

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut StackPanel { vertical: true }.window_tree(screen).unwrap();
    let root = tree.root();
    StaticText { color: (Fg::Green, Bg::None), text: "Hello!".to_string() }.window(tree, root, None).unwrap();
    loop {
        tree.update(true, &mut ()).unwrap();
    }
}
