#![feature(start)]

#![windows_subsystem = "windows"]

#![deny(warnings)]

#![no_std]

extern crate alloc;
extern crate rlibc;

mod no_std {
    use composable_allocators::{AsGlobal, System};

    #[global_allocator]
    static ALLOCATOR: AsGlobal<System> = AsGlobal(System);

    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! { panic_no_std::panic(info, b'P') }

    #[no_mangle]
    extern fn rust_eh_personality() { }
}

mod ui {
    include!(concat!(env!("OUT_DIR"), "/ui.rs"));
}

use alloc::boxed::Box;
use timer_no_std::MonoClock;
use tuifw_screen::{Error, Key};
use tuifw_window::{Event, EventHandler, Window, WindowTree};

#[derive(Clone)]
struct RootEventHandler;

type State = ();

impl EventHandler<State> for RootEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree<State>,
        _window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        _state: &mut State
    ) -> bool {
        match event {
            Event::Key(_, Key::Escape) => {
                tree.quit();
                true
            },
            _ => false
        }
    }
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
    CodePage::load_or_exit_with_msg(99);
    start_and_print_err()
}

fn start_and_print_err() -> u64 {
    if let Err(e) = start() {
        libc_print::libc_eprintln!("{e}");
        1
    } else {
        0
    }
}

fn start() -> Result<(), Error> {
    let clock = unsafe { MonoClock::new() };
    let screen = unsafe { tuifw_screen::init(None, None) }?;
    let tree = &mut ui::build_tree(screen, &clock)?;
    let root = tree.root();
    root.set_event_handler(tree, Some(Box::new(RootEventHandler)));
    let a = tree.window_by_tag(1).unwrap();
    a.focus(tree, true, &mut ());
    tree.run(&mut ())
}
