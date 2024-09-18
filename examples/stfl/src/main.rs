#![feature(extern_types)]
#![feature(start)]

#![windows_subsystem = "windows"]

#![deny(warnings)]

#![no_std]

#![cfg_attr(any(target_os="dos", windows), no_main)]

extern crate alloc;
#[cfg(target_os="dos")]
extern crate pc_atomics;
extern crate rlibc_ext;

#[cfg(all(windows, not(target_os="dos")))]
#[link(name="msvcrt")]
extern { }

mod no_std {
    #[cfg(not(target_os="dos"))]
    use composable_allocators::{AsGlobal, System};
    #[cfg(target_os="dos")]
    use composable_allocators::{AsGlobal, freelist_allocator_128_KiB_align_8};

    #[cfg(not(target_os="dos"))]
    #[global_allocator]
    static ALLOCATOR: AsGlobal<System> = AsGlobal(System);

    #[cfg(target_os="dos")]
    freelist_allocator_128_KiB_align_8!(FREELIST: Freelist);

    #[cfg(target_os="dos")]
    #[global_allocator]
    static ALLOCATOR: AsGlobal<&'static Freelist> = AsGlobal(&FREELIST);

    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! { panic_no_std::panic(info, b'P') }

    #[no_mangle]
    extern "C" fn rust_eh_personality() { }
}

#[cfg(any(target_os="dos", windows))]
extern {
    type PEB;
}

#[cfg(all(not(target_os="dos"), not(windows)))]
#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    start_and_print_err() as _
}

#[cfg(any(target_os="dos", windows))]
#[allow(non_snake_case)]
#[no_mangle]
extern "stdcall" fn mainCRTStartup(_: *const PEB) -> u64 {
    #[cfg(target_os="dos")]
    dos_cp::CodePage::load_or_exit_with_msg(99);
    start_and_print_err()
}

fn start_and_print_err() -> u64 {
    if let Err(e) = start() {
        print_no_std::eprintln!("{e}");
        1
    } else {
        0
    }
}

mod ui {
    include!(concat!(env!("OUT_DIR"), "/ui.rs"));
}

use alloc::boxed::Box;
use timer_no_std::MonoClock;
use tuifw_screen::{Error, Key};
use tuifw_window::{Event, EventHandler, Window, WindowTree, App};

struct State;

impl App for State { }

#[derive(Clone)]
struct RootEventHandler;

impl EventHandler for RootEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree,
        _window: Window,
        event: Event,
        _event_source: Window,
        _state: &mut dyn App,
    ) -> bool {
        match event {
            Event::Key(Key::Escape) => {
                tree.quit();
                true
            },
            _ => false
        }
    }
}

fn start() -> Result<(), Error> {
    let clock = unsafe { MonoClock::new() };
    let screen = unsafe { tuifw_screen::init(None, None) }?;
    let tree = &mut WindowTree::new(screen, &clock)?;
    let names = ui::build(tree)?;
    names.root.set_event_handler(tree, Some(Box::new(RootEventHandler)));
    let state = &mut State;
    tree.run(state)
}
