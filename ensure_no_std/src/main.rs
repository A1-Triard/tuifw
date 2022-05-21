#![feature(default_alloc_error_handler)]
#![feature(start)]

#![deny(warnings)]

#![no_std]

use core::alloc::Layout;
use core::panic::PanicInfo;
use dep_obj::binding::Bindings;
use dyn_context::{State, Stop};
#[cfg(not(windows))]
use libc::exit;
use libc_alloc::LibcAlloc;
use tuifw::WidgetTree;
#[cfg(windows)]
use winapi::shared::minwindef::UINT;
#[cfg(windows)]
use winapi::um::processthreadsapi::ExitProcess;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

#[cfg(windows)]
unsafe fn exit(code: UINT) -> ! {
    ExitProcess(code);
    loop { }
}

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    unsafe { exit(99) }
}

#[no_mangle]
pub fn rust_oom(_layout: Layout) -> ! {
    unsafe { exit(98) }
}

#[derive(State, Stop)]
struct App {
    #[state(part)]
    bindings: Bindings,
    #[state]
    widgets: WidgetTree,
}

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let mut bindings = Bindings::new();
    let widgets = WidgetTree::new(screen, &mut bindings);
    let app = &mut App { bindings, widgets };
    App::stop(app);
    0
}
