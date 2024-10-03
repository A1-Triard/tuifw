#![windows_subsystem = "windows"]

#![deny(warnings)]

use tuifw_screen::{Bg, Fg, Screen, Point, Event, Key};

fn draw(screen: &mut dyn Screen, point: Point, down: bool) {
    let w = 0 .. screen.size().x;
    screen.out(point, Fg::Green, Bg::None, if down { "X" } else { "█" }, w.clone(), w.clone());
}

fn main() {
    let mut screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let screen = screen.as_mut();
    let mut lmb_down_point = None;
    loop {
        if let Some(e) = screen.update(None, true).unwrap() {
            match e {
                Event::Key(_, Key::Escape) => break,
                Event::LmbDown(p) => {
                    lmb_down_point = Some(p);
                    draw(screen, p, true);
                },
                Event::LmbUp(p) => {
                    draw(screen, lmb_down_point.take().unwrap_or(p), false);
                }
                _ => { },
            }
        }
    }
}
