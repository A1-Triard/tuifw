#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use libc::*;

include!(concat!(env!("OUT_DIR"), "/curses_types.rs"));

pub const COLOR_BLACK: c_short = 0;
pub const COLOR_RED: c_short = 1;
pub const COLOR_GREEN: c_short = 2;
pub const COLOR_YELLOW: c_short = 3;
pub const COLOR_BLUE: c_short = 4;
pub const COLOR_MAGENTA: c_short = 5;
pub const COLOR_CYAN: c_short = 6;
pub const COLOR_WHITE: c_short = 7;

pub const ERR: c_int = -1;

pub const A_NORMAL: chtype = 0;
pub const A_BOLD: chtype = 1 << 21;
pub const A_ALTCHARSET: chtype = 1 << 22;

pub const WA_NORMAL: attr_t = A_NORMAL;

pub const KEY_CODE_YES: c_int = 256;
pub const KEY_DOWN: c_int = 258;
pub const KEY_UP: c_int = 259;
pub const KEY_LEFT: c_int = 260;
pub const KEY_RIGHT: c_int = 261;
pub const KEY_HOME: c_int = 262;
pub const KEY_BACKSPACE: c_int = 263;
pub const KEY_F0: c_int = 264;
pub const fn KEY_F(n: c_int) -> c_int { KEY_F0 + n }
pub const KEY_DC: c_int = 330;
pub const KEY_IC: c_int = 331;
pub const KEY_NPAGE: c_int = 338;
pub const KEY_PPAGE: c_int = 339;
pub const KEY_END: c_int = 360;
pub const KEY_RESIZE: c_int = 410;
pub const KEY_MOUSE: c_int = 0o631;

extern {
    pub type WINDOW;
}

pub type attr_t = chtype;

extern "C" {
    pub static mut curscr: *mut WINDOW;
    pub static mut stdscr: *mut WINDOW;
    pub static mut COLS: c_int;
    pub static mut LINES: c_int;
}

#[allow(clippy::upper_case_acronyms)]
#[repr(C)]
pub struct MEVENT {
    pub id: c_short,
    pub x: c_int,
    pub y: c_int,
    pub z: c_int,
    pub bstate: c_ulong,
}

extern "C" {
    pub fn COLOR_PAIR(arg1: c_int) -> c_int;
    #[must_use]
    pub fn cbreak() -> c_int;
    #[must_use]
    pub fn nonl() -> c_int;
    pub fn clearok(arg1: *mut WINDOW, arg2: bool) -> c_int;
    #[must_use]
    pub fn curs_set(arg1: c_int) -> c_int;
    #[must_use]
    pub fn delwin(arg1: *mut WINDOW) -> c_int;
    #[must_use]
    pub fn doupdate() -> c_int;
    #[must_use]
    pub fn endwin() -> c_int;
    #[must_use]
    pub fn newwin(
        arg1: c_int,
        arg2: c_int,
        arg3: c_int,
        arg4: c_int,
    ) -> *mut WINDOW;
    #[must_use]
    pub fn initscr() -> *mut WINDOW;
    #[must_use]
    pub fn init_pair(
        arg1: c_short,
        arg2: c_short,
        arg3: c_short,
    ) -> c_int;
    #[must_use]
    pub fn keypad(arg1: *mut WINDOW, arg2: bool) -> c_int;
    #[must_use]
    pub fn nodelay(arg1: *mut WINDOW, arg2: bool) -> c_int;
    #[must_use]
    pub fn noecho() -> c_int;
    #[must_use]
    pub fn start_color() -> c_int;
    #[must_use]
    pub fn waddch(arg1: *mut WINDOW, arg2: chtype) -> c_int;
    #[must_use]
    pub fn wattrset(arg1: *mut WINDOW, arg2: c_int) -> c_int;
    pub fn wgetch(arg1: *mut WINDOW) -> c_int;
    #[must_use]
    pub fn wmove(
        arg1: *mut WINDOW,
        arg2: c_int,
        arg3: c_int,
    ) -> c_int;
    #[must_use]
    pub fn wnoutrefresh(arg1: *mut WINDOW) -> c_int;
    #[must_use]
    pub fn use_default_colors() -> c_int;
    pub fn set_escdelay(arg1: c_int) -> c_int;
    #[must_use]
    pub fn waddnwstr(
        arg1: *mut WINDOW,
        arg2: *const wchar_t,
        arg3: c_int,
    ) -> c_int;
    pub fn wget_wch(arg1: *mut WINDOW, arg2: *mut wint_t) -> c_int;
    pub fn mousemask(newmask: c_ulong, oldmask: *mut c_ulong) -> c_ulong;
    pub fn getmouse(event: *mut MEVENT) -> c_int;
}
