#![cfg(not(windows))]

#![feature(allocator_api)]
#![feature(extern_types)]
#![feature(negative_impls)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::unnecessary_lazy_evaluations)]

#![no_std]

extern crate alloc;

mod ncurses;

mod common;

mod non_unicode;

mod unicode;

use alloc::alloc::Global;
use arraybox::{ArrayBox, BufFor, Or};
use core::alloc::Allocator;
use libc::{CODESET, setlocale, strcmp, nl_langinfo, LC_ALL};
use macro_attr_2018::macro_attr;
use newtype_derive_2018::{NewtypeDeref, NewtypeDerefMut};
use tuifw_screen_base::Error;
use tuifw_screen_base::Screen as base_Screen;

macro_attr! {
    #[derive(NewtypeDeref!, NewtypeDerefMut!)]
    pub struct Screen<A: Allocator = Global>(
        ArrayBox<'static, dyn base_Screen, BufFor<Or<unicode::Screen<A>, non_unicode::Screen<A>>>>
    );
}

/// # Safety
///
/// This function initializes ncurses lib. It is safe iff no other code in application calls ncurses functions
/// while `Screen` instance is alive. This rule also applies to another `Screen` instance:
/// it is not safe to call `init`/`init_in` again until `Screen` created by previous call is dropped.
///
/// It is impossible to garantee this conditions on a library level.
/// So this unsafity should be propagated through all wrappers to the final application.
pub unsafe fn init() -> Result<Screen, Error> {
    init_in(Global)
}

/// # Safety
///
/// This function initializes ncurses lib. It is safe iff no other code in application calls ncurses functions
/// while `Screen` instance is alive. This rule also applies to another `Screen` instance:
/// it is not safe to call `init`/`init_in` again until `Screen` created by previous call is dropped.
///
/// It is impossible to garantee this conditions on a library level.
/// So this unsafity should be propagated through all wrappers to the final application.
pub unsafe fn init_in<A: Allocator + Clone + 'static>(alloc: A) -> Result<Screen<A>, Error> {
    setlocale(LC_ALL, "\0".as_ptr() as _);
    let unicode = strcmp(nl_langinfo(CODESET), b"UTF-8\0".as_ptr() as _) == 0;
    let screen = if unicode {
        Screen(ArrayBox::new(unicode::Screen::new_in(alloc)?))
    } else {
        Screen(ArrayBox::new(non_unicode::Screen::new_in(alloc)?))
    };
    Ok(screen)
}
