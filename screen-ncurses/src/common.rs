use crate::ncurses::*;
use alloc::boxed::Box;
use core::alloc::Allocator;
use core::ptr::{NonNull, null_mut};
use core::num::NonZeroU16;
use either::{Either, Left, Right};
use libc::*;
use tuifw_screen_base::*;

pub const GLOBAL: composable_allocators::Global = composable_allocators::Global;

pub fn set_err<T>(r: Result<T, ()>, func_name: &'static str, error_alloc: &'static dyn Allocator) -> Result<T, Error> {
    r.map_err(|()| Error::System(Box::new_in(func_name, error_alloc)))
}

pub fn non_err(r: c_int) -> Result<c_int, ()> {
    if r == ERR { Err(()) } else { Ok(r) }
}

pub fn non_null<T: ?Sized>(r: *mut T) -> Result<NonNull<T>, ()> {
    NonNull::new(r).ok_or(())
}

fn bg_index(c: Bg) -> i16 {
    match c {
        Bg::None => -1,
        Bg::Black => COLOR_BLACK,
        Bg::Red => COLOR_RED,
        Bg::Green => COLOR_GREEN,
        Bg::Brown => COLOR_YELLOW,
        Bg::Blue => COLOR_BLUE,
        Bg::Magenta => COLOR_MAGENTA,
        Bg::Cyan => COLOR_CYAN,
        Bg::LightGray => COLOR_WHITE,
    }
}

fn fg_index(c: Fg) -> i16 {
    match c {
        Fg::Black | Fg::DarkGray => COLOR_BLACK,
        Fg::Red | Fg::BrightRed => COLOR_RED,
        Fg::Green | Fg::BrightGreen => COLOR_GREEN,
        Fg::Brown | Fg::Yellow => COLOR_YELLOW,
        Fg::Blue | Fg::BrightBlue => COLOR_BLUE,
        Fg::Magenta | Fg::BrightMagenta => COLOR_MAGENTA,
        Fg::Cyan | Fg::BrightCyan => COLOR_CYAN,
        Fg::LightGray | Fg::White => COLOR_WHITE,
    }
}

fn fg_attr(c: Fg) -> chtype {
    match c {
        Fg::Black | Fg::Red | Fg::Green | Fg::Brown |
        Fg::Blue | Fg::Magenta | Fg::Cyan | Fg::LightGray =>
            A_NORMAL,
        Fg::DarkGray | Fg::BrightRed | Fg::BrightGreen | Fg::Yellow |
        Fg::BrightBlue | Fg::BrightMagenta | Fg::BrightCyan | Fg::White =>
            A_BOLD,
    }
}

pub unsafe fn init_settings(error_alloc: &'static dyn Allocator) -> Result<(), Error> {
    set_err(non_err(cbreak()), "cbreak", error_alloc)?;
    set_err(non_err(noecho()), "noecho", error_alloc)?;
    set_err(non_err(nonl()), "nonl", error_alloc)?;
    register_colors(error_alloc)?;
    set_escdelay(0);
    set_err(non_err(keypad(stdscr, true)), "keypad", error_alloc)?;
    mousemask(BUTTON1_CLICKED, null_mut());
    Ok(())
}

unsafe fn register_colors(error_alloc: &'static dyn Allocator) -> Result<(), Error> {
    set_err(non_err(start_color()), "start_color", error_alloc)?;
    set_err(non_err(use_default_colors()), "use_default_colors", error_alloc)?;
    for fg in Fg::iter_variants().map(fg_index) {
        for bg in Bg::iter_variants().map(bg_index) {
            set_err(non_err(init_pair(1 + (bg + 1) * 8 + fg, fg, bg)), "init_pair", error_alloc)?;
        }
    }
    Ok(())
}

pub unsafe fn attr_ch(fg: Fg, bg: Bg) -> chtype {
    let color = COLOR_PAIR((1 + (bg_index(bg) + 1) * 8 + fg_index(fg)) as _);
    fg_attr(fg) | color as chtype
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
    getch: impl Fn(NonNull<WINDOW>) -> Option<Either<c_int, char>>,
    error_alloc: &'static dyn Allocator
) -> Result<Option<Event>, Error> {
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
            KEY_MOUSE => {
                let mut e = MEVENT {
                    id: 0, x: 0, y: 0, z: 0, bstate: 0
                };
                let m = unsafe { getmouse(&mut e as *mut _) };
                if m == ERR {
                    None
                } else {
                    Some(Event::Click(Point { x: e.x as i16, y: e.y as i16 }))
                }
            },
            _ => None
        }),
        Right(c) => Ok(match c {
            '\x1B' => {
                set_err(unsafe { non_err(nodelay(window.as_ptr(), true)) }, "nodelay", error_alloc)?;
                match getch(window) {
                    Some(Right(c)) if c < ' ' || c == '\x7F' => None,
                    Some(Right(c)) => Some(Event::Key(ONCE, Key::Alt(c))),
                    Some(Left(_)) => Some(Event::Key(ONCE, Key::Escape)),
                    None => Some(Event::Key(ONCE, Key::Escape)),
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
