#![feature(iter_advance_by)]
#![feature(stmt_expr_attributes)]
#![feature(trusted_len)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_map)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]

#![no_std]

use core::num::NonZeroU16;
use core::ops::Range;
use core::option::{Option};
use errno_no_std::Errno;
use enum_derive_2018::{EnumDisplay, EnumFromStr, IterVariants};
use macro_attr_2018::macro_attr;

pub use int_vec_2d::*;

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!)]
    pub enum Orient {
        Hor,
        Vert
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!, IterVariants!(BgVariants))]
    pub enum Bg {
        None,
        Black,
        Red,
        Green,
        Brown,
        Blue,
        Magenta,
        Cyan,
        LightGray
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!, IterVariants!(FgVariants))]
    pub enum Fg {
        Black,
        Red,
        Green,
        Brown,
        Blue,
        Magenta,
        Cyan,
        LightGray,
        DarkGray,
        LightRed,
        LightGreen,
        Yellow,
        LightBlue,
        LightMagenta,
        LightCyan,
        White
    }
}

impl TryFrom<Bg> for Fg {
    type Error = ();

    fn try_from(bg: Bg) -> Result<Fg, ()> {
        match bg {
            Bg::None => Err(()),
            Bg::Black => Ok(Fg::Black),
            Bg::Red => Ok(Fg::Red),
            Bg::Green => Ok(Fg::Green),
            Bg::Brown => Ok(Fg::Brown),
            Bg::Blue => Ok(Fg::Blue),
            Bg::Magenta => Ok(Fg::Magenta),
            Bg::Cyan => Ok(Fg::Cyan),
            Bg::LightGray => Ok(Fg::LightGray),
        }
    }
}

impl TryFrom<Fg> for Bg {
    type Error = ();

    fn try_from(fg: Fg) -> Result<Bg, ()> {
        match fg {
            Fg::Black => Ok(Bg::Black),
            Fg::Red => Ok(Bg::Red),
            Fg::Green => Ok(Bg::Green),
            Fg::Brown => Ok(Bg::Brown),
            Fg::Blue => Ok(Bg::Blue),
            Fg::Magenta => Ok(Bg::Magenta),
            Fg::Cyan => Ok(Bg::Cyan),
            Fg::LightGray => Ok(Bg::LightGray),
            _ => Err(()),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
pub enum Ctrl {
    At, A, B, C, D, E, F, G, J, K, L, N,
    O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Backslash, Bracket, Caret, Underscore
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
#[non_exhaustive]
pub enum Key {
    Char(char),
    Alt(char),
    Ctrl(Ctrl),
    Enter,
    Escape,
    Down,
    Up,
    Left,
    Right,
    Home,
    End,
    Backspace,
    Delete,
    Insert,
    PageDown,
    PageUp,
    Tab,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
#[non_exhaustive]
pub enum Event {
    Resize,
    Key(NonZeroU16, Key),
}

pub trait Screen {
    fn size(&self) -> Vector;

    fn out(
        &mut self,
        p: Point,
        fg: Fg,
        bg: Bg,
        text: &str,
        hard: Range<i16>,
        soft: Range<i16>,
    ) -> Range<i16>;

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Errno>;
}
