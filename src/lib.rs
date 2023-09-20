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

use alloc::boxed::Box;
//use alloc::string::String;
use alloc::vec::Vec;
use components_arena::{Arena, Id, Component};
//use core::fmt::Debug;
//use core::ops::Range;
use macro_attr_2018::macro_attr;
//use phantom_type::PhantomType;
use tuifw_screen_base::{Bg, /*Event,*/ Fg, /*Key,*/ Point, Range1d /*, Rect*/};
use tuifw_window::{RenderPort /*, Window, WindowTree*/};
//use unicode_width::UnicodeWidthChar;

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

macro_attr! {
    #[derive(Component!)]
    pub struct BoxedView(pub Box<dyn TView>);
}

pub struct ViewTag(pub Vec<Id<BoxedView>>);

#[portrait::make]
pub trait TView {
    fn view(&self) -> &View;

    fn view_mut(&mut self) -> &mut View;

    fn init(&mut self);
}

pub struct View {
}

impl TView for View {
    fn view(&self) -> &View { self }

    fn view_mut(&mut self) -> &mut View { self }

    fn init(&mut self) {
    }
}

impl View {
    pub fn new() -> Self {
        View { }
    }
}

#[portrait::make]
pub trait TGroup: TView {
    fn group(&self) -> &Group;

    fn group_mut(&mut self) -> &mut Group;

    fn add_child(&mut self, view: Box<dyn TView>) -> Id<BoxedView> {
        self.group_mut().children.insert(move |id| (BoxedView(view), id))
    }

    fn remove_child(&mut self, id: Id<BoxedView>) -> Box<dyn TView> {
        self.group_mut().children.remove(id).0
    }
}

pub struct Group {
    pub view_: View,
    children: Arena<BoxedView>,
}

#[portrait::fill(portrait::delegate(View; self.view_))]
impl TView for Group { }

impl TGroup for Group {
    fn group(&self) -> &Group { self }

    fn group_mut(&mut self) -> &mut Group { self }
}

impl Group {
    pub fn new() -> Self {
        Group { view_: View::new(), children: Arena::new() }
    }
}
