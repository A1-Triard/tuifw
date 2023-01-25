#![windows_subsystem = "windows"]

#![deny(warnings)]

use tuifw_screen::{Bg, Event, Fg, HAlign, Key, Point, Rect, Thickness, VAlign, Vector};
use tuifw_window::{RenderPort, Window, WindowTree};

fn draw(
    _tree: &WindowTree<()>,
    window: Option<Window>,
    rp: &mut RenderPort,
    _state: &mut ()
) {
    if window.is_none() {
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
    let tree = &mut WindowTree::new(screen, draw);
    let size = Vector { x: 13, y: 7 };
    let padding = Thickness::align(size, tree.screen_size(), HAlign::Center, VAlign::Center);
    let mut bounds = padding.shrink_rect(Rect { tl: Point { x: 0, y: 0 }, size: tree.screen_size() });
    let window = Window::new(tree, None, None);
    window.move_xy(tree, bounds);
    loop {
        if let Some(e) = tree.update(true, &mut ()).unwrap() {
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
            bounds = bounds.offset(d);
            window.move_xy(tree, bounds)
        }
    }
}
