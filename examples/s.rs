#![windows_subsystem = "windows"]

#![deny(warnings)]

//use alloc::boxed::Box;
//use core::any::Any;
use std::mem::replace;
use tuifw_screen::{Bg, Fg, HAlign, VAlign};
use tuifw::{Background, InputLine, InputLineValueRange, StackPanel, StaticText};

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut Background::new().window_tree(screen).unwrap();
    let root = tree.root();
    let panel = StackPanel::new().window(tree, root, None).unwrap();
    panel.set_h_align(tree, Some(HAlign::Center));
    panel.set_v_align(tree, Some(VAlign::Center));
    let text = StaticText::new().window(tree, panel, None).unwrap();
    StaticText::text_mut(tree, text, |value| replace(value, "Hello!".to_string()));
    let input = InputLine {
        value_range: InputLineValueRange::Integer(0 .. i64::MAX),
        normal_color: (Fg::White, Bg::Blue),
        error_color: (Fg::White, Bg::Red),
        value: "12345".to_string(),
        view_start: 0, cursor_index: 0, cursor_x: 0,
    }.window(tree, panel, Some(text)).unwrap();
    input.set_width(tree, 10);
    input.focus(tree, &mut ());
    loop {
        tree.update(true, &mut ()).unwrap();
    }
}
