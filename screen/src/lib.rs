#![feature(allocator_api)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![no_std]

extern crate alloc;

use alloc::alloc::Global;
use alloc::boxed::Box;
use core::alloc::Allocator;

pub use tuifw_screen_base::*;

/// # Safety
///
/// This function may initialize ncurses lib. It is safe iff no other code in application calls ncurses functions
/// while `Screen` instance is alive. This rule also applies to another `Screen` instance:
/// it is not safe to call `init`/`init_in` again until `Screen` created by previous call is dropped.
///
/// Also, iff compiled with `cfg(target_os="dos")` this method may not be invoked until it is guaranteed the memory addresses
/// in `0xB8000 .. 0xBBE80` are not used by Rust abstract machine.
///
/// It is impossible to garantee this conditions on a library level.
/// So this unsafity should be propagated through all wrappers to the final application.
pub unsafe fn init(max_size: Option<(u16, u16)>) -> Result<Box<dyn Screen>, Error> {
    init_raw_in(max_size, Global)
}

/// # Safety
///
/// This function may initialize ncurses lib. It is safe iff no other code in application calls ncurses functions
/// while `Screen` instance is alive. This rule also applies to another `Screen` instance:
/// it is not safe to call `init`/`init_in` again until `Screen` created by previous call is dropped.
///
/// Also, iff compiled with `cfg(target_os="dos")` this method may not be invoked until it is guaranteed the memory addresses
/// in `0xB8000 .. 0xBBE80` are not used by Rust abstract machine.
///
/// It is impossible to garantee this conditions on a library level.
/// So this unsafity should be propagated through all wrappers to the final application.
pub unsafe fn init_in<A: Allocator + Clone + 'static>(max_size: Option<(u16, u16)>, alloc: A) -> Result<Box<dyn Screen>, Error> {
    init_raw_in(max_size, alloc)
}

#[cfg(target_os="dos")]
unsafe fn init_raw_in<A: Allocator + Clone>(max_size: Option<(u16, u16)>, alloc: A) -> Result<Box<dyn Screen>, Error> {
    if let Some(max_size) = max_size {
        assert!(max_size.0 >= 80 && max_size.1 >= 25);
    }
    Ok(Box::new(tuifw_screen_dos::Screen::new_in(alloc)?))
}

#[cfg(all(not(target_os="dos"), windows))]
unsafe fn init_raw_in<A: Allocator + Clone + 'static>(max_size: Option<(u16, u16)>, alloc: A) -> Result<Box<dyn Screen>, Error> {
    Ok(Box::new(tuifw_screen_winapi::Screen::new_in(max_size, alloc)?))
}

#[cfg(all(not(target_os="dos"), not(windows)))]
unsafe fn init_raw_in<A: Allocator + Clone + 'static>(max_size: Option<(u16, u16)>, alloc: A) -> Result<Box<dyn Screen>, Error> {
    Ok(tuifw_screen_ncurses::init_in(max_size, alloc)?)
}
