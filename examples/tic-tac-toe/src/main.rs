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
use alloc::vec::Vec;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use timer_no_std::MonoClock;
use tuifw_screen::Error;
use tuifw_window::{Event, EventHandler, Window, WindowTree, Visibility, App};
use tuifw::{Button, CMD_BUTTON_CLICK, StaticText, Label};

const CMD_NEW_GAME: u16 = 1000;
const CMD_EXIT: u16 = 1001;

struct State {
    squares: [Window; 9],
    res: Window,
    res_text: Window,
    rng: SmallRng,
}

impl App for State { }

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

impl State {
    fn new_game(&mut self, tree: &mut WindowTree) {
        self.res.set_visibility(tree, Visibility::Hidden);
        for &square in &self.squares {
            square.set_is_enabled(tree, true);
            Button::set_text(tree, square, " ");
        }
        self.ai_move(tree);
    }

    fn ai_move(&mut self, tree: &mut WindowTree) {
        let enabled_squares = self.squares.iter().cloned().filter(|x| x.is_enabled(tree)).collect::<Vec<_>>();
        if !enabled_squares.is_empty() {
            let ai_move = self.rng.gen_range(0 .. enabled_squares.len());
            let ai_move = enabled_squares[ai_move];
            Button::set_text(tree, ai_move, "x");
            ai_move.set_is_enabled(tree, false);
            ai_move.set_focused_primary(tree, true);
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
            StaticText::set_text(tree, self.res_text, result);
            self.res.set_visibility(tree, Visibility::Visible);
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
        state: &mut dyn App
    ) -> bool {
        let state = state.downcast_mut::<State>().unwrap();
        match event {
            Event::Cmd(CMD_EXIT) => {
                tree.quit();
                true
            },
            Event::Cmd(CMD_NEW_GAME) => {
                state.new_game(tree);
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
    let tree = &mut WindowTree::new(screen, &clock)?;
    let names = ui::build(tree)?;
    names.root.set_event_handler(tree, Some(Box::new(RootEventHandler)));
    let state = &mut State {
        squares: [
            names.tl,
            names.t,
            names.tr,
            names.l,
            names.c,
            names.r,
            names.bl,
            names.b,
            names.br,
        ],
        res: names.res,
        res_text: names.res_text,
        rng: SmallRng::from_entropy(),
    };
    Label::set_cmd(tree, names.new_game, CMD_NEW_GAME);
    Label::set_cmd(tree, names.exit, CMD_EXIT);
    state.ai_move(tree);
    tree.run(state, None)
}
