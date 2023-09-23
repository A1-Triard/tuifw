#![windows_subsystem = "windows"]

//#![deny(warnings)]

//use alloc::boxed::Box;
//use core::any::Any;
use tuifw_screen::{Bg, Fg, Vector, Thickness, HAlign, VAlign};
use tuifw::{Background, InputLine, InputLineValueRange, StackPanel, StaticText};

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut Background { bg: Bg::LightGray, fg: Some(Fg::Blue) }.window_tree(screen).unwrap();
    let root = tree.root();
    let panel = StackPanel { vertical: true }.window(tree, root, None).unwrap();
    panel.move_xy(tree, Some(HAlign::Center), Some(VAlign::Center), Thickness::all(0), Vector::null(), Vector { x: -1, y: -1 });
    let text = StaticText { color: (Fg::Green, Bg::None), text: "Hello!".to_string() }.window(tree, panel, None).unwrap();
    let input = InputLine {
        value_range: InputLineValueRange::Integer(0 .. i64::MAX),
        normal_color: (Fg::White, Bg::Blue),
        error_color: (Fg::White, Bg::Red),
        value: "12345".to_string(),
        view_start: 0, cursor_index: 0, cursor_x: 0,
    }.window(tree, panel, Some(text)).unwrap();
    input.move_xy(tree, None, None, Thickness::all(0), Vector { x: 10, y: 0 }, Vector { x: 10, y: -1 });
    input.focus(tree, &mut ());
    loop {
        tree.update(true, &mut ()).unwrap();
    }
}
