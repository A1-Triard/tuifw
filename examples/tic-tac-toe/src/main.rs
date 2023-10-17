#![feature(start)]

#![windows_subsystem = "windows"]

//#![deny(warnings)]

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
use tuifw_window::{Event, EventHandler, Window, WindowTree, Visibility, State};
use tuifw::{Button, CMD_BUTTON_CLICK, StaticText};

struct App {
    squares: [Window; 9],
    rng: SmallRng,
}

impl State for App { }

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Winner {
    Player,
    Ai,
    Draw,
}

impl Winner {
    fn from_symbol(s: Symbol) -> Self {
        match s {
            Symbol::Nought => Winner::Player,
            Symbol::Cross => Winner::Ai,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Symbol {
    Nought,
    Cross,
}

impl Symbol {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "x" => Some(Symbol::Cross),
            "o" => Some(Symbol::Nought),
            " " => None,
            _ => panic!(),
        }
    }
}

impl App {
    fn ai_move(&mut self, tree: &mut WindowTree) {
        let enabled_squares = self.squares.iter().cloned().filter(|x| x.is_enabled(tree)).collect::<Vec<_>>();
        if !enabled_squares.is_empty() {
            let ai_move = self.rng.gen_range(0 .. enabled_squares.len());
            let ai_move = enabled_squares[ai_move];
            Button::set_text(tree, ai_move, "x");
            ai_move.set_is_enabled(tree, false);
            if let Some(enabled_square) = self.squares.iter().find(|x| x.is_enabled(tree)) {
                enabled_square.set_focused_primary(tree, true);
            }
        }
    }

    fn winner(&self, tree: &WindowTree) -> Option<Winner> {
        let s7 = Symbol::from_str(Button::text(tree, self.squares[0]));
        let s8 = Symbol::from_str(Button::text(tree, self.squares[1]));
        let s9 = Symbol::from_str(Button::text(tree, self.squares[2]));
        let s4 = Symbol::from_str(Button::text(tree, self.squares[3]));
        let s5 = Symbol::from_str(Button::text(tree, self.squares[4]));
        let s6 = Symbol::from_str(Button::text(tree, self.squares[5]));
        let s1 = Symbol::from_str(Button::text(tree, self.squares[6]));
        let s2 = Symbol::from_str(Button::text(tree, self.squares[7]));
        let s3 = Symbol::from_str(Button::text(tree, self.squares[8]));
        if s7.is_some() && s7 == s8 && s8 == s9 { return Some(Winner::from_symbol(s7.unwrap())); }
        if s4.is_some() && s4 == s5 && s5 == s6 { return Some(Winner::from_symbol(s4.unwrap())); }
        if s1.is_some() && s1 == s2 && s2 == s3 { return Some(Winner::from_symbol(s1.unwrap())); }
        if s1.is_some() && s1 == s4 && s4 == s7 { return Some(Winner::from_symbol(s1.unwrap())); }
        if s2.is_some() && s2 == s5 && s5 == s8 { return Some(Winner::from_symbol(s2.unwrap())); }
        if s3.is_some() && s3 == s6 && s6 == s9 { return Some(Winner::from_symbol(s3.unwrap())); }
        if s1.is_some() && s1 == s5 && s5 == s9 { return Some(Winner::from_symbol(s1.unwrap())); }
        if s7.is_some() && s7 == s5 && s5 == s3 { return Some(Winner::from_symbol(s7.unwrap())); }
        if
            s1.is_some() && s2.is_some() && s3.is_some() &&
            s4.is_some() && s5.is_some() && s6.is_some() &&
            s7.is_some() && s8.is_some() && s9.is_some()
        {
            Some(Winner::Draw)
        } else {
            None
        }
    }

    fn show_winner(&mut self, tree: &mut WindowTree) -> bool {
        if let Some(winner) = self.winner(tree) {
            let result = match winner {
                Winner::Player => "You won!",
                Winner::Ai => "AI won!",
                Winner::Draw => "Draw!",
            };
            let result_window = tree.window_by_tag(11).unwrap();
            StaticText::set_text(tree, result_window, result);
            tree.window_by_tag(10).unwrap().set_visibility(tree, Visibility::Visible);
            true
        } else {
            false
        }
    }
}

#[derive(Clone)]
struct RootEventHandler;

impl EventHandler for RootEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree,
        _window: Window,
        event: Event,
        event_source: Window,
        state: &mut dyn State
    ) -> bool {
        let state = state.downcast_mut::<App>().unwrap();
        match event {
            Event::Key(Key::Escape) => {
                tree.quit();
                true
            },
            Event::Cmd(CMD_BUTTON_CLICK) => {
                Button::set_text(tree, event_source, "o");
                event_source.set_is_enabled(tree, false);
                if !state.show_winner(tree) {
                    state.ai_move(tree);
                    state.show_winner(tree);
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
    let state = &mut App {
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
    state.ai_move(tree);
    tree.run(state)
}
