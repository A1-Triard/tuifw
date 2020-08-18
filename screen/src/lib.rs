use std::io::{self};
use tuifw_screen_base::{Screen};

/// # Safety
///
/// This function initializes ncurses lib. It is safe iff no other code in application calls ncurses functions
/// while `Screen` instance is alive. This rule also applies to another `Screen` instance:
/// it is not safe to call `init` again until `Screen` created by previous call is dropped.
///
/// It is impossible to garantee this conditions on a library level.
/// So this unsafity should be propagated through all wrappers to the final application.
#[cfg(windows)]
pub unsafe fn init() -> io::Result<Box<dyn Screen<Error=io::Error>>> {
    Ok(Box::new(tuifw_screen_winapi::Screen::new()?) as Box<dyn Screen<Error=io::Error>>)
}

/// # Safety
///
/// This function initializes ncurses lib. It is safe iff no other code in application calls ncurses functions
/// while `Screen` instance is alive. This rule also applies to another `Screen` instance:
/// it is not safe to call `init` again until `Screen` created by previous call is dropped.
///
/// It is impossible to garantee this conditions on a library level.
/// So this unsafity should be propagated through all wrappers to the final application.
#[cfg(not(windows))]
pub unsafe fn init() -> io::Result<Box<dyn Screen<Error=io::Error>>> {
    tuifw_screen_ncurses::init()
}
