#![feature(effects)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]

#![no_std]

extern crate alloc;

//use alloc::boxed::Box;
use alloc::string::String;
//use alloc::vec::Vec;
//use components_arena::{Arena, Id, Component};
//use core::fmt::Debug;
use core::ops::Range;
//use macro_attr_2018::macro_attr;
//use phantom_type::PhantomType;
use tuifw_screen_base::{Bg, Event, Fg, Key, Point, Range1d, Rect};
use tuifw_window::{RenderPort, Window, WindowTree};
use unicode_width::UnicodeWidthChar;

pub trait RenderPortExt {
    fn fill_bg(&mut self, bg: Bg);
    fn h_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg);
    fn v_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg);
    fn tl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn tr_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn bl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn br_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
}

impl RenderPortExt for RenderPort {
    fn fill_bg(&mut self, bg: Bg) {
        self.fill(|rp, p| rp.out(p, Fg::LightGray, bg, " "));
    }

    fn h_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg) {
        let s = if double { "═" } else { "─" };
        for x in Range1d::new(start.x, start.x.wrapping_add(len)) {
            self.out(Point { x, y: start.y }, fg, bg, s);
        }
    }

    fn v_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg) {
        let s = if double { "║" } else { "│" };
        for y in Range1d::new(start.y, start.y.wrapping_add(len)) {
            self.out(Point { x: start.x, y }, fg, bg, s);
        }
    }

    fn tl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╔" } else { "┌" });
    }

    fn tr_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╗" } else { "┐" });
    }

    fn bl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╚" } else { "└" });
    }

    fn br_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╝" } else { "┘" });
    }
}

pub struct LineEdit {
    fg: Fg,
    bg: Bg,
    line: String,
    is_active: bool,
    window: Window,
    view: Range<usize>,
    cursor: usize,
}

impl LineEdit {
    pub fn new<State: ?Sized>(
        tree: &mut WindowTree<State>,
        parent: Option<Window>,
        prev: Option<Window>
    ) -> Result<Self, tuifw_screen_base::Error> {
        let window = Window::new(tree, parent, prev)?;
        Ok(LineEdit {
            fg: Fg::White,
            bg: Bg::Blue,
            line: String::new(),
            is_active: false,
            window,
            cursor: 0,
            view: 0 .. 0
        })
    }

    pub fn fg(&self) -> Fg { self.fg }

    pub fn set_fg<State: ?Sized>(&mut self, tree: &mut WindowTree<State>, fg: Fg) {
        self.fg = fg;
        self.window.invalidate(tree);
    }

    pub fn bg(&self) -> Bg { self.bg }

    pub fn set_bg<State: ?Sized>(&mut self, tree: &mut WindowTree<State>, bg: Bg) {
        self.bg = bg;
        self.window.invalidate(tree);
    }

    pub fn window(&self) -> Window { self.window }

    pub fn move_xy<State: ?Sized>(&mut self, tree: &mut WindowTree<State>, bounds: Rect) {
        self.window.move_xy(tree, bounds);
        self.update_view(tree);
    }

    fn update_view<State: ?Sized>(&mut self, tree: &WindowTree<State>) {
        let bounds = self.window.bounds(tree);
        let s = &self.line[self.view.start ..];
        self.view.end = self.view.start;
        let mut width = 0;
        for c in s.chars().filter(|&x| x != '\0' && x.width().is_some()) {
            let c_width = c.width().unwrap();
            if usize::from(bounds.w() as u16) - width < c_width { break; }
            width += c_width;
            self.view.end += c.len_utf8();
        }
        if !self.view.is_empty() && !self.view.contains(&self.cursor) {
            self.cursor = self.view.end - 1;
        }
    }

    pub fn line(&self) -> &String { &self.line }

    pub fn line_mut<State: ?Sized, T>(&mut self, tree: &mut WindowTree<State>, f: impl FnOnce(&mut String) -> T) -> T {
        let res = f(&mut self.line);
        self.window.invalidate(tree);
        self.update_view(tree);
        res
    }

    pub fn is_active(&self) -> bool { self.is_active }

    pub fn set_is_active(&mut self, is_active: bool) {
        self.is_active = is_active;
    }

    pub fn render(
        &self,
        window: Window,
        port: &mut RenderPort,
    ) {
        if self.window != window { return; }
        port.out(Point { x: 0, y: 0 }, self.fg, self.bg, &self.line[self.view.clone()]);
    }

    pub fn update<State: ?Sized>(
        &mut self,
        tree: &mut WindowTree<State>,
        event: Event,
    ) {
        if self.is_active { return; }
        match event {
            Event::Key(n, Key::Char(c)) => {
                for _ in 0 .. n.get() {
                    self.line.insert(self.cursor, c);
                }
                self.window.invalidate(tree);
            },
            _ => { }
        }
    }
}
