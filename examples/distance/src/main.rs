#![feature(extern_types)]
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
    extern "C" fn rust_eh_personality() { }
}

#[start]
fn main(_: isize, _: *const *const u8) -> isize {
    start_and_print_err() as _
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
use alloc::string::ToString;
use core::mem::replace;
use core::str::FromStr;
use timer_no_std::MonoClock;
use tuifw_screen::{Error, Key};
use tuifw_window::{Event, EventHandler, Window, WindowTree, App};
use tuifw::{Button, InputLine, StaticText, CMD_INPUT_LINE_IS_VALID_CHANGED};

const CMD_CALC: u16 = 1000;

struct State {
    a: Window,
    v: Window,
    t: Window,
    n: Window,
    s: Window,
    calc: Window,
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
        match event {
            Event::Key(Key::Escape) => {
                tree.quit();
                true
            },
            Event::Cmd(CMD_CALC) => {
                let a = f64::from_str(InputLine::text(tree, state.a)).unwrap();
                let v = f64::from_str(InputLine::text(tree, state.v)).unwrap();
                let t = f64::from_str(InputLine::text(tree, state.t)).unwrap();
                let n = f64::from(i32::from_str(InputLine::text(tree, state.n)).unwrap());
                let s = v * t + a * t * (n - 1.0) / (2.0 * n);
                StaticText::text_mut(tree, state.s, |value| replace(value, s.to_string()));
                true
            },
            Event::Cmd(CMD_INPUT_LINE_IS_VALID_CHANGED) => {
                let a_valid = InputLine::is_valid(tree, state.a);
                let v_valid = InputLine::is_valid(tree, state.v);
                let t_valid = InputLine::is_valid(tree, state.t);
                let n_valid = InputLine::is_valid(tree, state.n);
                state.calc.set_is_enabled(tree, a_valid && v_valid && t_valid && n_valid);
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
    let state = &mut State {
        a: names.a,
        v: names.v,
        t: names.t,
        n: names.n,
        s: names.s,
        calc: names.calc,
    };
    Button::set_cmd(tree, names.calc, CMD_CALC);
    tree.run(state, None)
}
