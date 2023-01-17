#![windows_subsystem = "windows"]

#![deny(warnings)]

use tuifw_screen::{Bg, Fg, Screen, Point, Event, Key};

const CONTROL_CHARS: &str = "\
    \x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\
    \x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F\
";

const DOS_CHARS: &str = "\
    .☺☻♥♦♣♠•◘○◙♂♀♪♫☼\
    ►◄↕‼¶§▬↨↑↓→←∟↔▲▼\
";

const WIDE_CHARS: &str = "好 🏈 子";

fn draw(screen: &mut dyn Screen) {
    let w = 0 .. screen.size().x;
    screen.out(Point { x: 0, y: 0 }, Fg::LightGray, Bg::Blue, CONTROL_CHARS, w.clone(), w.clone());
    screen.out(Point { x: 0, y: 1 }, Fg::LightGray, Bg::Blue, DOS_CHARS, w.clone(), w.clone());
    screen.out(Point { x: 0, y: 3 }, Fg::LightGray, Bg::Blue, WIDE_CHARS, w.clone(), w.clone());
}

fn main() {
    let mut screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
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
