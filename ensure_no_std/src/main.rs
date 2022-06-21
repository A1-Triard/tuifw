#![feature(default_alloc_error_handler)]
#![feature(start)]

#![deny(warnings)]

#![no_std]

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

mod no_std {
    use core::panic::PanicInfo;
    use exit_no_std::exit;
    use composable_allocators::{AsGlobal, System};

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

use tuifw_window::{RenderPort, WindowTree, Window};

fn render<State: ?Sized>(_: &WindowTree<State>, _: Option<Window>, _: &mut RenderPort, _: &mut State) { }

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let _windows = WindowTree::<()>::new(screen, render);
    0
}
