#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![no_std]

extern crate alloc;

use alloc::boxed::Box;

pub use tuifw_screen_base::*;

/// # Safety
///
/// This function may initialize ncurses lib. It is safe iff no other code in application calls ncurses functions
/// while `Screen` instance is alive. This rule also applies to another `Screen` instance:
/// it is not safe to call `init` again until `Screen` created by previous call is dropped.
///
/// Also, iff compiled with `cfg(target_os="dos")` this method may not be invoked until it is guaranteed the memory addresses
/// in `0xB8000 .. 0xBBE80` are not used by Rust abstract machine.
///
/// It is impossible to garantee this conditions on a library level.
/// So this unsafity should be propagated through all wrappers to the final application.
pub unsafe fn init() -> Result<Box<dyn Screen>, Error> {
    init_raw()
}

#[cfg(target_os="dos")]
unsafe fn init_raw() -> Result<Box<dyn Screen>, Error> {
    Ok(Box::new(tuifw_screen_dos::Screen::new()?) as _)
}

#[cfg(all(not(target_os="dos"), windows))]
unsafe fn init_raw() -> Result<Box<dyn Screen>, Error> {
    Ok(Box::new(tuifw_screen_winapi::Screen::new()?) as _)
}

#[cfg(all(not(target_os="dos"), not(windows)))]
unsafe fn init_raw() -> Result<Box<dyn Screen>, Error> {
    tuifw_screen_ncurses::init()
}
