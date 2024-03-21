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
        print_no_std::eprintln!("{e}");
        1
    } else {
        0
    }
}

mod floating_frame;

mod ui {
    include!(concat!(env!("OUT_DIR"), "/ui.rs"));
}

use alloc::boxed::Box;
use timer_no_std::MonoClock;
use tuifw_screen::{Error, Key, Vector};
use tuifw_window::{Event, EventHandler, Window, WindowTree, App};
use tuifw::Canvas;

struct State {
    floating_frame: Window,
}

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
        state: &mut dyn App
    ) -> bool {
        let state = state.downcast_mut::<State>().unwrap();
        if let Event::Key(Key::Escape) = event {
            tree.quit();
            return true;
        }
        let offset = match event {
            Event::Key(Key::Left) | Event::Key(Key::Char('h')) =>
                Some(Vector { x: -2, y: 0 }),
            Event::Key(Key::Right) | Event::Key(Key::Char('l')) =>
                Some(Vector { x: 2, y: 0 }),
            Event::Key(Key::Up) | Event::Key(Key::Char('k')) =>
                Some(Vector { x: 0, y: -1 }),
            Event::Key(Key::Down) | Event::Key(Key::Char('j')) =>
                Some(Vector { x: 0, y: 1 }),
            _ => None
        };
        if let Some(offset) = offset {
            Canvas::set_tl(tree, state.floating_frame, Canvas::tl(tree, state.floating_frame).offset(offset));
            true
        } else {
            false
        }
    }
}

fn start() -> Result<(), Error> {
    let clock = unsafe { MonoClock::new() };
    let screen = unsafe { tuifw_screen::init(None, None) }?;
    let tree = &mut WindowTree::new(screen, &clock)?;
    let names = ui::build(tree)?;
    names.root.set_event_handler(tree, Some(Box::new(RootEventHandler)));
    let state = &mut State {
        floating_frame: names.floating_frame,
    };
    tree.run(state)
}
