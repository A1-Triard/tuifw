#![feature(allocator_api)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![no_std]

extern crate alloc;

use alloc::alloc::Global;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::iter::repeat;
use core::cmp::{min, max};
use core::ops::Range;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;
use unicode_width::UnicodeWidthChar;

pub struct Screen<A: Allocator + Clone = Global> {
    buf: Vec<(char, Fg, Bg), A>,
    out: Vec<(char, Fg, Bg), A>,
    size: Vector,
    invalidated: Rect,
    cursor: Option<Point>,
    data: Vec<Range<i16>, A>,
}

impl Screen {
    pub fn new(size: Vector) -> Self {
        Self::new_in(size, Global)
    }
}

impl<A: Allocator + Clone> Screen<A> {
    pub fn new_in(size: Vector, alloc: A) -> Self {
        assert!(size.x >= 0 && size.y >= 0);
        let mut s = Screen {
            data: Vec::new_in(alloc.clone()),
            buf: Vec::new_in(alloc.clone()),
            out: Vec::new_in(alloc),
            size: Vector::null(),
            invalidated: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
            cursor: None,
        };
        s.resize(size);
        s
    }

    pub fn cursor(&self) -> Option<Point> { self.cursor }

    fn resize(&mut self, out_size: Vector) {
        self.data.clear();
        self.data.resize(usize::from(out_size.y as u16), 0 .. out_size. x);
        self.buf.resize(out_size.rect_area() as usize, (' ', Fg::LightGray, Bg::None));
        self.out.resize(out_size.rect_area() as usize, (' ', Fg::LightGray, Bg::None));
        self.size = out_size;
        self.invalidated = Rect { tl: Point { x: 0, y: 0 }, size: self.size };
    }
}

impl<A: Allocator + Clone> base_Screen for Screen<A> {
    fn size(&self) -> Vector { self.size }

    fn out(
        &mut self,
        p: Point,
        fg: Fg,
        bg: Bg,
        text: &str,
        hard: Range<i16>,
        soft: Range<i16>
    ) -> Range<i16> {
        assert!(p.y >= 0 && p.y < self.size().y);
        assert!(hard.start >= 0 && hard.end > hard.start && hard.end <= self.size().x);
        assert!(soft.start >= 0 && soft.end > soft.start && soft.end <= self.size().x);
        let text_end = if soft.end <= p.x { return 0 .. 0 } else { soft.end.saturating_sub(p.x) };
        let text_start = if soft.start <= p.x { 0 } else { soft.start.saturating_sub(p.x) };
        let size = self.size;
        let line = (p.y as u16 as usize) * (size.x as u16 as usize);
        let line = &mut self.buf[line .. line + size.x as u16 as usize];
        let mut before_hard_start = min(p.x, hard.start);
        let mut before_text_start = 0i16;
        let x0 = max(hard.start, p.x);
        let mut x = x0;
        let text = text.chars()
            .filter(|&c| c != '\0' && c.width().is_some())
            .flat_map(|c| repeat(c).take(c.width().unwrap()))
        ;
        for g in text.take(text_end as u16 as usize) {
            if x >= hard.end { break; }
            let visible_1 = if before_text_start < text_start {
                before_text_start += 1;
                false
            } else {
                true
            };
            let visible_2 = if before_hard_start < hard.start {
                before_hard_start += 1;
                false
            } else {
                true
            };
            if visible_1 && visible_2 {
                let col = &mut line[x as u16 as usize];
                *col = (g, fg, bg);
            }
            x += 1;
        }
        self.invalidated = self.invalidated
            .union(Rect::from_tl_br(Point { x: x0, y: p.y }, Point { x, y: p.y + 1 }))
            .unwrap().right().unwrap()
        ;
        x0 .. x
    }

    fn update(&mut self, cursor: Option<Point>, _wait: bool) -> Result<Option<Event>, Error> {
        for y in self.invalidated.t() .. self.invalidated.b() {
            let line = (y as u16 as usize) * (self.size.x as u16 as usize);
            let s = line + self.invalidated.l() as u16 as usize;
            let f = line + self.invalidated.r() as u16 as usize;
            self.out[s .. f].copy_from_slice(&self.buf[s .. f]);
        }
        self.invalidated.size = Vector::null();
        self.cursor = cursor.and_then(|cursor| {
            if (Rect { tl: Point { x: 0, y: 0 }, size: self.size() }).contains(cursor) {
                Some(cursor)
            } else {
                None
            }
        });
        Ok(None)
    }

    fn line_invalidated_range(&self, line: i16) -> &Range<i16> { &self.data[usize::from(line as u16)] }

    fn line_invalidated_range_mut(&mut self, line: i16) -> &mut Range<i16> { &mut self.data[usize::from(line as u16)] }
}
