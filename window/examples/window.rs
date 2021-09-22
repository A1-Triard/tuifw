#![windows_subsystem = "windows"]

#![deny(warnings)]

use dyn_context::state::State;
use tuifw_screen::{Attr, Color, Event, HAlign, Key, Point, Rect, Thickness, VAlign, Vector};
use tuifw_window::{RenderPort, Window, WindowTree};

fn draw(
    _tree: &WindowTree,
    window: Option<Window>,
    port: &mut RenderPort,
    _state: &mut dyn State
) {
    if window.is_none() {
        port.fill(|port, p| port.out(p, Color::Black, None, Attr::empty(), " "));
    } else {
        port.out(Point { x: 0, y: 0 }, Color::Green, None, Attr::empty(), "╔═══════════╗");
        port.out(Point { x: 0, y: 1 }, Color::Green, None, Attr::empty(), "║     ↑     ║");
        port.out(Point { x: 0, y: 2 }, Color::Green, None, Attr::empty(), "║     k     ║");
        port.out(Point { x: 0, y: 3 }, Color::Green, None, Attr::empty(), "║ ←h     l→ ║");
        port.out(Point { x: 0, y: 4 }, Color::Green, None, Attr::empty(), "║     j     ║");
        port.out(Point { x: 0, y: 5 }, Color::Green, None, Attr::empty(), "║     ↓     ║");
        port.out(Point { x: 0, y: 6 }, Color::Green, None, Attr::empty(), "╚═══════════╝");
    }
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let tree = &mut WindowTree::new(screen, draw);
    let size = Vector { x: 13, y: 7 };
    let padding = Thickness::align(size, tree.screen_size(), HAlign::Center, VAlign::Center);
    let mut bounds = padding.shrink_rect(Rect { tl: Point { x: 0, y: 0 }, size: tree.screen_size() });
    let window = Window::new(tree, None, None, bounds);
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
