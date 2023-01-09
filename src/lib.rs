#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]

#![no_std]

use tuifw_screen_base::{Bg, Fg, Point, Range1d};
use tuifw_window::RenderPort;

pub trait RenderPortExt {
    fn h_line(&mut self, start: Point, len: u16, double: bool, fg: Fg, bg: Bg);
    fn v_line(&mut self, start: Point, len: u16, double: bool, fg: Fg, bg: Bg);
}

impl RenderPortExt for RenderPort {
    fn h_line(&mut self, start: Point, len: u16, double: bool, fg: Fg, bg: Bg) {
        let s = if double { "═" } else { "─" };
        for x in Range1d::new(start.x, start.x.wrapping_add(len as i16)) {
            self.out(Point { x, y: start.y }, fg, bg, s);
        }
    }

    fn v_line(&mut self, start: Point, len: u16, double: bool, fg: Fg, bg: Bg) {
        let s = if double { "║" } else { "│" };
        for y in Range1d::new(start.y, start.y.wrapping_add(len as i16)) {
            self.out(Point { x: start.x, y }, fg, bg, s);
        }
    }
}
