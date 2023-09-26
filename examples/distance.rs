#![windows_subsystem = "windows"]

//#![deny(warnings)]

use either::Right;
use std::mem::replace;
use tuifw::{Background, Dock, DockPanel, InputLine, InputLineValueRange, StackPanel, StaticText};
use tuifw_screen::{Bg, Fg, HAlign, VAlign, Key, Thickness};
use tuifw_window::{Event, EventHandler, Window, WindowTree};

struct State {
    quit: bool,
}

#[derive(Clone)]
struct RootEventHandler;

impl EventHandler<State> for RootEventHandler {
    fn invoke(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        event: Event,
        preview: bool,
        state: &mut State
    ) -> bool {
        if !preview {
            if let Event::Key(_, Key::Escape) = event {
                state.quit = true;
                true
            } else {
                false
            }
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
    let panel = DockPanel::new().window(tree, root, None).unwrap();
    panel.set_h_align(tree, Some(HAlign::Center));
    panel.set_v_align(tree, Some(VAlign::Center));
    let labels = StackPanel::new().window(tree, panel, None).unwrap();
    DockPanel::set_layout(tree, labels, Some(Dock::Left));
    let edits = StackPanel::new().window(tree, panel, Some(labels)).unwrap();
    edits.set_width(tree, 12);
    let a_label = StaticText::new().window(tree, labels, None).unwrap();
    StaticText::text_mut(tree, a_label, |value| replace(value, "A:".to_string()));
    a_label.set_margin(tree, Thickness::new(1, 1, 0, 1));
    let a = InputLine::new().window(tree, edits, None).unwrap();
    InputLine::set_value_range(tree, a, InputLineValueRange::Float(f64::from(f32::MIN) ..= f64::from(f32::MAX)));
    InputLine::value_mut(tree, a, |value| replace(value, "0".to_string()));
    a.set_margin(tree, Thickness::new(1, 1, 1, 1));
    let v_label = StaticText::new().window(tree, labels, Some(a_label)).unwrap();
    StaticText::text_mut(tree, v_label, |value| replace(value, "V:".to_string()));
    v_label.set_margin(tree, Thickness::new(1, 0, 0, 1));
    let v = InputLine::new().window(tree, edits, Some(a)).unwrap();
    InputLine::set_value_range(tree, v, InputLineValueRange::Float(f64::from(f32::MIN) ..= f64::from(f32::MAX)));
    InputLine::value_mut(tree, v, |value| replace(value, "1".to_string()));
    v.set_margin(tree, Thickness::new(1, 0, 1, 1));
    let mut state = State { quit: false };
    a.focus(tree, &mut state);
    while !state.quit {
        tree.update(true, &mut state).unwrap();
    }
}
