#![windows_subsystem = "windows"]

#![deny(warnings)]

use tuifw_screen::{Bg, Fg, HAlign, Key, Point, Rect, Thickness, VAlign, Vector};
use tuifw_window::{Event, RenderPort, Window, WindowTree};

fn measure(
    tree: &mut WindowTree<bool, (Point, bool)>,
    window: Window<bool>,
    available_width: Option<i16>,
    available_height: Option<i16>,
    state: &mut (Point, bool)
) -> Vector {
    if *window.data(tree) {
        let child = window.first_child(tree).unwrap();
        child.measure(tree, None, None, state);
        Vector { x: available_width.unwrap(), y: available_height.unwrap() }
    } else {
        Vector { x: 13, y: 7 }
    }
}

fn arrange(
    tree: &mut WindowTree<bool, (Point, bool)>,
    window: Window<bool>,
    final_inner_bounds: Rect,
    state: &mut (Point, bool)
) -> Vector {
    if *window.data(tree) {
        let child = window.first_child(tree).unwrap();
        let child_desired_size = child.desired_size(tree);
        child.arrange(tree, Rect { tl: state.0, size: child_desired_size }, state);
        final_inner_bounds.size
    } else {
        final_inner_bounds.size
    }
}

fn render(
    tree: &WindowTree<bool, (Point, bool)>,
    window: Window<bool>,
    rp: &mut RenderPort,
    _state: &mut (Point, bool)
) {
    if *window.data(tree) {
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

fn update(
    tree: &mut WindowTree<bool, (Point, bool)>,
    _window: Window<bool>,
    event: Event,
    preview: bool,
    state: &mut (Point, bool)
) -> bool {
    if preview { return false; }
    let d = match event {
        Event::Key(n, Key::Left) | Event::Key(n, Key::Char('h')) =>
            -Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
        Event::Key(n, Key::Right) | Event::Key(n, Key::Char('l')) =>
            Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
        Event::Key(n, Key::Up) | Event::Key(n, Key::Char('k')) =>
            -Vector { x: 0, y: n.get() as i16 },
        Event::Key(n, Key::Down) | Event::Key(n, Key::Char('j')) =>
            Vector { x: 0, y: n.get() as i16 },
        Event::Key(_, Key::Escape) => { state.1 = true; Vector::null() },
        _ => Vector::null(),
    };
    state.0 = state.0.offset(d);
    let root = tree.root();
    root.invalidate_arrange(tree);
    true
}

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut WindowTree::new(screen, render, measure, arrange, update, true).unwrap();
    let root = tree.root();
    Window::new(tree, false, root, None).unwrap();
    let size = Vector { x: 13, y: 7 };
    let screen_size = root.bounds(tree).size;
    let padding = Thickness::align(size, screen_size, HAlign::Center, VAlign::Center);
    let mut state = (
        padding.shrink_rect(Rect { tl: Point { x: 0, y: 0 }, size: screen_size }).tl,
        false
    );
    while !state.1 {
        tree.update(true, &mut state).unwrap();
    }
}
