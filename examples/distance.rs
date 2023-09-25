#![windows_subsystem = "windows"]

//#![deny(warnings)]

use either::Right;
use std::mem::replace;
use tuifw::{Background, InputLine, InputLineValueRange, StackPanel, StaticText};
use tuifw_screen::{Bg, Fg, HAlign, VAlign, Key};
use tuifw_window::{Event, EventHandler, Window, WindowTree};

struct State {
    quit: bool,
}

#[derive(Clone)]
struct RootEventHandler;

impl EventHandler<State> for RootEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        preview: bool,
        state: &mut State
    ) -> bool {
        if let Event::Key(_, Key::Escape) = event {
            state.quit = true;
            true
        } else {
            false
        }
    }
}

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut Background::new().window_tree(screen).unwrap();
    let root = tree.root();
    Background::set_show_pattern(tree, root, false);
    root.palette_mut(tree, |palette| palette.set(0, Right((Fg::Black, Bg::None))));
    root.set_event_handler(tree, Some(Box::new(RootEventHandler)));
    /*
    let panel = StackPanel::new().window(tree, root, None).unwrap();
    panel.set_h_align(tree, Some(HAlign::Center));
    panel.set_v_align(tree, Some(VAlign::Center));
    let text = StaticText::new().window(tree, panel, None).unwrap();
    StaticText::text_mut(tree, text, |value| replace(value, "Hello!".to_string()));
    let input = InputLine::new().window(tree, panel, Some(text)).unwrap();
    InputLine::set_value_range(tree, input, InputLineValueRange::Integer(0 ..= i64::MAX));
    input.set_width(tree, 10);
    InputLine::value_mut(tree, input, |value| replace(value, "1111222233334444".to_string()));
    input.focus(tree, &mut ());
    */
    let mut state = State { quit: false };
    while !state.quit {
        tree.update(true, &mut state).unwrap();
    }
}
