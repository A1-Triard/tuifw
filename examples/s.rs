#![windows_subsystem = "windows"]

#![deny(warnings)]

use std::mem::replace;
use tuifw_screen::{HAlign, VAlign};
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
    let input = InputLine::new().window(tree, panel, Some(text)).unwrap();
    InputLine::set_value_range(tree, input, InputLineValueRange::Integer(0 ..= i64::MAX));
    input.set_width(tree, 10);
    InputLine::value_mut(tree, input, |value| replace(value, "1111222233334444".to_string()));
    input.focus(tree, &mut ());
    loop {
        tree.update(true, &mut ()).unwrap();
    }
}
