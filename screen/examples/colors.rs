#![windows_subsystem = "windows"]

#![deny(warnings)]

use tuifw_screen::{Bg, Fg, Screen, Point, Event, Key};

fn draw(screen: &mut dyn Screen) {
    let w = 0 .. screen.size().x;
    for (bg_n, bg) in Bg::iter_variants().enumerate() {
        let bg_n: i16 = bg_n.try_into().unwrap();
        for (fg_n, fg) in Fg::iter_variants().enumerate() {
            let fg_n: i16 = fg_n.try_into().unwrap();
            screen.out(Point { x: 3 * fg_n, y: bg_n }, fg, bg, " ■ ", w.clone(), w.clone());
        }
    }
}

fn main() {
    let mut screen = unsafe { tuifw_screen::init(None) }.unwrap();
    let screen = screen.as_mut();
    draw(screen);
    loop {
        if let Some(e) = screen.update(None, true).unwrap() {
            if matches!(e, Event::Key(_, Key::Escape)) { break; }
            if matches!(e, Event::Resize) {
                let w = 0 .. screen.size().x;
                for x in 0 .. screen.size().x {
                    for y in 0 .. screen.size().y {
                        screen.out(Point { x, y }, Fg::LightGray, Bg::None, " ", w.clone(), w.clone());
                    }
                }
                draw(screen);
            }
        }
    }
}
