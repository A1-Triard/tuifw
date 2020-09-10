#![deny(warnings)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]
#![feature(stmt_expr_attributes)]

#![no_std]
extern crate alloc;

#[macro_use]
mod bitflags_ext;

use alloc::boxed::Box;
use core::any::Any;
use core::cmp::{min, max};
use core::num::{NonZeroU16, NonZeroI16};
use core::ops::{Add, AddAssign, Sub, SubAssign, Neg, Range, Index, IndexMut};
use core::option::{Option};
use num_traits::Zero;
use either::{Either, Left, Right};
use macro_attr_2018::macro_attr;
use enum_derive_2018::{EnumDisplay, EnumFromStr, IterVariants};
#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

mod std {
    pub use core::*;
}

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
        Point { x: self.x.overflowing_add(d.x).0, y: self.y.overflowing_add(d.y).0 }
    }

    pub fn offset_from(self, other: Point) -> Vector {
        Vector { x: self.x.overflowing_sub(other.x).0, y: self.y.overflowing_sub(other.y).0 }
    }
}

#[cfg(test)]
impl Arbitrary for Point {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
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
    pub fn null() -> Vector { Vector { x: 0, y: 0 } }

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
        Vector { x: self.x.overflowing_add(other.x).0, y: self.y.overflowing_add(other.y).0 }
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
        Vector { x: self.x.overflowing_sub(other.x).0, y: self.y.overflowing_sub(other.y).0 }
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
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
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
    pub fn with_l_r(l: i16, r: i16) -> Option<VBand> {
        NonZeroI16::new(r.overflowing_sub(l).0).map(|w| VBand { l, w })
    }

    pub fn r(self) -> i16 { self.l.overflowing_add(self.w.get()).0 }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct HBand {
    pub t: i16,
    pub h: NonZeroI16,
}

impl HBand {
    pub fn with_t_b(t: i16, b: i16) -> Option<HBand> {
        NonZeroI16::new(b.overflowing_sub(t).0).map(|h| HBand { t, h })
    }

    pub fn b(self) -> i16 { self.t.overflowing_add(self.h.get()).0 }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct Thickness {
    pub l: i16,
    pub t: i16,
    pub r: i16,
    pub b: i16
}

impl Thickness {
    pub fn align(inner: Vector, outer: Vector, h_align: HAlign, v_align: VAlign) -> Thickness {
        let w = outer.x.overflowing_sub(inner.x).0;
        let (l, r) = match h_align {
            HAlign::Left => (0, w),
            HAlign::Right => (w, 0),
            HAlign::Center => {
                let l = w / 2;
                (l, w.overflowing_sub(l).0)
            }
        };
        let h = outer.y.overflowing_sub(inner.y).0;
        let (t, b) = match v_align {
            VAlign::Top => (0, h),
            VAlign::Bottom => (h, 0),
            VAlign::Center => {
                let t = h / 2;
                (t, h.overflowing_sub(t).0)
            }
        };
        Thickness { l, t, r, b }
    }

    pub fn shrink(self, rect: Rect) -> Rect {
        Rect::with_tl_br(
            rect.tl.offset(Vector { x: self.l, y: self.t }),
            rect.br().offset(-Vector { x: self.r, y: self.b })
        )
    }

    pub fn all(a: i16) -> Thickness {
        Thickness { l: a, t: a, r: a, b: a }
    }
}

impl Index<Side> for Thickness {
    type Output = i16;

