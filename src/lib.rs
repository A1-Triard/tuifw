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
use core::str::FromStr;
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

pub enum EditValueRange {
    Any,
    Float(Range<f64>),
    Integer(Range<i64>),
}

pub struct LineEdit {
    allowed_values: EditValueRange,
    norm_fg: Fg,
    norm_bg: Bg,
    err_bg: Bg,
    err_fg: Fg,
    line: String,
    is_active: bool,
    window: Window,
    view: Range<usize>,
    cursor: usize,
}

impl LineEdit {
    pub fn new<State: ?Sized>(
        tree: &mut WindowTree<State>,
        allowed_values: EditValueRange,
        parent: Option<Window>,
        prev: Option<Window>
    ) -> Result<Self, tuifw_screen_base::Error> {
        let window = Window::new(tree, parent, prev)?;
        Ok(LineEdit {
            allowed_values,
            norm_fg: Fg::White,
            norm_bg: Bg::Blue,
            err_fg: Fg::White,
            err_bg: Bg::Red,
            line: String::new(),
            is_active: false,
            window,
            cursor: 0,
            view: 0 .. 0
        })
    }

    pub fn norm_fg(&self) -> Fg { self.norm_fg }

    pub fn err_fg(&self) -> Fg { self.err_fg }

    pub fn set_norm_fg<State: ?Sized>(&mut self, tree: &mut WindowTree<State>, fg: Fg) {
        self.norm_fg = fg;
        self.window.invalidate(tree);
    }

    pub fn set_err_fg<State: ?Sized>(&mut self, tree: &mut WindowTree<State>, fg: Fg) {
        self.err_fg = fg;
        self.window.invalidate(tree);
    }

    pub fn norm_bg(&self) -> Bg { self.norm_bg }

    pub fn err_bg(&self) -> Bg { self.err_bg }

    pub fn set_norm_bg<State: ?Sized>(&mut self, tree: &mut WindowTree<State>, bg: Bg) {
        self.norm_bg = bg;
        self.window.invalidate(tree);
    }

    pub fn set_err_bg<State: ?Sized>(&mut self, tree: &mut WindowTree<State>, bg: Bg) {
        self.err_bg = bg;
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

    fn show_err(&self) -> bool {
        match &self.allowed_values {
            EditValueRange::Any => false,
            EditValueRange::Integer(range) => if let Ok(value) = i64::from_str(&self.line) {
                !range.contains(&value)
            } else {
                true
            },
            EditValueRange::Float(range) => if let Ok(value) = f64::from_str(&self.line) {
                !range.contains(&value)
            } else {
                true
            },
        }
    }

    pub fn render(
        &self,
        window: Window,
        port: &mut RenderPort,
    ) {
        if self.window != window { return; }
        let (bg, fg) = if self.show_err() { (self.err_bg, self.err_fg) } else { (self.norm_bg, self.norm_fg) };
        port.fill_bg(bg);
        port.out(Point { x: 0, y: 0 }, fg, bg, &self.line[self.view.clone()]);
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
                self.update_view(tree);
                self.window.invalidate(tree);
            },
            _ => { }
        }
    }
}
