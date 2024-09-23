#![windows_subsystem = "windows"]

#![deny(warnings)]

use tuifw_screen::{Bg, Fg, Screen, Point, Event, Key};

fn draw(screen: &mut dyn Screen, point: Point) {
    let w = 0 .. screen.size().x;
    screen.out(point, Fg::Green, Bg::None, "█", w.clone(), w.clone());
}

fn main() {
    let mut screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let screen = screen.as_mut();
    loop {
        if let Some(e) = screen.update(None, true).unwrap() {
            match e {
                Event::Key(_, Key::Escape) => break,
                Event::MouseClick(p) => draw(screen, p),
                _ => { },
            }
        }
    }
}
