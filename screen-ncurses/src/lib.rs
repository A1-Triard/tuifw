#![cfg(not(windows))]

#![deny(warnings)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]
#![feature(negative_impls)]
#![feature(extern_types)]
#![feature(internal_uninit_const)]

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
mod ncurses;

mod common;

mod non_unicode;

mod unicode;

use std::io::{self};
use libc::{CODESET, setlocale, strcmp, nl_langinfo, LC_ALL};
use tuifw_screen_base::{Screen};

/// # Safety
///
/// This function initializes ncurses lib. It is safe iff no other code in application calls ncurses functions
/// while `Screen` instance is alive. This rule also applies to another `Screen` instance:
/// it is not safe to call `init` again until `Screen` created by previous call is dropped.
///
/// It is impossible to garantee this conditions on a library level.
/// So this unsafity should be propagated through all wrappers to the final application.
pub unsafe fn init() -> io::Result<Box<dyn Screen>> {
    setlocale(LC_ALL, "\0".as_ptr() as _);
    let unicode = strcmp(nl_langinfo(CODESET), b"UTF-8\0".as_ptr() as _) == 0;
    let screen = if unicode {
        Box::new(unicode::Screen::new()?) as Box<dyn Screen>
    } else {
        Box::new(non_unicode::Screen::new()?) as Box<dyn Screen>
    };
    Ok(screen)
}
