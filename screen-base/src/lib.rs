#![feature(iter_advance_by)]
#![feature(stmt_expr_attributes)]
#![feature(trusted_len)]

#![deny(warnings)]
#![allow(unstable_name_collisions)] // because I can
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::manual_map)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]

#![no_std]

#[macro_use]
mod bitflags_ext;

use core::cmp::{min, max};
use core::iter::{DoubleEndedIterator, FusedIterator, Iterator, TrustedLen};
use core::num::{NonZeroU16, NonZeroI16};
use core::ops::{Add, AddAssign, Sub, SubAssign, Neg, Range, Index, IndexMut};
use core::option::{Option};
use either::{Either, Left, Right};
use errno_no_std::Errno;
use enum_derive_2018::{EnumDisplay, EnumFromStr, IterVariants};
use macro_attr_2018::macro_attr;
use num_traits::Zero;
#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Range1d {
    pub start: i16,
    pub end: i16,
}

impl Range1d {
    pub fn new(start: i16, end: i16) -> Self {
        Range1d { start, end }
    }

    pub fn len(self) -> u16 { self.end.wrapping_sub(self.start) as u16 }

    pub fn inclusive(start: i16, end: i16) -> Option<Self> {
        let res = Range1d { start, end: end.wrapping_add(1) };
        if res.is_empty() { None } else { Some(res) }
    }

    pub fn contains(self, coord: i16) -> bool {
        (coord.wrapping_sub(self.start) as u16) < (self.end.wrapping_sub(self.start) as u16)
    }

    pub fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub fn intersect(self, other: Range1d) -> Range1d {
        let (long, short) = if self.len() <= other.len() {
            (other, self)
        } else {
            (self, other)
        };
        if long.contains(short.start) {
            if long.contains(short.end) {
                short
            } else {
                Range1d::new(short.start, long.end)
            }
        } else {
            if long.contains(short.end) {
                Range1d::new(long.start, short.end)
            } else {
                Range1d::new(self.start, self.start)
            }
        }
    }

    pub fn union(self, other: Range1d) -> Option<Range1d> {
        let (long, short) = if self.len() <= other.len() {
            (other, self)
        } else {
            (self, other)
        };
        if long.contains(short.start) {
            if long.contains(short.end) {
                if Range1d::new(long.start, short.end).len() >= Range1d::new(long.start, short.start).len() {
                    Some(long)
                } else {
                    None
                }
            } else {
                let res = Range1d::new(long.start, short.end);
                if res.is_empty() { None } else { Some(res) }
            }
        } else {
            if long.contains(short.end) {
                let res = Range1d::new(short.start, long.end);
                if res.is_empty() { None } else { Some(res) }
            } else {
                if other.is_empty() {
                    Some(self)
                } else if self.is_empty() {
                    Some(other)
                } else {
                    let u = Range1d::new(self.start, other.end);
                    let v = Range1d::new(other.start, self.end);
                    if u.is_empty() {
                        if v.is_empty() { None } else { Some(v) }
                    } else {
                        Some(if v.is_empty() {
                            u
                        } else {
                            if u.len() < v.len() { u } else { v }
                        })
                    }
                }
            }
        }
    }
}

impl Iterator for Range1d {
    type Item = i16;

    fn next(&mut self) -> Option<i16> {
        if !self.is_empty() {
            let item = self.start;
            self.start = self.start.wrapping_add(1);
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    fn count(self) -> usize { self.len() as usize }

    fn last(self) -> Option<i16> {
        if self.is_empty() { None } else { Some(self.end) }
    }

    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        let len = self.len();
        if n > len {
            self.start = self.end;
            return Err(len);
        }
        self.start = self.start.wrapping_add(n as u16 as i16);
        Ok(())
    }
}

impl FusedIterator for Range1d { }

impl DoubleEndedIterator for Range1d {
    fn next_back(&mut self) -> Option<i16> {
        if !self.is_empty() {
            let item = self.end;
            self.end = self.end.wrapping_sub(1);
            Some(item)
        } else {
            None
        }
    }

