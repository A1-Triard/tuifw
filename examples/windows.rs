#![feature(explicit_generic_args_with_impl_trait)]

#![deny(warnings)]
#![allow(unused_variables)]

use tuifw_screen::{Attr, Color, Event, Key, Point, Rect};
use tuifw_window::*;

struct State {
    window_1: Window,
    window_2: Window,
    window_3: Window,
    focused: Window,
}

fn render(
    tree: &WindowTree<State>,
    window: Option<Window>,
    port: &mut RenderPort,
    state: &mut State,
) {
    if let Some(window) = window {
        let size = window.bounds(tree).size;
        let title = if window == state.window_1 {
            "1"
        } else if window == state.window_2 {
            "2"
        } else if window == state.window_3 {
            "3"
        } else {
            unreachable!()
        };
        let focused = state.focused == window;
        port.fill(|port, p| port.out(p, Color::White, Some(Color::Blue), Attr::empty(), " "));
        let (tl, t, tr, r, br, b, bl, l) = if focused {
            ("╔", "═", "╗", "║", "╝", "═", "╚", "║")
        } else {
            ("┌", "─", "┐", "│", "┘", "─", "└", "│")
        };
        port.out(Point { x: 0, y: 0 }, Color::White, Some(Color::Blue), Attr::empty(), tl);
        port.out(Point { x: size.x - 1, y: size.y - 1 }, Color::White, Some(Color::Blue), Attr::empty(), br);
    }
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let mut windows = WindowTree::new(screen, render);
    let window_1 = Window::new(
        &mut windows, None, None, 
        Rect::from_tl_br(Point { x: 5, y: 0}, Point { x: 40, y: 15 })
    );
    let window_2 = Window::new(
        &mut windows, None, Some(window_1), 
        Rect::from_tl_br(Point { x: 30, y: 5}, Point { x: 62, y: 20 })
    );
    let window_3 = Window::new(
        &mut windows, None, Some(window_1), 
        Rect::from_tl_br(Point { x: 20, y: 10}, Point { x: 50, y: 22 })
    );
    let mut state = State { window_1, window_2, window_3, focused: window_1 };
    loop { 
        let event = WindowTree::update(&mut windows, true, &mut state).unwrap().unwrap();
        match event {
            Event::Key(_, Key::Escape) => break,
            _ => { },
        }
    }
}
