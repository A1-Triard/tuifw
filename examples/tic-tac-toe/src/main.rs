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
        libc_print::libc_eprintln!("{e}");
        1
    } else {
        0
    }
}

mod ui {
    include!(concat!(env!("OUT_DIR"), "/ui.rs"));
}

use alloc::boxed::Box;
use alloc::vec::Vec;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use timer_no_std::MonoClock;
use tuifw_screen::{Error, Key};
use tuifw_window::{Event, EventHandler, Window, WindowTree};
use tuifw::{Button, CMD_BUTTON_CLICK};

struct State {
    squares: [Window<State>; 9],
    rng: SmallRng,
}

#[derive(Clone)]
struct RootEventHandler;

impl EventHandler<State> for RootEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree<State>,
        _window: Window<State>,
        event: Event,
        event_source: Window<State>,
        state: &mut State
    ) -> bool {
        match event {
            Event::Key(Key::Escape) => {
                tree.quit();
                true
            },
            Event::Cmd(CMD_BUTTON_CLICK) => {
                Button::set_text(tree, event_source, "o");
                event_source.set_is_enabled(tree, false);
                let enabled_squares = state.squares.iter().cloned().filter(|x| x.is_enabled(tree)).collect::<Vec<_>>();
                if !enabled_squares.is_empty() {
                    let ai_move = state.rng.gen_range(0 .. enabled_squares.len());
                    let ai_move = enabled_squares[ai_move];
                    Button::set_text(tree, ai_move, "x");
                    ai_move.set_is_enabled(tree, false);
                    if let Some(enabled_square) = state.squares.iter().find(|x| x.is_enabled(tree)) {
                        enabled_square.set_focused_primary(tree, true);
                    }
                }
                true
            },
            _ => false
        }
    }
}

fn start() -> Result<(), Error> {
    let clock = unsafe { MonoClock::new() };
    let screen = unsafe { tuifw_screen::init(None, None) }?;
    let tree = &mut ui::build_tree(screen, &clock)?;
    let root = tree.root();
    root.set_event_handler(tree, Some(Box::new(RootEventHandler)));
    let state = &mut State {
        squares: [
            tree.window_by_tag(7).unwrap(),
            tree.window_by_tag(8).unwrap(),
            tree.window_by_tag(9).unwrap(),
            tree.window_by_tag(4).unwrap(),
            tree.window_by_tag(5).unwrap(),
            tree.window_by_tag(6).unwrap(),
            tree.window_by_tag(1).unwrap(),
            tree.window_by_tag(2).unwrap(),
            tree.window_by_tag(3).unwrap(),
        ],
        rng: SmallRng::from_entropy(),
    };
    tree.run(state)
}
