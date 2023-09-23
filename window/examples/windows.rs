#![deny(warnings)]
#![allow(unused_variables)]

use core::cmp::min;
use tuifw_screen::{HAlign, VAlign, Bg, Fg};
use tuifw_screen::{Key, Point, Range1d, Rect, Thickness, Vector};
use tuifw_window::*;
use unicode_width::UnicodeWidthStr;

struct State {
    window_1: Window<()>,
    window_2: Window<()>,
    window_3: Window<()>,
    quit: bool,
}

fn measure(
    tree: &mut WindowTree<(), State>,
    window: Window<()>,
    available_width: Option<i16>,
    available_height: Option<i16>,
    state: &mut State
) -> Vector {
    if window == state.window_1 {
        Vector { x: 35, y: 15 }
    } else if window == state.window_2 {
        Vector { x: 32, y: 15 }
    } else if window == state.window_3 {
        Vector { x: 30, y: 15 }
    } else {
        let first_child = window.first_child(tree).unwrap();
        let mut child = first_child;
        loop {
            child.measure(tree, None, None, state);
            child = child.next(tree);
            if child == first_child { break; }
        }
        Vector { x: available_width.unwrap(), y: available_height.unwrap() }
    }
}

fn arrange(
    tree: &mut WindowTree<(), State>,
    window: Window<()>,
    final_inner_bounds: Rect,
    state: &mut State
) -> Vector {
    if window == state.window_1 {
        final_inner_bounds.size
    } else if window == state.window_2 {
        final_inner_bounds.size
    } else if window == state.window_3 {
        final_inner_bounds.size
    } else {
        let first_child = window.first_child(tree).unwrap();
        let mut child = first_child;
        loop {
            let child_desired_size = child.desired_size(tree);
            let bounds = if child == state.window_1 {
                Rect { tl: Point { x: 5, y: 0 }, size: child_desired_size }
            } else if child == state.window_2 {
                Rect { tl: Point { x: 30, y: 5 }, size: child_desired_size }
            } else if child == state.window_3 {
                Rect { tl: Point { x: 20, y: 10 }, size: child_desired_size }
            } else {
                unreachable!()
            };
            child.arrange(tree, bounds, state);
            child = child.next(tree);
            if child == first_child { break; }
        }
        final_inner_bounds.size
    }
}

fn render(
    tree: &WindowTree<(), State>,
    window: Window<()>,
    rp: &mut RenderPort,
    state: &mut State,
) {
    let bounds = window.bounds(tree);
    let bounds = bounds.relative_to(bounds.tl);
    let (title, content, cursor) = if window == state.window_1 {
        ("1", "First Window", false)
    } else if window == state.window_2 {
        ("2", "Second Window", true)
    } else if window == state.window_3 {
        ("3", "Third Window", false)
    } else {
        rp.fill(|rp, p| rp.out(p, Fg::White, Bg::None, " "));
        return;
    };
    let focused = tree.focused() == window;
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
}

fn update(
    tree: &mut WindowTree<(), State>,
    window: Window<()>,
    event: Event,
    preview: bool,
    state: &mut State
) -> bool {
    match event {
        Event::GotFocus => {
            let old_focused = tree.focused();
            if old_focused != tree.root() {
                window.move_z(tree, Some(old_focused));
            }
        },
        Event::Key(_, Key::Escape) => state.quit = true,
        Event::Key(_, Key::Char('1')) | Event::Key(_, Key::Alt('1')) => {
            let window_1 = state.window_1;
            window_1.focus(tree, state);
        },
        Event::Key(_, Key::Char('2')) | Event::Key(_, Key::Alt('2')) => {
            let window_2 = state.window_2;
            window_2.focus(tree, state);
        },
        Event::Key(_, Key::Char('3')) | Event::Key(_, Key::Alt('3')) => {
            let window_3 = state.window_3;
            window_3.focus(tree, state);
        },
        _ => { },
    }
    true
}
 
fn main() {
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let mut windows = WindowTree::new(screen, render, measure, arrange, update, ()).unwrap();
    let root = windows.root();
    let window_1 = Window::new(&mut windows, (), root, None).unwrap();
    let window_2 = Window::new(&mut windows, (), root, None).unwrap();
    let window_3 = Window::new(&mut windows, (), root, Some(window_2)).unwrap();
    let mut state = State { window_1, window_2, window_3, quit: false };
    window_1.focus(&mut windows, &mut state);
    while !state.quit { 
        WindowTree::update(&mut windows, true, &mut state).unwrap();
    }
}
