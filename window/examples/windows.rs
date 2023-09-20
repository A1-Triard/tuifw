#![deny(warnings)]
#![allow(unused_variables)]

use core::cmp::min;
use core::mem::replace;
use tuifw_screen::{HAlign, VAlign, Bg, Fg};
use tuifw_screen::{Event, Key, Point, Range1d, Rect, Thickness, Vector};
use tuifw_window::*;
use unicode_width::UnicodeWidthStr;

struct State {
    window_1: Window<()>,
    window_2: Window<()>,
    window_3: Window<()>,
    focused: Window<()>,
}

fn render(
    tree: &WindowTree<(), State>,
    window: Option<Window<()>>,
    rp: &mut RenderPort,
    state: &mut State,
) {
    if let Some(window) = window {
        let bounds = window.bounds(tree);
        let bounds = bounds.relative_to(bounds.tl);
        let (title, content, cursor) = if window == state.window_1 {
            ("1", "First Window", false)
        } else if window == state.window_2 {
            ("2", "Second Window", true)
        } else if window == state.window_3 {
            ("3", "Third Window", false)
        } else {
            unreachable!()
        };
        let focused = state.focused == window;
        rp.fill(|rp, p| rp.out(p, Fg::White, Bg::Blue, " "));
        let (tl, t, tr, r, br, b, bl, l) = if focused {
            ("╔", "═", "╗", "║", "╝", "═", "╚", "║")
        } else {
            ("┌", "─", "┐", "│", "┘", "─", "└", "│")
        };
        rp.out(bounds.tl, Fg::White, Bg::Blue, tl);
        rp.out(bounds.tr_inner(), Fg::White, Bg::Blue, tr);
        rp.out(bounds.br_inner(), Fg::White, Bg::Blue, br);
        rp.out(bounds.bl_inner(), Fg::White, Bg::Blue, bl);
        let border_thickness = Thickness::all(1);
        let content_bounds = border_thickness.shrink_rect(bounds);
        for x in Range1d::new(content_bounds.l(), content_bounds.r()) {
            rp.out(Point { x, y: bounds.t() }, Fg::White, Bg::Blue, t);
            rp.out(Point { x, y: bounds.b_inner() }, Fg::White, Bg::Blue, b);
        }
        for y in Range1d::new(content_bounds.t(), content_bounds.b()) {
            rp.out(Point { x: bounds.l(), y }, Fg::White, Bg::Blue, l);
            rp.out(Point { x: bounds.r_inner(), y }, Fg::White, Bg::Blue, r);
        }
        let title_tl = Thickness::align(
            Vector { x: 1, y: 1 },
            bounds.t_line().size,
            HAlign::Center,
            VAlign::Top
        ).shrink_rect(bounds.t_line()).tl;
        rp.out(title_tl, Fg::White, Bg::Blue, title);
        let content_width = min(u16::MAX as usize, content.width()) as u16 as i16;
        let content_tl = Thickness::align(
            Vector { x: content_width, y: 1 },
            content_bounds.size,
            HAlign::Center,
            VAlign::Center
        ).shrink_rect(content_bounds).tl;
        rp.out(content_tl, Fg::White, Bg::Blue, content);
        if cursor {
            rp.cursor(Point { x: 1, y: 1 })
        }
    } else {
        rp.fill(|rp, p| rp.out(p, Fg::White, Bg::None, " "));
    }
}

fn focus_window(tree: &mut WindowTree<(), State>, window: Window<()>, state: &mut State) {
    let prev = replace(&mut state.focused, window);
    if prev != window {
        window.move_z(tree, Some(prev));
    }
}

fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let mut windows = WindowTree::new(screen, render).unwrap();
    let window_1 = Window::new(&mut windows, (), None, None).unwrap();
    window_1.move_xy(&mut windows, Rect::from_tl_br(Point { x: 5, y: 0}, Point { x: 40, y: 15 }));
    let window_2 = Window::new(&mut windows, (), None, None).unwrap();
    window_2.move_xy(&mut windows, Rect::from_tl_br(Point { x: 30, y: 5}, Point { x: 62, y: 20 }));
    let window_3 = Window::new(&mut windows, (), None, Some(window_2)).unwrap();
    window_3.move_xy(&mut windows, Rect::from_tl_br(Point { x: 20, y: 10}, Point { x: 50, y: 22 }));
    let mut state = State { window_1, window_2, window_3, focused: window_1 };
    loop { 
        if let Some(event) = WindowTree::update(&mut windows, true, &mut state).unwrap() {
            match event {
                Event::Key(_, Key::Escape) => break,
                Event::Key(_, Key::Char('1')) | Event::Key(_, Key::Alt('1')) =>
                    focus_window(&mut windows, window_1, &mut state),
                Event::Key(_, Key::Char('2')) | Event::Key(_, Key::Alt('2')) =>
                    focus_window(&mut windows, window_2, &mut state),
                Event::Key(_, Key::Char('3')) | Event::Key(_, Key::Alt('3')) =>
                    focus_window(&mut windows, window_3, &mut state),
                Event::Key(n, Key::Left) | Event::Key(n, Key::Char('h')) => {
                    let offset = Vector { x: (n.get() as i16).wrapping_neg(), y: 0 };
                    let bounds = state.focused.bounds(&windows);
                    state.focused.move_xy(&mut windows, bounds.offset(offset));
                },
                Event::Key(n, Key::Right) | Event::Key(n, Key::Char('l')) => {
                    let offset = Vector { x: n.get() as i16, y: 0 };
                    let bounds = state.focused.bounds(&windows);
                    state.focused.move_xy(&mut windows, bounds.offset(offset));
                },
                Event::Key(n, Key::Up) | Event::Key(n, Key::Char('k')) => {
                    let offset = Vector { x: 0, y: (n.get() as i16).wrapping_neg() };
                    let bounds = state.focused.bounds(&windows);
                    state.focused.move_xy(&mut windows, bounds.offset(offset));
                },
                Event::Key(n, Key::Down) | Event::Key(n, Key::Char('j')) => {
                    let offset = Vector { x: 0, y: n.get() as i16 };
                    let bounds = state.focused.bounds(&windows);
                    state.focused.move_xy(&mut windows, bounds.offset(offset));
                },
                _ => { },
            }
        }
    }
}
