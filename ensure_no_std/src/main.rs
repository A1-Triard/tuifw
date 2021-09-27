#![feature(default_alloc_error_handler)]
#![feature(start)]

#![no_std]

use core::alloc::Layout;
use core::any::{Any, TypeId};
use core::panic::PanicInfo;
use dep_obj::binding::Bindings;
use dyn_context::state::State;
#[cfg(not(windows))]
use libc::exit;
use libc_alloc::LibcAlloc;
use tuifw::WidgetTree;
#[cfg(windows)]
use winapi::um::processthreadsapi::ExitProcess;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[global_allocator]
static ALLOCATOR: LibcAlloc = LibcAlloc;

#[cfg(windows)]
unsafe fn exit(code: UINT) -> ! {
    unsafe { ExitProcess(code); }
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

struct App {
    bindings: Bindings,
    widgets: WidgetTree,
}

impl State for App {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if let Some(res) = self.bindings.get_raw(ty) { return Some(res); }
        if let Some(res) = self.widgets.get_raw(ty) { return Some(res); }
        None
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if let Some(res) = self.bindings.get_mut_raw(ty) { return Some(res); }
        if let Some(res) = self.widgets.get_mut_raw(ty) { return Some(res); }
        None
    }
}

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let mut bindings = Bindings::new();
    let widgets = WidgetTree::new(screen, &mut bindings);
    let app = &mut App { bindings, widgets };
    WidgetTree::drop_self(app);
    0
}
