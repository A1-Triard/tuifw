#![feature(start)]

#![deny(warnings)]

#![no_std]

extern crate alloc;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

mod no_std {
    use composable_allocators::{AsGlobal, System};
    use core::panic::PanicInfo;
    use exit_no_std::exit;

    #[global_allocator]
    static ALLOCATOR: AsGlobal<System> = AsGlobal(System);

    #[panic_handler]
    fn panic(_info: &PanicInfo) -> ! {
        exit(99)
    }

    #[no_mangle]
    extern "C" fn rust_eh_personality() { }
}

use alloc::boxed::Box;
use core::hint::black_box;
use timer_no_std::MonoClock;
use tuifw::Background;
use tuifw_screen::Vector;
use tuifw_window::WindowTree;

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let _ = black_box(tuifw_screen::init);
    let clock = unsafe { MonoClock::new() };
    let screen = Box::new(tuifw_screen_test::Screen::new(Vector { x: 80, y: 25 }));
    let tree = &mut WindowTree::new(screen, &clock).unwrap();
    let _root = Background::new(tree, None, None).unwrap();
    0
}
