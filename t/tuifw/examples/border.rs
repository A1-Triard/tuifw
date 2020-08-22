#![windows_subsystem = "windows"]
#![deny(warnings)]

use std::io::{self};
use tuifw_screen_base::{Point, Color, Attr, Vector, Event, Key, Rect};
use tuifw_window::{WindowTree, Window, DrawingPort};
use tuifw::{Border};

fn draw(
    tree: &WindowTree<Box<dyn Drawing<io::Error>>, io::Error>,
    _window: Option<Window<Box<dyn Drawing<io::Error>>>,
    port: &mut DrawingPort<io::Error>,
    tag: &Box<dyn Drawing<io::Error>>
) {
    tag.draw(tree, port)
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let tree = &mut WindowTree::new(screen, draw, Box::new(Border::new(None)));
    let mut bounds = Rect {
        tl: Point { x: (tree.screen_size().x - 13) / 2, y: (tree.screen_size().y - 7) / 2 },
        size: Vector { x: 13, y: 7 }
    };
    let window = Window::new(tree, None, bounds, ());
    loop {
        if let Some(e) = tree.update(true).unwrap() {
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