    fn index(&self, index: Side) -> &i16 {
        match index {
            Side::Left => &self.l,
            Side::Top => &self.t,
            Side::Right => &self.r,
            Side::Bottom => &self.b
        }
    }
}

impl IndexMut<Side> for Thickness {
    fn index_mut(&mut self, index: Side) -> &mut i16 {
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

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct Rect {
    pub tl: Point,
    pub size: Vector,
}

impl Rect {
    pub fn with_tl_br(tl: Point, br: Point) -> Rect {
        Rect { tl, size: br.offset_from(tl) }
    }

    pub fn is_empty(self) -> bool { self.w() == 0 || self.h() == 0 }

    pub fn w(self) -> i16 { self.size.x }

    pub fn h(self) -> i16 { self.size.y }

    pub fn l(self) -> i16 { self.tl.x }

    pub fn t(self) -> i16 { self.tl.y }

    pub fn r(self) -> i16 { self.tl.x.overflowing_add(self.size.x).0 }

    pub fn b(self) -> i16 { self.tl.y.overflowing_add(self.size.y).0 }

    pub fn tr(self) -> Point { Point { x: self.r(), y: self.t() } }

    pub fn bl(self) -> Point { Point { x: self.l(), y: self.b() } }

    pub fn br(self) -> Point { Point { x: self.r(), y: self.b() } }

    pub fn area(self) -> u32 { self.size.rect_area() }

    fn contains_1d(r: (i16, i16), p: i16) -> bool {
        (p.overflowing_sub(r.0).0 as u16) < (r.1 as u16)
    }

    pub fn contains(self, p: Point) -> bool {
        Self::contains_1d((self.l(), self.w()), p.x) && Self::contains_1d((self.t(), self.h()), p.y)
    }

    fn intersect_1d(s: (i16, i16), o: (i16, i16)) -> (i16, i16) {
        let (a, b) = if (s.1 as u16) <= (o.1 as u16) { (o, s) } else { (s, o) };
        if Self::contains_1d(a, b.0) {
            if Self::contains_1d(a, b.0.overflowing_add(b.1).0) {
                b
            } else {
                (b.0, a.0.overflowing_add(a.1).0.overflowing_sub(b.0).0)
            }
        } else {
            if Self::contains_1d(a, b.0.overflowing_add(b.1).0) {
                (a.0, b.0.overflowing_add(b.1).0.overflowing_sub(a.0).0)
            } else {
                (b.0, 0)
            }
        }
    }

    pub fn intersect(self, other: Rect) -> Rect {
        let (l, w) = Self::intersect_1d((self.l(), self.w()), (other.l(), other.w()));
        let (t, h) = Self::intersect_1d((self.t(), self.h()), (other.t(), other.h()));
        Rect { tl: Point { x: l, y: t }, size: Vector { x: w, y: h } }
    }

    pub fn intersect_h_band(self, band: HBand) -> Rect {
        let (t, h) = Self::intersect_1d((self.t(), self.h()), (band.t, band.h.get()));
        Rect { tl: Point { x: self.l(), y: t }, size: Vector { x: self.w(), y: h } }
    }

    pub fn intersect_v_band(self, band: VBand) -> Rect {
        let (l, w) = Self::intersect_1d((self.l(), self.w()), (band.l, band.w.get()));
        Rect { tl: Point { x: l, y: self.t() }, size: Vector { x: w, y: self.h() } }
    }

    fn union_1d(s: (i16, NonZeroI16), o: (i16, NonZeroI16)) -> Option<(i16, NonZeroI16)> {
        let (a, b) = if (s.1.get() as u16) <= (o.1.get() as u16) { (o, s) } else { (s, o) };
        if Self::contains_1d((a.0, a.1.get()), b.0) {
            if Self::contains_1d((a.0, a.1.get()), b.0.overflowing_add(b.1.get()).0) {
                if (b.0.overflowing_add(b.1.get()).0.overflowing_sub(a.0).0 as u16) >= (b.0.overflowing_sub(a.0).0 as u16) {
                    Some(a)
                } else {
                    None
                }
            } else {
                let w = NonZeroI16::new(b.0.overflowing_add(b.1.get()).0.overflowing_sub(a.0).0);
                w.map(|w| (a.0, w))
            }
        } else {
            if Self::contains_1d((a.0, a.1.get()), b.0.overflowing_add(b.1.get()).0) {
                let w = NonZeroI16::new(a.0.overflowing_add(a.1.get()).0.overflowing_sub(b.0).0);
                w.map(|w| (b.0, w))
            } else {
                let u = NonZeroI16::new(o.0.overflowing_add(o.1.get()).0.overflowing_sub(s.0).0);
                let v = NonZeroI16::new(s.0.overflowing_add(s.1.get()).0.overflowing_sub(o.0).0);
                u.map_or_else(|| v.map(|v| (o.0, v)), |u| Some(v.map_or_else(
                    || (s.0, u),
                    |v| if (u.get() as u16) <= (v.get() as u16) { (s.0, u) } else { (o.0, v) }
                )))
            }
        }
    }

    pub fn union(self, other: Rect) -> Option<Either<Either<HBand, VBand>, Rect>> {
        if other.is_empty() { return Some(Right(self)); }
        if self.is_empty() { return Some(Right(other)); }
        let self_w = unsafe { NonZeroI16::new_unchecked(self.w()) };
        let self_h = unsafe { NonZeroI16::new_unchecked(self.h()) };
        let other_w = unsafe { NonZeroI16::new_unchecked(other.w()) };
        let other_h = unsafe { NonZeroI16::new_unchecked(other.h()) };
        let hr = Self::union_1d((self.l(), self_w), (other.l(), other_w));
        let vr = Self::union_1d((self.t(), self_h), (other.t(), other_h));
        if let Some((l, w)) = hr {
            if let Some((t, h)) = vr {
                Some(Right(Rect { tl: Point { x: l, y: t }, size: Vector { x: w.get(), y: h.get() } }))
            } else {
                Some(Left(Right(VBand { l, w })))
            }
        } else {
            if let Some((t, h)) = vr {
                Some(Left(Left(HBand { t, h })))
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
}

#[cfg(test)]
impl Arbitrary for Rect {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        let a = <(Point, Point)>::arbitrary(g);
        Rect::with_tl_br(a.0, a.1)
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
    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Box<dyn Any>>;
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;
    use crate::*;

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
        r1.is_empty() || r1.intersect(r2) == r2
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
}