    fn advance_back_by(&mut self, n: usize) -> Result<(), usize> {
        let len = self.len();
        if n > len {
            self.end = self.start;
            return Err(len);
        }
        self.end = self.end.wrapping_sub(n as u16 as i16);
        Ok(())
    }
}

impl ExactSizeIterator for Range1d {
    fn len(&self) -> usize {
        Range1d::len(*self) as usize
    }
}

unsafe impl TrustedLen for Range1d { }

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!)]
    pub enum Side {
        Left,
        Top,
        Right,
        Bottom
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!)]
    pub enum Orient {
        Hor,
        Vert
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!, IterVariants!(ColorVariants))]
    pub enum Color {
        Black,
        Red,
        Green,
        Yellow,
        Blue,
        Magenta,
        Cyan,
        White
    }
}

bitflags_ext! {
    pub struct Attr: u8 {
        REVERSE = 1 << 0,
        INTENSITY = 1 << 1,
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
pub enum Ctrl {
    At, A, B, C, D, E, F, G, J, K, L, N,
    O, P, Q, R, S, T, U, V, W, X, Y, Z,
    Backslash, Bracket, Caret, Underscore
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
#[non_exhaustive]
pub enum Key {
    Char(char),
    Alt(char),
    Ctrl(Ctrl),
    Enter,
    Escape,
    Down,
    Up,
    Left,
    Right,
    Home,
    End,
    Backspace,
    Delete,
    Insert,
    PageDown,
    PageUp,
    Tab,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
#[non_exhaustive]
pub enum Event {
    Resize,
    Key(NonZeroU16, Key),
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub fn offset(self, d: Vector) -> Point {
        Point { x: self.x.wrapping_add(d.x), y: self.y.wrapping_add(d.y) }
    }

    pub fn offset_from(self, other: Point) -> Vector {
        Vector { x: self.x.wrapping_sub(other.x), y: self.y.wrapping_sub(other.y) }
    }

    pub fn relative_to(self, base: Point) -> Point {
        Point { x: self.x.wrapping_sub(base.x), y: self.y.wrapping_sub(base.y) }
    }

    pub fn absolute_with(self, base: Point) -> Point {
        Point { x: self.x.wrapping_add(base.x), y: self.y.wrapping_add(base.y) }
    }
}

#[cfg(test)]
impl Arbitrary for Point {
    fn arbitrary(g: &mut Gen) -> Self {
        let a = <(_, _)>::arbitrary(g);
        Point { x: a.0, y: a.1 }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct Vector {
    pub x: i16,
    pub y: i16,
}

impl Vector {
    pub const fn null() -> Vector { Vector { x: 0, y: 0 } }

    pub fn is_null(self) -> bool { self.x == 0 && self.y == 0 }

    pub fn rect_area(self) -> u32 { (self.x as u16 as u32) * (self.y as u16 as u32) }

    pub fn max(self, other: Vector) -> Vector {
        Vector {
            x: max(self.x as u16, other.x as u16) as i16,
            y: max(self.y as u16, other.y as u16) as i16,
        }
    }

    pub fn min(self, other: Vector) -> Vector {
        Vector {
            x: min(self.x as u16, other.x as u16) as i16,
            y: min(self.y as u16, other.y as u16) as i16,
        }
    }
}

impl Default for Vector {
    fn default() -> Self { Vector::null() }
}

impl Zero for Vector {
    fn zero() -> Self { Vector::null() }

    fn is_zero(&self) -> bool { self.is_null() }

    fn set_zero(&mut self) { *self = Vector::null() }
}

impl Add for Vector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Vector { x: self.x.wrapping_add(other.x), y: self.y.wrapping_add(other.y) }
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Vector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Vector { x: self.x.wrapping_sub(other.x), y: self.y.wrapping_sub(other.y) }
    }
}

impl SubAssign for Vector {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Neg for Vector {
    type Output = Self;

    fn neg(self) -> Self {
        Vector::null() - self
    }
}

#[cfg(test)]
impl Arbitrary for Vector {
    fn arbitrary(g: &mut Gen) -> Self {
        let a = <(_, _)>::arbitrary(g);
        Vector { x: a.0, y: a.1 }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct VBand {
    pub l: i16,
    pub w: NonZeroI16,
}

impl VBand {
    pub fn from_l_r(l: i16, r: i16) -> Option<VBand> {
        NonZeroI16::new(r.wrapping_sub(l)).map(|w| VBand { l, w })
    }

    pub fn from_h_range(h_range: Range1d) -> Option<VBand> {
        VBand::from_l_r(h_range.start, h_range.end)
    }

    pub fn r(self) -> i16 { self.l.wrapping_add(self.w.get()) }

    pub fn h_range(self) -> Range1d { Range1d::new(self.l, self.r()) }

    pub fn offset(self, d: Vector) -> VBand {
        VBand { l: self.l.wrapping_add(d.x), w: self.w }
    }

    pub fn relative_to(self, base: Point) -> VBand {
        VBand { l: self.l.wrapping_sub(base.x), w: self.w }
    }

    pub fn absolute_with(self, base: Point) -> VBand {
        VBand { l: self.l.wrapping_add(base.x), w: self.w }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct HBand {
    pub t: i16,
    pub h: NonZeroI16,
}

impl HBand {
    pub fn from_t_b(t: i16, b: i16) -> Option<HBand> {
        NonZeroI16::new(b.wrapping_sub(t)).map(|h| HBand { t, h })
    }

    pub fn from_v_range(v_range: Range1d) -> Option<HBand> {
        HBand::from_t_b(v_range.start, v_range.end)
    }

    pub fn b(self) -> i16 { self.t.wrapping_add(self.h.get()) }

    pub fn v_range(self) -> Range1d { Range1d::new(self.t, self.b()) }

    pub fn offset(self, d: Vector) -> HBand {
        HBand { t: self.t.wrapping_add(d.y), h: self.h }
    }

    pub fn relative_to(self, base: Point) -> HBand {
        HBand { t: self.t.wrapping_sub(base.y), h: self.h }
    }

    pub fn absolute_with(self, base: Point) -> HBand {
        HBand { t: self.t.wrapping_add(base.y), h: self.h }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Default)]
pub struct Thickness {
    l: i32,
    r: i32,
    t: i32,
    b: i32,
}

impl Thickness {
    pub const fn new(l: i32, t: i32, r: i32, b: i32) -> Self {
        assert!(l >= -(u16::MAX as u32 as i32) && l <= u16::MAX as u32 as i32);
        assert!(t >= -(u16::MAX as u32 as i32) && t <= u16::MAX as u32 as i32);
        assert!(r >= -(u16::MAX as u32 as i32) && r <= u16::MAX as u32 as i32);
        assert!(b >= -(u16::MAX as u32 as i32) && b <= u16::MAX as u32 as i32);
        Thickness { l, t, r, b }
    }

    /// # Safety
    ///
    /// All passed parameters should be in the `-(2¹⁶ - 1) ..= (2¹⁶ - 1)` range.
    pub const unsafe fn new_unchecked(l: i32, t: i32, r: i32, b: i32) -> Self {
        Thickness { l, t, r, b }
    }

    pub const fn all(a: i32) -> Thickness {
        assert!(a >= -(u16::MAX as u32 as i32) && a <= u16::MAX as u32 as i32);
        Thickness { l: a, t: a, r: a, b: a }
    }

    pub fn l(self) -> i32 { self.l }

    pub fn t(self) -> i32 { self.t }

    pub fn r(self) -> i32 { self.r }

    pub fn b(self) -> i32 { self.b }

    pub fn align(inner: Vector, outer: Vector, h_align: HAlign, v_align: VAlign) -> Thickness {
        let h_neg = inner.x as u16 > outer.x as u16;
        let (outer_x, inner_x) = if h_neg { (inner.x, outer.x) } else { (outer.x, inner.x) };
        let w = (outer_x as u16) - (inner_x as u16);
        let (l, r) = match h_align {
            HAlign::Left => (0, w),
            HAlign::Right => (w, 0),
            HAlign::Center => {
                let l = w / 2;
                let r = w - l;
                (l, r)
            }
        };
        let v_neg = inner.y as u16 > outer.y as u16;
        let (outer_y, inner_y) = if v_neg { (inner.y, outer.y) } else { (outer.y, inner.y) };
        let h = (outer_y as u16) - (inner_y as u16);
        let (t, b) = match v_align {
            VAlign::Top => (0, h),
            VAlign::Bottom => (h, 0),
            VAlign::Center => {
                let t = h / 2;
                let b = h - t;
                (t, b)
            }
        };
        let l = l as u32 as i32;
        let t = t as u32 as i32;
        let r = r as u32 as i32;
        let b = b as u32 as i32;
        Thickness {
            l: if h_neg { -l } else { l },
            t: if v_neg { -t } else { t },
            r: if h_neg { -r } else { r },
            b: if v_neg { -b } else { b }
        }
    }

    fn shrink_near(thickness: u16, rect: (i16, i16)) -> (i16, i16) {
        let thickness = min(thickness, rect.1 as u16);
        (rect.0.wrapping_add(thickness as i16), rect.1.wrapping_sub(thickness as i16))
    }

    fn shrink_far(thickness: u16, rect: (i16, i16)) -> (i16, i16) {
        let thickness = min(thickness, rect.1 as u16);
        (rect.0, rect.1.wrapping_sub(thickness as i16))
    }

    fn expand_near(thickness: u16, rect: (i16, i16)) -> (i16, i16) {
        let thickness = min(thickness, u16::MAX - (rect.1 as u16));
        (rect.0.wrapping_sub(thickness as i16), rect.1.wrapping_add(thickness as i16))
    }

    fn expand_far(thickness: u16, rect: (i16, i16)) -> (i16, i16) {
        let thickness = min(thickness, u16::MAX - (rect.1 as u16));
        (rect.0, rect.1.wrapping_add(thickness as i16))
    }

    pub fn shrink_rect(self, rect: Rect) -> Rect {
        let (l, w) = if self.l < 0 {
            Self::expand_near((-self.l) as u32 as u16, (rect.l(), rect.w()))
        } else {
            Self::shrink_near(self.l as u32 as u16, (rect.l(), rect.w()))
        };
        let (t, h) = if self.t < 0 {
            Self::expand_near((-self.t) as u32 as u16, (rect.t(), rect.h()))
        } else {
            Self::shrink_near(self.t as u32 as u16, (rect.t(), rect.h()))
        };
        let (l, w) = if self.r < 0 {
            Self::expand_far((-self.r) as u32 as u16, (l, w))
        } else {
            Self::shrink_far(self.r as u32 as u16, (l, w))
        };
        let (t, h) = if self.b < 0 {
            Self::expand_far((-self.b) as u32 as u16, (t, h))
        } else {
            Self::shrink_far(self.b as u32 as u16, (t, h))
        };
        Rect { tl: Point { x: l, y: t }, size: Vector { x: w, y: h } }
    }

    pub fn expand_rect(self, rect: Rect) -> Rect {
        (-self).shrink_rect(rect)
    }

    pub fn shrink_rect_size(self, rect_size: Vector) -> Vector {
        self.shrink_rect(Rect { tl: Point { x: 0, y: 0 }, size: rect_size }).size
    }

    pub fn expand_rect_size(self, rect_size: Vector) -> Vector {
        self.expand_rect(Rect { tl: Point { x: 0, y: 0 }, size: rect_size }).size
    }

    pub fn shrink_band_h(self, band_h: i16) -> i16 {
        let (_, h) = if self.t < 0 {
            Self::expand_near((-self.t) as u32 as u16, (0, band_h))
        } else {
            Self::shrink_near(self.t as u32 as u16, (0, band_h))
        };
        h
    }

    pub fn expand_band_h(self, band_h: i16) -> i16 {
        (-self).shrink_band_h(band_h)
    }

    pub fn shrink_band_w(self, band_w: i16) -> i16 {
        let (_, w) = if self.l < 0 {
            Self::expand_near((-self.t) as u32 as u16, (0, band_w))
        } else {
            Self::shrink_near(self.t as u32 as u16, (0, band_w))
        };
        w
    }

    pub fn expand_band_w(self, band_w: i16) -> i16 {
        (-self).shrink_band_w(band_w)
    }

    fn add_side(this: i32, other: i32) -> i32 {
        if this < 0 {
            if other < 0 {
                -(((-this) as u32 as u16).saturating_add((-other) as u32 as u16) as u32 as i32)
            } else {
                other + this
            }
        } else {
            if other > 0 {
                (this as u32 as u16).saturating_add(other as u32 as u16) as u32 as i32
            } else {
                other + this
            }
        }
    }
}

impl Add for Thickness {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let l = Self::add_side(self.l, other.l);
        let t = Self::add_side(self.t, other.t);
        let r = Self::add_side(self.r, other.r);
        let b = Self::add_side(self.b, other.b);
        Thickness { l, t, r, b }
    }
}

impl AddAssign for Thickness {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Thickness {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self + (-other)
    }
}

impl SubAssign for Thickness {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl Neg for Thickness {
    type Output = Self;

    fn neg(self) -> Self {
        Thickness { l: -self.l, t: -self.t, r: -self.r, b: -self.b }
    }
}

impl Index<Side> for Thickness {
    type Output = i32;

    fn index(&self, index: Side) -> &i32 {
        match index {
            Side::Left => &self.l,
            Side::Top => &self.t,
            Side::Right => &self.r,
            Side::Bottom => &self.b
        }
    }
}

impl IndexMut<Side> for Thickness {
    fn index_mut(&mut self, index: Side) -> &mut i32 {
        match index {
            Side::Left => &mut self.l,
            Side::Top => &mut self.t,
            Side::Right => &mut self.r,
            Side::Bottom => &mut self.b
        }
    }
}

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!)]
    pub enum HAlign { Left, Center, Right }
}

macro_attr! {
    #[derive(Eq, PartialEq, Debug, Hash, Clone, Copy, Ord, PartialOrd)]
    #[derive(EnumDisplay!, EnumFromStr!)]
    pub enum VAlign { Top, Center, Bottom }
}

pub struct RectPoints {
    rect: Rect,
    x: i16,
}

impl Iterator for RectPoints {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        if self.rect.is_empty() {
            return None;
        }
        let item = Point { x: self.x, y: self.rect.t() };
        self.x = self.x.wrapping_add(1);
        if self.x == self.rect.r() {
            self.x = self.rect.l();
            self.rect.tl = Point { x: self.x, y: self.rect.t().wrapping_add(1) };
            self.rect.size = Vector { x: self.rect.w(), y: self.rect.h().wrapping_sub(1) };
        }
        Some(item)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.rect.area() - (self.x.wrapping_sub(self.rect.l()) as u16 as u32);
        if len as usize as u32 == len {
            (len as usize, Some(len as usize))
        } else {
            (usize::MAX, None)
        }
    }

    fn count(self) -> usize { self.size_hint().1.unwrap() }

    fn last(self) -> Option<Point> {
        if self.rect.is_empty() { None } else { Some(self.rect.br_inner()) }
    }

    fn advance_by(&mut self, n: usize) -> Result<(), usize> {
        if let Some(size) = self.size_hint().1 {
            if n > size {
                self.x = self.rect.l();
                self.rect.tl = self.rect.bl();
                return Err(size);
            }
        }
        let n = n as u32;
        let current_line_last = self.rect.r().wrapping_sub(self.x) as u16 as u32;
        if n < current_line_last {
            self.x = self.x.wrapping_add(n as u16 as i16);
            return Ok(());
        }
        let n = n - current_line_last;
        let skip_lines = 1i16.wrapping_add((n / self.rect.w() as u32) as u16 as i16);
        self.rect.tl = Point { x: self.rect.l(), y: self.rect.t().wrapping_add(skip_lines) };
        self.rect.size = Vector { x: self.rect.w(), y: self.rect.h().wrapping_sub(skip_lines) };
        self.x = (n % self.rect.w() as u32) as u16 as i16;
        Ok(())
    }
}

impl FusedIterator for RectPoints { }

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct Rect {
    pub tl: Point,
    pub size: Vector,
}

impl Rect {
    pub fn from_tl_br(tl: Point, br: Point) -> Rect {
        Rect { tl, size: br.offset_from(tl) }
    }
    
    pub fn from_h_v_ranges(h_range: Range1d, v_range: Range1d) -> Rect {
        Rect::from_tl_br(
            Point { x: h_range.start, y: v_range.start },
            Point { x: h_range.end, y: v_range.end }
        )
    }

    pub fn is_empty(self) -> bool { self.w() == 0 || self.h() == 0 }

    pub fn w(self) -> i16 { self.size.x }

    pub fn h(self) -> i16 { self.size.y }

    pub fn l(self) -> i16 { self.tl.x }

    pub fn t(self) -> i16 { self.tl.y }

    pub fn r(self) -> i16 { self.tl.x.wrapping_add(self.size.x) }

    pub fn b(self) -> i16 { self.tl.y.wrapping_add(self.size.y) }

    pub fn tr(self) -> Point { Point { x: self.r(), y: self.t() } }

    pub fn bl(self) -> Point { Point { x: self.l(), y: self.b() } }

    pub fn br(self) -> Point { Point { x: self.r(), y: self.b() } }

    pub fn r_inner(self) -> i16 {
        self.l().wrapping_add((self.size.x as u16).saturating_sub(1) as i16)
    }

    pub fn b_inner(self) -> i16 {
        self.t().wrapping_add((self.size.y as u16).saturating_sub(1) as i16)
    }

    pub fn tr_inner(self) -> Point { Point { x: self.r_inner(), y: self.t() } }

    pub fn bl_inner(self) -> Point { Point { x: self.l(), y: self.b_inner() } }

    pub fn br_inner(self) -> Point { Point { x: self.r_inner(), y: self.b_inner() } }

    pub fn area(self) -> u32 { self.size.rect_area() }

    pub fn points(self) -> RectPoints { RectPoints { rect: self, x: self.l() } }

    pub fn h_range(self) -> Range1d { Range1d { start: self.l(), end: self.r() } }

    pub fn v_range(self) -> Range1d { Range1d { start: self.t(), end: self.b() } }

    pub fn contains(self, p: Point) -> bool {
        self.h_range().contains(p.x) && self.v_range().contains(p.y)
    }

    pub fn intersect(self, other: Rect) -> Rect {
        let h = self.h_range().intersect(other.h_range());
        let v = self.v_range().intersect(other.v_range());
        Rect::from_h_v_ranges(h, v)
    }

    pub fn intersect_h_band(self, band: HBand) -> Rect {
        let v = self.v_range().intersect(band.v_range());
        Rect::from_h_v_ranges(self.h_range(), v)
    }

    pub fn intersect_v_band(self, band: VBand) -> Rect {
        let h = self.h_range().intersect(band.h_range());
        Rect::from_h_v_ranges(h, self.v_range())
    }

    pub fn union(self, other: Rect) -> Option<Either<Either<HBand, VBand>, Rect>> {
        if other.is_empty() { return Some(Right(self)); }
        if self.is_empty() { return Some(Right(other)); }
        let hr = self.h_range().union(other.h_range());
        let vr = self.v_range().union(other.v_range());
        if let Some(hr) = hr {
            if let Some(vr) = vr {
                Some(Right(Rect::from_h_v_ranges(hr, vr)))
            } else {
                Some(Left(Right(VBand::from_h_range(hr).unwrap())))
            }
        } else {
            if let Some(vr) = vr {
                Some(Left(Left(HBand::from_v_range(vr).unwrap())))
            } else {
                None
            }
        }
    }

    pub fn union_intersect(self, union_with: Rect, intersect_with: Rect) -> Rect {
        match self.union(union_with) {
            None => intersect_with,
            Some(Right(rect)) => rect.intersect(intersect_with),
            Some(Left(Right(v_band))) => Rect {
                tl: Point { x: v_band.l, y: intersect_with.t() },
                size: Vector { x: v_band.w.get(), y: intersect_with.h() }
            },
            Some(Left(Left(h_band))) => Rect {
                tl: Point { y: h_band.t, x: intersect_with.l() },
                size: Vector { y: h_band.h.get(), x: intersect_with.w() }
            },
        }
    }
 
    pub fn offset(self, d: Vector) -> Rect {
        Rect { tl: self.tl.offset(d), size: self.size }
    }

    pub fn relative_to(self, base: Point) -> Rect {
        Rect { tl: self.tl.relative_to(base), size: self.size }
    }

    pub fn absolute_with(self, base: Point) -> Rect {
        Rect { tl: self.tl.absolute_with(base), size: self.size }
    }

    pub fn t_line(self) -> Rect {
        let height = min(1, self.size.y as u16) as i16;
        Rect { tl: self.tl, size: Vector { x: self.size.x, y: height } }
    }

    pub fn b_line(self) -> Rect {
        let height = min(1, self.size.y as u16) as i16;
        Rect {
            tl: Point { x: self.l(), y: self.b().wrapping_sub(height) },
            size: Vector { x: self.size.x, y: height }
        }
    }

    pub fn l_line(self) -> Rect {
        let width = min(1, self.size.x as u16) as i16;
        Rect { tl: self.tl, size: Vector { x: width, y: self.size.y } }
    }

    pub fn r_line(self) -> Rect {
        let width = min(1, self.size.x as u16) as i16;
        Rect {
            tl: Point { x: self.r().wrapping_sub(width), y: self.t() },
            size: Vector { x: width, y: self.size.y }
        }
    }
}

#[cfg(test)]
impl Arbitrary for Rect {
    fn arbitrary(g: &mut Gen) -> Self {
        let a = <(Point, Point)>::arbitrary(g);
        Rect::from_tl_br(a.0, a.1)
    }
}

pub trait Screen {
    fn size(&self) -> Vector;

    fn out(
        &mut self,
        p: Point,
        fg: Color,
        bg: Option<Color>,
        attr: Attr,
        text: &str,
        hard: Range<i16>,
        soft: Range<i16>,
    ) -> Range<i16>;

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Errno>;
}

#[cfg(test)]
mod tests {
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;
    use crate::*;

    #[test]
    fn test_range_iterator() {
        let r = Range1d::new(0, 29).step_by(2);
        assert_eq!(r.count(), 15);
    }

    #[quickcheck]
    fn rect_area(r: Rect) -> bool {
        r.area() == r.size.rect_area()
    }

    #[quickcheck]
    #[allow(clippy::bool_comparison)]
    fn rect_is_empty_area(r: Rect) -> bool {
        !r.is_empty() == (r.area() > 0)
    }

    #[quickcheck]
    fn null_size_rect_is_empty(tl: Point) -> bool {
        Rect { tl, size: Vector::null() }.is_empty()
    }

    #[quickcheck]
    fn rect_empty_intersect(tl1: Point, r2: Rect) -> bool {
        let r1 = Rect { tl: tl1, size: Vector::null() };
        r1.intersect(r2) == r1
    }

    #[quickcheck]
    fn rect_intersect_empty(r1: Rect, tl2: Point) -> bool {
        let r2 = Rect { tl: tl2, size: Vector::null() };
        r1.is_empty() || r1.contains(r1.intersect(r2).tl)
    }

    #[quickcheck]
    fn rect_intersect_contains(r1: Rect, r2: Rect, p: Point) -> bool {
        r1.intersect(r2).contains(p) || !(r1.contains(p) && r2.contains(p))
    }

    #[quickcheck]
    fn rect_union_contains(r1: Rect, r2: Rect, p: Point) -> bool {
        r1.union(r2).map_or(true, |u| u.either(|_| true, |u| u.contains(p))) || !(r1.contains(p) || r2.contains(p))
    }

    #[quickcheck]
    fn rect_empty_h_union(tl1: Point, r2: Rect, w: i16) -> bool {
        let r1 = Rect { tl: tl1, size: Vector { x: w, y: 0} };
        r2.is_empty() || r1.union(r2).unwrap().right().unwrap() == r2
    }

    #[quickcheck]
    fn rect_union_empty_h(r1: Rect, tl2: Point, w: i16) -> bool {
        let r2 = Rect { tl: tl2, size: Vector { x: w, y: 0 } };
        r1.union(r2).unwrap().right().unwrap() == r1
    }

    #[quickcheck]
    fn rect_empty_w_union(tl1: Point, r2: Rect, h: i16) -> bool {
        let r1 = Rect { tl: tl1, size: Vector { x: 0, y: h } };
        r2.is_empty() || r1.union(r2).unwrap().right().unwrap() == r2
    }

    #[quickcheck]
    fn rect_union_empty_w(r1: Rect, tl2: Point, h: i16) -> bool {
        let r2 = Rect { tl: tl2, size: Vector { x: 0, y: h } };
        r1.union(r2).unwrap().right().unwrap() == r1
    }

    #[quickcheck]
    fn rect_contains_all_self_points(r: Rect) -> TestResult {
        if r.area() > 100000 { return TestResult::discard(); }
        TestResult::from_bool(r.points().all(|x| r.contains(x)))
    }
}
