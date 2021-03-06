use libc::*;
use tuifw_screen_base::*;
use crate::ncurses::*;
use std::ptr::NonNull;
use std::io::{self};
use std::num::NonZeroU16;
use either::{Either, Left, Right};

pub fn no_err(r: c_int) -> io::Result<c_int> {
    if r == ERR { Err(io::ErrorKind::Other.into()) } else { Ok(r) }
}

pub fn no_null<T: ?Sized>(r: *mut T) -> io::Result<NonNull<T>> {
    NonNull::new(r).ok_or_else(|| io::ErrorKind::Other.into())
}

fn colors_count() -> i16 {
    Color::iter_variants().len() as i16
}

fn color_index(c: Color) -> i16 {
    match c {
        Color::Black => COLOR_BLACK,
        Color::Red => COLOR_RED,
        Color::Green => COLOR_GREEN,
        Color::Yellow => COLOR_YELLOW,
        Color::Blue => COLOR_BLUE,
        Color::Magenta => COLOR_MAGENTA,
        Color::Cyan => COLOR_CYAN,
        Color::White => COLOR_WHITE,
    }
}

pub unsafe fn init_settings() -> io::Result<()> {
    no_err(cbreak())?; 
    no_err(noecho())?; 
    nonl(); 
    register_colors()?;
    set_escdelay(0);
    no_err(keypad(stdscr, true))?;
    Ok(())
}

unsafe fn register_colors() -> io::Result<()> {
    no_err(start_color())?;
    no_err(assume_default_colors(0, -1))?;
    for fg in Color::iter_variants().map(color_index) {
        if fg != 0 {
            no_err(init_pair(fg, fg, -1))?;
        }
        for bg in Color::iter_variants().map(color_index) {
            no_err(init_pair((1 + bg) * colors_count() + fg, fg, bg))?;
        }
    }
    Ok(())
}

fn attr_value(a: Attr) -> chtype {
    let mut r = 0;
    if a.contains(Attr::REVERSE) { r |= A_REVERSE; }
    if a.contains(Attr::INTENSITY) { r |= A_BOLD; }
    r
}

pub unsafe fn attr_ch(fg: Color, bg: Option<Color>, attr: Attr) -> chtype {
    let color = COLOR_PAIR((bg.map_or(0, |b| (color_index(b) + 1) * colors_count()) + color_index(fg)) as _);
    attr_value(attr) | color as chtype
}

const KEY_F1: c_int = KEY_F(1);
const KEY_F2: c_int = KEY_F(2);
const KEY_F3: c_int = KEY_F(3);
const KEY_F4: c_int = KEY_F(4);
const KEY_F5: c_int = KEY_F(5);
const KEY_F6: c_int = KEY_F(6);
const KEY_F7: c_int = KEY_F(7);
const KEY_F8: c_int = KEY_F(8);
const KEY_F9: c_int = KEY_F(9);
const KEY_F10: c_int = KEY_F(10);
const KEY_F11: c_int = KEY_F(11);
const KEY_F12: c_int = KEY_F(12);

const ONCE: NonZeroU16 = unsafe { NonZeroU16::new_unchecked(1) };

pub fn read_event(
    window: NonNull<WINDOW>,
    getch: impl Fn(NonNull<WINDOW>
) -> Option<Either<c_int, char>>) -> io::Result<Option<Event>> {
    let e = if let Some(e) = getch(window) {
        e
    } else {
        return Ok(None);
    };
    match e {
        Left(key) => Ok(match key {
            KEY_RESIZE => Some(Event::Resize),
            KEY_DOWN => Some(Event::Key(ONCE, Key::Down)),
            KEY_UP => Some(Event::Key(ONCE, Key::Up)),
            KEY_LEFT => Some(Event::Key(ONCE, Key::Left)),
            KEY_RIGHT => Some(Event::Key(ONCE, Key::Right)),
            KEY_HOME => Some(Event::Key(ONCE, Key::Home)),
            KEY_END => Some(Event::Key(ONCE, Key::End)),
            KEY_BACKSPACE => Some(Event::Key(ONCE, Key::Backspace)),
            KEY_DC => Some(Event::Key(ONCE, Key::Delete)),
            KEY_IC => Some(Event::Key(ONCE, Key::Insert)),
            KEY_NPAGE => Some(Event::Key(ONCE, Key::PageDown)),
            KEY_PPAGE => Some(Event::Key(ONCE, Key::PageUp)),
            KEY_F1 => Some(Event::Key(ONCE, Key::F1)),
            KEY_F2 => Some(Event::Key(ONCE, Key::F2)),
            KEY_F3 => Some(Event::Key(ONCE, Key::F3)),
            KEY_F4 => Some(Event::Key(ONCE, Key::F4)),
            KEY_F5 => Some(Event::Key(ONCE, Key::F5)),
            KEY_F6 => Some(Event::Key(ONCE, Key::F6)),
            KEY_F7 => Some(Event::Key(ONCE, Key::F7)),
            KEY_F8 => Some(Event::Key(ONCE, Key::F8)),
            KEY_F9 => Some(Event::Key(ONCE, Key::F9)),
            KEY_F10 => Some(Event::Key(ONCE, Key::F10)),
            KEY_F11 => Some(Event::Key(ONCE, Key::F11)),
            KEY_F12 => Some(Event::Key(ONCE, Key::F12)),
            _ => None
        }),
        Right(c) => Ok(match c {
            '\x1B' => {
                unsafe { no_err(nodelay(window.as_ptr(), true)) }?;
                if let Some(e) = getch(window) {
                    if let Right(c) = e {
                        if c < ' ' || c == '\x7F' {
                            None
                        } else {
                            Some(Event::Key(ONCE, Key::Alt(c)))
                        }
                    } else {
                        Some(Event::Key(ONCE, Key::Escape))
                    }
                } else {
                    Some(Event::Key(ONCE, Key::Escape))
                }
            },
            '\0' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::At))),
            '\x01' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::A))),
            '\x02' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::B))),
            '\x03' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::C))),
            '\x04' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::D))),
            '\x05' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::E))),
            '\x06' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::F))),
            '\x07' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::G))),
            '\x08' => Some(Event::Key(ONCE, Key::Backspace)),
            '\t' => Some(Event::Key(ONCE, Key::Tab)),
            '\x0A' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::J))),
            '\x0B' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::K))),
            '\x0C' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::L))),
            '\r' => Some(Event::Key(ONCE, Key::Enter)),
            '\x0E' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::N))),
            '\x0F' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::O))),
            '\x10' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::P))),
            '\x11' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::Q))),
            '\x12' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::R))),
            '\x13' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::S))),
            '\x14' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::T))),
            '\x15' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::U))),
            '\x16' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::V))),
            '\x17' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::W))),
            '\x18' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::X))),
            '\x19' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::Y))),
            '\x1A' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::Z))),
            '\x1C' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::Backslash))),
            '\x1D' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::Bracket))),
            '\x1E' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::Caret))),
            '\x1F' => Some(Event::Key(ONCE, Key::Ctrl(Ctrl::Underscore))),
            '\x7F' => Some(Event::Key(ONCE, Key::Backspace)),
            c => Some(Event::Key(ONCE, Key::Char(c)))
        })
    }
}
