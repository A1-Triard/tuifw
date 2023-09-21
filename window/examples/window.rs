#![windows_subsystem = "windows"]

#![deny(warnings)]

use tuifw_screen::{Bg, Event, Fg, HAlign, Key, Point, Rect, Thickness, VAlign, Vector};
use tuifw_window::{RenderPort, Window, WindowTree};

fn measure(
    tree: &mut WindowTree<bool, Point>,
    window: Window<bool>,
    available_width: Option<i16>,
    available_height: Option<i16>,
    state: &mut Point
) -> Vector {
    if *window.tag(tree) {
        let child = window.first_child(tree).unwrap();
        child.measure(tree, None, None, state);
        Vector { x: available_width.unwrap(), y: available_height.unwrap() }
    } else {
        Vector { x: 13, y: 7 }
    }
}

fn arrange(
    tree: &mut WindowTree<bool, Point>,
    window: Window<bool>,
    final_inner_bounds: Rect,
    state: &mut Point
) -> Vector {
    if *window.tag(tree) {
        let child = window.first_child(tree).unwrap();
        let child_desired_size = child.desired_size(tree);
        child.arrange(tree, Rect { tl: *state, size: child_desired_size }, state);
        final_inner_bounds.size
    } else {
        final_inner_bounds.size
    }
}

fn render(
    tree: &WindowTree<bool, Point>,
    window: Window<bool>,
    rp: &mut RenderPort,
    _state: &mut Point
) {
    if *window.tag(tree) {
        rp.fill(|rp, p| rp.out(p, Fg::Black, Bg::None, " "));
    } else {
        rp.out(Point { x: 0, y: 0 }, Fg::Green, Bg::None, "╔═══════════╗");
        rp.out(Point { x: 0, y: 1 }, Fg::Green, Bg::None, "║     ↑     ║");
        rp.out(Point { x: 0, y: 2 }, Fg::Green, Bg::None, "║     k     ║");
        rp.out(Point { x: 0, y: 3 }, Fg::Green, Bg::None, "║ ←h     l→ ║");
        rp.out(Point { x: 0, y: 4 }, Fg::Green, Bg::None, "║     j     ║");
        rp.out(Point { x: 0, y: 5 }, Fg::Green, Bg::None, "║     ↓     ║");
        rp.out(Point { x: 0, y: 6 }, Fg::Green, Bg::None, "╚═══════════╝");
    }
}

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut WindowTree::new(screen, render, measure, arrange, true).unwrap();
    let root = tree.root();
    let size = Vector { x: 13, y: 7 };
    let screen_size = root.bounds(tree).size;
    let padding = Thickness::align(size, screen_size, HAlign::Center, VAlign::Center);
    let mut location = padding.shrink_rect(Rect { tl: Point { x: 0, y: 0 }, size: screen_size }).tl;
    Window::new(tree, false, root, None).unwrap();
    loop {
        if let Some(e) = tree.update(true, &mut location).unwrap() {
            let d = match e {
                Event::Key(n, Key::Left) | Event::Key(n, Key::Char('h')) =>
                    -Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
                Event::Key(n, Key::Right) | Event::Key(n, Key::Char('l')) =>
                    Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
                Event::Key(n, Key::Up) | Event::Key(n, Key::Char('k')) =>
                    -Vector { x: 0, y: n.get() as i16 },
                Event::Key(n, Key::Down) | Event::Key(n, Key::Char('j')) =>
                    Vector { x: 0, y: n.get() as i16 },
                Event::Key(_, Key::Escape) => break,
                _ => Vector::null(),
            };
            location = location.offset(d);
            root.invalidate_arrange(tree);
        }
    }
}
