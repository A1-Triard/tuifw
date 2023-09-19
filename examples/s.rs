#![windows_subsystem = "windows"]

#![deny(warnings)]

use tuifw_screen::{Bg, Fg, Point, Rect, Vector};
use tuifw_window::{RenderPort, Window, WindowTree};
use tuifw::{EditValueRange, LineEdit};

fn render(
    _tree: &WindowTree<Model>,
    window: Option<Window>,
    rp: &mut RenderPort,
    state: &mut Model
) {
    if let Some(window) = window {
        state.a.render(window, rp);
    } else {
        rp.fill(|rp, p| rp.out(p, Fg::Black, Bg::None, " "));
    }
}

struct Model {
    a: LineEdit,
}

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut WindowTree::new(screen, render).unwrap();
    let mut a = LineEdit::new(tree, EditValueRange::Float(f64::from(f32::MIN) .. f64::from(f32::MAX)), None, None).unwrap();
    a.move_xy(tree, Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: 20, y: 1 } });
    a.line_mut(tree, |line| line.push_str("0.0"));
    let mut model = Model { a };
    loop {
        if let Some(e) = tree.update(true, &mut model).unwrap() {
            model.a.update(tree, e);
        }
    }
}
