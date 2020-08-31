#![windows_subsystem = "windows"]
#![deny(warnings)]

use tuifw_screen_base::{Point, Color, Attr, Vector, Event, Key, Rect};
use tuifw_window::{WindowTree, Window, RenderPort};

fn draw(
    _tree: &WindowTree<(), ()>,
    window: Option<Window<(), ()>>,
    port: &mut RenderPort,
    _tag: &(),
    _context: &mut ()
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
    let tree = &mut WindowTree::new(screen, draw, ());
    let mut bounds = Rect {
        tl: Point { x: (tree.screen_size().x - 13) / 2, y: (tree.screen_size().y - 7) / 2 },
        size: Vector { x: 13, y: 7 }
    };
    let window = Window::new(tree, None, bounds, |window| ((), window));
    loop {
        if let Some(e) = tree.update(true, &mut ()).unwrap() {
            let d = match e {
                Event::Key(n, Key::Left) => -Vector { x: n.get() as i16, y: 0 },
                Event::Key(n, Key::Right) => Vector { x: n.get() as i16, y: 0 },
                Event::Key(n, Key::Up) => -Vector { x: 0, y: n.get() as i16 },
                Event::Key(n, Key::Down) => Vector { x: 0, y: n.get() as i16 },
                Event::Key(n, Key::Char('h')) => -Vector { x: n.get() as i16, y: 0 },
                Event::Key(n, Key::Char('l')) => Vector { x: n.get() as i16, y: 0 },
                Event::Key(n, Key::Char('k')) => -Vector { x: 0, y: n.get() as i16 },
                Event::Key(n, Key::Char('j')) => Vector { x: 0, y: n.get() as i16 },
                Event::Key(_, Key::Escape) => break,
                _ => Vector::null(),
            };
            bounds = bounds.offset(d);
            window.move_(tree, bounds)
        }
    }
}
