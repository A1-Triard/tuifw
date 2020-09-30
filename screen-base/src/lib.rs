#![feature(const_fn)]
#![feature(const_panic)]
#![feature(stmt_expr_attributes)]

#![deny(warnings)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::too_many_arguments)]

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
        NonZeroI16::new(r.wrapping_sub(l)).map(|w| VBand { l, w })
    }

    pub fn r(self) -> i16 { self.l.wrapping_add(self.w.get()) }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct HBand {
    pub t: i16,
    pub h: NonZeroI16,
}

impl HBand {
    pub fn with_t_b(t: i16, b: i16) -> Option<HBand> {
        NonZeroI16::new(b.wrapping_sub(t)).map(|h| HBand { t, h })
    }

    pub fn b(self) -> i16 { self.t.wrapping_add(self.h.get()) }
}

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
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

impl Default for Thickness {
    fn default() -> Self {
        Thickness { l: 0, t: 0, r: 0, b: 0 }
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

#[derive(Eq, PartialEq, Debug, Hash, Clone, Copy)]
pub struct Rect {
    pub tl: Point,
    pub size: Vector,
}

impl Rect {
    pub fn from_tl_br(tl: Point, br: Point) -> Rect {
        Rect { tl, size: br.offset_from(tl) }
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

    pub fn area(self) -> u32 { self.size.rect_area() }

    fn contains_1d(r: (i16, i16), p: i16) -> bool {
        (p.wrapping_sub(r.0) as u16) < (r.1 as u16)
    }

    pub fn contains(self, p: Point) -> bool {
        Self::contains_1d((self.l(), self.w()), p.x) && Self::contains_1d((self.t(), self.h()), p.y)
    }

    fn intersect_1d(s: (i16, i16), o: (i16, i16)) -> (i16, i16) {
        let (a, b) = if (s.1 as u16) <= (o.1 as u16) { (o, s) } else { (s, o) };
        if Self::contains_1d(a, b.0) {
            if Self::contains_1d(a, b.0.wrapping_add(b.1)) {
                b
            } else {
                (b.0, a.0.wrapping_add(a.1).wrapping_sub(b.0))
            }
        } else {
            if Self::contains_1d(a, b.0.wrapping_add(b.1)) {
                (a.0, b.0.wrapping_add(b.1).wrapping_sub(a.0))
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
            if Self::contains_1d((a.0, a.1.get()), b.0.wrapping_add(b.1.get())) {
                if (b.0.wrapping_add(b.1.get()).wrapping_sub(a.0) as u16) >= (b.0.wrapping_sub(a.0) as u16) {
                    Some(a)
                } else {
                    None
                }
            } else {
                let w = NonZeroI16::new(b.0.wrapping_add(b.1.get()).wrapping_sub(a.0));
                w.map(|w| (a.0, w))
            }
        } else {
            if Self::contains_1d((a.0, a.1.get()), b.0.wrapping_add(b.1.get())) {
                let w = NonZeroI16::new(a.0.wrapping_add(a.1.get()).wrapping_sub(b.0));
                w.map(|w| (b.0, w))
            } else {
                let u = NonZeroI16::new(o.0.wrapping_add(o.1.get()).wrapping_sub(s.0));
                let v = NonZeroI16::new(s.0.wrapping_add(s.1.get()).wrapping_sub(o.0));
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
