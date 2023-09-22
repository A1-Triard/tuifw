#![windows_subsystem = "windows"]

//#![deny(warnings)]

//use alloc::boxed::Box;
//use core::any::Any;
use tuifw_screen::{Bg, Fg, Vector, Thickness, HAlign, VAlign};
use tuifw::{Background, StackPanel, StaticText};

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut Background { bg: Bg::LightGray, fg: Some(Fg::Blue) }.window_tree(screen).unwrap();
    let root = tree.root();
    let panel = StackPanel { vertical: true }.window(tree, root, None).unwrap();
    StaticText { color: (Fg::Green, Bg::None), text: "Hello!".to_string() }.window(tree, panel, None).unwrap();
    panel.move_xy(tree, Some(HAlign::Center), Some(VAlign::Center), Thickness::all(0), Vector::null(), Vector { x: -1, y: -1 });
    loop {
        tree.update(true, &mut ()).unwrap();
    }
}
