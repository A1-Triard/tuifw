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

    #[cfg(windows)]
    #[no_mangle]
    fn rust_oom(_layout: core::alloc::Layout) -> ! {
        exit(98)
    }
}

use alloc::boxed::Box;
use core::hint::black_box;
use tuifw::WindowManager;
use tuifw_screen::Vector;
use tuifw_window::{RenderPort, WindowTree, Window};

fn render<State: ?Sized>(_: &WindowTree<State>, _: Option<Window>, _: &mut RenderPort, _: &mut State) { }

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let _ = black_box(tuifw_screen::init);
    let screen = Box::new(tuifw_screen_test::Screen::new(Vector { x: 80, y: 25 }));
    let _windows = WindowTree::<()>::new(screen, render);
    let _window_manager = WindowManager::<()>::new();
    0
}
