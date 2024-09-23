#![windows_subsystem = "windows"]

//#![deny(warnings)]

use tuifw_screen::{Bg, Fg, Screen, Point, Event, Key};

fn draw(screen: &mut dyn Screen, event: &'static str, point: Point) {
    let w = 0 .. screen.size().x;
    screen.out(point, Fg::Green, Bg::None, event, w.clone(), w.clone());
}

fn main() {
    let mut screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let screen = screen.as_mut();
    loop {
        if let Some(e) = screen.update(None, true).unwrap() {
            match e {
                Event::Key(_, Key::Escape) => break,
                Event::MouseMove(p) => draw(screen, "M", p),
                Event::LmbPressed(p) => draw(screen, "l", p),
                Event::LmbReleased(p) => draw(screen, "L", p),
                _ => { },
            }
        }
    }
}
