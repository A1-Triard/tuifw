#![feature(effects)]
#![feature(never_type)]

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
use alloc::vec::Vec;
use components_arena::{Arena, Component, Id, NewtypeComponentId};
use core::any::Any;
use core::cmp::{max, min};
use core::mem::replace;
use core::num::NonZeroU16;
use dyn_clone::{DynClone, clone_trait_object};
use educe::Educe;
use either::{Either, Left, Right};
use macro_attr_2018::macro_attr;
use tuifw_screen_base::{Bg, Error, Fg, Key, Point, Rect, Screen, Vector};
use tuifw_screen_base::Event as screen_Event;
use tuifw_screen_base::{HAlign, VAlign, Thickness, Range1d};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Event {
    Key(NonZeroU16, Key),
    GotFocus,
    LostFocus,
}

fn invalidate_rect(screen: &mut dyn Screen, rect: Rect) {
    let rect = rect.intersect(Rect { tl: Point { x: 0, y: 0 }, size: screen.size() });
    if rect.is_empty() { return; }
    let l = rect.l();
    let r = rect.r();
    for y in rect.t() .. rect.b() {
        let row = screen.line_invalidated_range_mut(y);
        row.start = min(row.start, l);
        row.end = max(row.end, r);
    }
}

fn rect_invalidated(screen: &dyn Screen, rect: Rect) -> bool {
    let rect = rect.intersect(Rect { tl: Point { x: 0, y: 0 }, size: screen.size() });
    if rect.is_empty() { return false; }
    let l = rect.l();
    let r = rect.r();
    for y in rect.t() .. rect.b() {
        let row = screen.line_invalidated_range(y);
        if row.end == row.start { continue; }
        if l < row.start {
            if r > row.end { return true; }
        } else if l < row.end {
            return true;
        }
    }
    false
}

pub struct RenderPort {
    screen: Box<dyn Screen>,
    offset: Vector,
    size: Vector,
    cursor: Option<Point>,
}

impl RenderPort {
    pub fn out(&mut self, p: Point, fg: Fg, bg: Bg, text: &str) {
        let screen_size = self.screen.size();
        if p.y as u16 >= self.size.y as u16 || self.size.x == 0 { return; }
        let p = p.offset(self.offset);
        if p.y < 0 || p.y >= self.screen.size().y { return; }
        let row = self.screen.line_invalidated_range(p.y).clone();
        if p.x >= row.end { return; }

        let window_start = Point { x: 0, y: 0 }.offset(self.offset).x;
        let window_end = Point { x: 0, y: 0 }.offset(self.size + self.offset).x;
        let chunks = if window_start <= window_end {
            if window_end <= 0 || window_start >= screen_size.x { return; }
            [max(0, window_start) .. min(screen_size.x, window_end), 0 .. 0]
        } else {
            if window_end > 0 && window_start < screen_size.x {
                [0 .. window_end, window_start .. screen_size.x]
            } else if window_end > 0 {
                [0 .. window_end, 0 .. 0]
            } else if window_start < screen_size.x {
                [window_start .. screen_size.x, 0 .. 0]
            } else {
                return
            }
        };

        for chunk in &chunks {
            if chunk.start >= chunk.end { continue; }
            let out = self.screen.out(p, fg, bg, text, chunk.clone(), row.clone());
            if out.start >= out.end { continue; }
            let row = self.screen.line_invalidated_range_mut(p.y);
            row.start = min(row.start, out.start);
            row.end = max(row.end, out.end);
            if let Some(cursor) = self.cursor {
                if cursor.y == p.y && cursor.x >= out.start && cursor.x < out.end {
                    self.cursor = None;
                }
            }
        }
    }

    pub fn cursor(&mut self, p: Point) {
        if self.cursor.is_some() { return; }
        let p = p.offset(self.offset);
        if p.y < 0 || p.y >= self.screen.size().y { return; }
        let row = &self.screen.line_invalidated_range(p.y);
        if p.x < row.start || p.x >= row.end { return; }
        self.cursor = Some(p);
    }

    pub fn fill(&mut self, mut f: impl FnMut(&mut Self, Point)) {
        for y in 0 .. self.screen.size().y {
            for x in self.screen.line_invalidated_range(y).clone() {
                f(self, Point { x, y }.offset(-self.offset));
            }
        }
    }

    pub fn fill_bg(&mut self, bg: Bg, fg: Option<Fg>) {
        self.fill(|rp, p| rp.out(p, fg.unwrap_or(Fg::LightGray), bg, if fg.is_some() { "░" } else { " " }));
    }

    pub fn h_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg) {
        let s = if double { "═" } else { "─" };
        for x in Range1d::new(start.x, start.x.wrapping_add(len)) {
            self.out(Point { x, y: start.y }, fg, bg, s);
        }
    }

    pub fn v_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg) {
        let s = if double { "║" } else { "│" };
        for y in Range1d::new(start.y, start.y.wrapping_add(len)) {
            self.out(Point { x: start.x, y }, fg, bg, s);
        }
    }

    pub fn tl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╔" } else { "┌" });
    }

    pub fn tr_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╗" } else { "┐" });
    }

    pub fn bl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╚" } else { "└" });
    }

    pub fn br_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╝" } else { "┘" });
    }
}

pub trait Widget<State: ?Sized>: DynClone {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        state: &mut State,
    );

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector;

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector;

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        preview: bool,
        state: &mut State,
    ) -> bool;
}

clone_trait_object!(<State: ?Sized> Widget<State>);

pub struct Palette(Vec<Either<u8, (Fg, Bg)>>);

impl Palette {
    pub fn new() -> Palette {
        Palette(Vec::new())
    }

    pub fn get(&self, i: u8) -> Either<u8, (Fg, Bg)> {
        self.0.get(usize::from(i)).cloned().unwrap_or(Left(i))
    }

    pub fn set(&mut self, i: u8, o: Either<u8, (Fg, Bg)>) {
        for k in self.0.len() ..= usize::from(i) {
            self.0.push(Left(u8::try_from(k).unwrap()));
        }
        self.0[usize::from(i)] = o;
    }
}

macro_attr! {
    #[derive(Component!(class=WindowNodeClass))]
    struct WindowNode<State: ?Sized> {
        parent: Option<Window<State>>,
        prev: Window<State>,
        next: Window<State>,
        first_child: Option<Window<State>>,
        widget: Box<dyn Widget<State>>,
        data: Box<dyn Any>,
        palette: Palette,
        measure_size: Option<(Option<i16>, Option<i16>)>,
        desired_size: Vector,
        arrange_bounds: Option<Rect>,
        bounds: Rect,
        h_align: Option<HAlign>,
        v_align: Option<VAlign>,
        margin: Thickness,
        min_size: Vector,
        max_size: Vector,
    }
}

fn offset_from_root<State: ?Sized>(
    mut window: Window<State>,
    tree: &WindowTree<State>
) -> Vector {
    let mut offset = Vector::null();
    loop {
        offset += tree.arena[window.0].bounds.tl.offset_from(Point { x: 0, y: 0 });
        if let Some(parent) = tree.arena[window.0].parent {
            window = parent;
        } else {
            break;
        }
    }
    offset
}

macro_attr! {
    #[derive(NewtypeComponentId!)]
    #[derive(Educe)]
    #[educe(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Window<State: ?Sized>(Id<WindowNode<State>>);
}

impl<State: ?Sized> Window<State> {
    pub fn new(
        tree: &mut WindowTree<State>,
        widget: Box<dyn Widget<State>>,
        data: Box<dyn Any>,
        parent: Self,
        prev: Option<Self>,
    ) -> Result<Self, Error> {
        tree.arena.try_reserve().map_err(|_| Error::Oom)?;
        let window = tree.arena.insert(move |window| {
            (WindowNode {
                parent: Some(parent),
                prev: Window(window),
                next: Window(window),
                first_child: None,
                widget,
                data,
                palette: Palette::new(),
                measure_size: None,
                desired_size: Vector::null(),
                arrange_bounds: None,
                bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                h_align: None,
                v_align: None,
                margin: Thickness::all(0),
                min_size: Vector::null(),
                max_size: Vector { x: -1, y: -1 },
            }, Window(window))
        });
        window.attach(tree, parent, prev);
        Ok(window)
    }

    pub fn invalidate_measure(self, tree: &mut WindowTree<State>) {
        let mut window = self;
        loop {
            let node = &mut tree.arena[window.0];
            let old_measure_size = node.measure_size.take();
            if old_measure_size.is_none() { break; }
            let Some(parent) = node.parent else { break; };
            window = parent;
        }
    }

    pub fn invalidate_arrange(self, tree: &mut WindowTree<State>) {
        let mut window = self;
        loop {
            let node = &mut tree.arena[window.0];
            let old_arrange_bounds = node.arrange_bounds.take();
            if old_arrange_bounds.is_none() { break; }
            let Some(parent) = node.parent else { break; };
            window = parent;
        }
    }

    pub fn measure(
        self,
        tree: &mut WindowTree<State>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State
    ) {
        let node = &mut tree.arena[self.0];
        let available_size = Vector { x: available_width.unwrap_or(0), y: available_height.unwrap_or(0) };
        let measure_size = node.margin.shrink_rect_size(available_size).min(node.max_size).max(node.min_size);
        let measure_size = (available_width.map(|_| measure_size.x), available_height.map(|_| measure_size.y));
        if node.measure_size == Some(measure_size) { return; }
        node.measure_size = Some(measure_size);
        let widget = node.widget.clone();
        let measured_size = widget.measure(tree, self, measure_size.0, measure_size.1, state);
        let node = &mut tree.arena[self.0];
        node.desired_size = measured_size.min(node.max_size).max(node.min_size);
        self.invalidate_arrange(tree);
    }

    pub fn arrange(self, tree: &mut WindowTree<State>, final_bounds: Rect, state: &mut State) {
        let node = &mut tree.arena[self.0];
        let shrinked_bounds = node.margin.shrink_rect(final_bounds);
        let shrinked_bounds_size = Vector {
            x: if node.h_align.is_none() { shrinked_bounds.w() } else { node.desired_size.x },
            y: if node.v_align.is_none() { shrinked_bounds.h() } else { node.desired_size.y }
        };
        let arrange_bounds_size = shrinked_bounds_size.min(node.max_size).max(node.min_size);
        let arrange_bounds_margin = Thickness::align(
            arrange_bounds_size,
            shrinked_bounds.size,
            node.h_align.unwrap_or(HAlign::Left),
            node.v_align.unwrap_or(VAlign::Top)
        );
        let arrange_bounds = arrange_bounds_margin.shrink_rect(shrinked_bounds);
        debug_assert_eq!(arrange_bounds.size, arrange_bounds_size);
        if node.arrange_bounds == Some(arrange_bounds) { return; }
        node.arrange_bounds = Some(arrange_bounds);
        let widget = node.widget.clone();
        let arranged_size = widget.arrange(
            tree,
            self,
            Rect { tl: Point { x: 0, y: 0 }, size: arrange_bounds.size },
            state
        );
        let node = &mut tree.arena[self.0];
        let arranged_size = arranged_size.min(node.max_size).max(node.min_size);
        let arranged_bounds_margin = Thickness::align(
            arranged_size,
            arrange_bounds.size,
            node.h_align.unwrap_or(HAlign::Left),
            node.v_align.unwrap_or(VAlign::Top)
        );
        let arranged_bounds = arranged_bounds_margin.shrink_rect(arrange_bounds);
        debug_assert_eq!(arranged_bounds.size, arranged_size);
        self.move_xy_raw(tree, arranged_bounds);
    }

    pub fn desired_size(
        self,
        tree: &WindowTree<State>
    ) -> Vector {
        tree.arena[self.0].desired_size
    }

    pub fn bounds(
        self,
        tree: &WindowTree<State>
    ) -> Rect {
        tree.arena[self.0].bounds
    }

    pub fn inner_bounds(
        self,
        tree: &WindowTree<State>
    ) -> Rect {
        let bounds = self.bounds(tree);
        Rect { tl: Point { x: 0, y: 0 }, size: bounds.size }
    }

    pub fn data<T: 'static>(
        self,
        tree: &WindowTree<State>
    ) -> &T {
        tree.arena[self.0].data.downcast_ref::<T>().expect("wrong type")
    }

    pub fn data_mut<T: 'static>(
        self,
        tree: &mut WindowTree<State>
    ) -> &mut T {
        tree.arena[self.0].data.downcast_mut::<T>().expect("wrong type")
    }

    pub fn palette(self, tree: &WindowTree<State>) -> &Palette {
        &tree.arena[self.0].palette
    }

    pub fn palette_mut<T>(self, tree: &mut WindowTree<State>, f: impl FnOnce(&mut Palette) -> T) -> T {
        let res = f(&mut tree.arena[self.0].palette);
        self.invalidate_subtree(tree);
        res
    }

    pub fn color(self, tree: &mut WindowTree<State>, i: u8) -> (Fg, Bg) {
        let mut window = self;
        let mut index = i;
        loop {
            match window.palette(tree).get(index) {
                Left(i) => if let Some(parent) = window.parent(tree) {
                    window = parent;
                    index = i;
                } else {
                    break (Fg::Red, Bg::Green);
                },
                Right(c) => break c,
            }
        }
    }

    fn invalidate_subtree(self, tree: &mut WindowTree<State>) {
        self.invalidate(tree);
        if let Some(first_child) = self.first_child(tree) {
            let mut child = first_child;
            loop {
                child.invalidate_subtree(tree);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
    }

    pub fn parent(
        self,
        tree: &WindowTree<State>
    ) -> Option<Self> {
        tree.arena[self.0].parent
    }

    pub fn first_child(
        self,
        tree: &WindowTree<State>
    ) -> Option<Self> {
        tree.arena[self.0].first_child
    }

    pub fn prev(
        self,
        tree: &WindowTree<State>
    ) -> Self {
        tree.arena[self.0].prev
    }

    pub fn next(
        self,
        tree: &WindowTree<State>
    ) -> Self {
        tree.arena[self.0].next
    }

    pub fn focus(
        self,
        tree: &mut WindowTree<State>,
        state: &mut State
    ) -> Option<Self> {
        let old_focused = tree.focused;
        if self == old_focused { return None; }
        let widget = tree.arena[self.0].widget.clone();
        let handled = widget.update(tree, self, Event::GotFocus, false, state);
        if !handled { return None; }
        tree.focused = self;
        let widget = tree.arena[old_focused.0].widget.clone();
        widget.update(tree, old_focused, Event::LostFocus, false, state);
        Some(old_focused)
    }

    fn update(
        self,
        tree: &mut WindowTree<State>,
        event: Event,
        preview: bool,
        handled: &mut bool,
        state: &mut State
    ) {
        let parent = self.parent(tree);
        if !*handled && preview {
            if let Some(parent) = parent {
                parent.update(tree, event, true, handled, state);
            }
        }
        if !*handled {
            let widget = tree.arena[self.0].widget.clone();
            *handled = widget.update(tree, self, event, preview, state);
        }
        if !*handled && !preview {
            if let Some(parent) = parent {
                parent.update(tree, event, false, handled, state);
            }
        }
    }

    fn move_xy_raw(
        self,
        tree: &mut WindowTree<State>,
        bounds: Rect
    ) {
        let Some(parent) = tree.arena[self.0].parent else { return; };
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
        let bounds = replace(&mut tree.arena[self.0].bounds, bounds);
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }

    pub fn h_align(self, tree: &WindowTree<State>) -> Option<HAlign> {
        tree.arena[self.0].h_align
    }

    pub fn set_h_align(self, tree: &mut WindowTree<State>, value: Option<HAlign>) {
        tree.arena[self.0].h_align = value;
        self.invalidate_measure(tree);
    }

    pub fn v_align(self, tree: &WindowTree<State>) -> Option<VAlign> {
        tree.arena[self.0].v_align
    }

    pub fn set_v_align(self, tree: &mut WindowTree<State>, value: Option<VAlign>) {
        tree.arena[self.0].v_align = value;
        self.invalidate_measure(tree);
    }

    pub fn margin(self, tree: &WindowTree<State>) -> Thickness {
        tree.arena[self.0].margin
    }

    pub fn set_margin(self, tree: &mut WindowTree<State>, value: Thickness) {
        tree.arena[self.0].margin = value;
        self.invalidate_measure(tree);
    }

    pub fn min_size(self, tree: &WindowTree<State>) -> Vector {
        tree.arena[self.0].min_size
    }

    pub fn set_min_size(self, tree: &mut WindowTree<State>, value: Vector) {
        tree.arena[self.0].min_size = value;
        self.invalidate_measure(tree);
    }

    pub fn max_size(self, tree: &WindowTree<State>) -> Vector {
        tree.arena[self.0].max_size
    }

    pub fn set_max_size(self, tree: &mut WindowTree<State>, value: Vector) {
        tree.arena[self.0].max_size = value;
        self.invalidate_measure(tree);
    }

    pub fn set_width(self, tree: &mut WindowTree<State>, value: i16) {
        let node = &mut tree.arena[self.0];
        node.min_size.x = value;
        node.max_size.x = value;
        self.invalidate_measure(tree);
    }

    pub fn set_height(self, tree: &mut WindowTree<State>, value: i16) {
        let node = &mut tree.arena[self.0];
        node.min_size.y = value;
        node.max_size.y = value;
        self.invalidate_measure(tree);
    }

    pub fn move_z(
        self,
        tree: &mut WindowTree<State>,
        prev: Option<Self>
    ) {
        let parent = self.detach(tree);
        self.attach(tree, parent, prev);
        let bounds = tree.arena[self.0].bounds;
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }

    fn detach(
        self,
        tree: &mut WindowTree<State>
    ) -> Self {
        let node = &mut tree.arena[self.0];
        let parent = node.parent.take().expect("root can not be detached");
        let prev = replace(&mut node.prev, self);
        let next = replace(&mut node.next, self);
        tree.arena[prev.0].next = next;
        tree.arena[next.0].prev = prev;
        let parent_node = &mut tree.arena[parent.0];
        if parent_node.first_child.unwrap() == self {
            parent_node.first_child = if next == self { None } else { Some(next) };
        }
        parent.invalidate_measure(tree);
        parent
    }

    fn attach(
        self,
        tree: &mut WindowTree<State>,
        parent: Self,
        prev: Option<Self>
    ) {
        let (prev, next) = if let Some(prev) = prev {
            assert_eq!(tree.arena[prev.0].parent.unwrap(), parent);
            let next = replace(&mut tree.arena[prev.0].next, self);
            tree.arena[next.0].prev = self;
            (prev, next)
        } else {
            let parent_node = &mut tree.arena[parent.0];
            let next = parent_node.first_child.replace(self).unwrap_or(self);
            let prev = replace(&mut tree.arena[next.0].prev, self);
            tree.arena[prev.0].next = self;
            (prev, next)
        };
        let node = &mut tree.arena[self.0];
        node.parent = Some(parent);
        node.prev = prev;
        node.next = next;
        parent.invalidate_measure(tree);
    }

    pub fn drop_window(
        self,
        tree: &mut WindowTree<State>
    ) {
        let parent = self.detach(tree);
        let node = tree.arena.remove(self.0);
        let screen_bounds = node.bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
        Self::drop_node_tree(node, tree);
    }

    fn drop_node_tree(
        node: WindowNode<State>,
        tree: &mut WindowTree<State>
    ) {
        if let Some(first_child) = node.first_child {
            let mut child = first_child;
            loop {
                let child_node = tree.arena.remove(child.0);
                child = child_node.next;
                Self::drop_node_tree(child_node, tree);
                if child == first_child { break; }
            }
        }
    }

    pub fn invalidate_rect(
        self,
        tree: &mut WindowTree<State>,
        rect: Rect
    ) {
        let bounds = tree.arena[self.0].bounds;
        let rect = rect.offset(bounds.tl.offset_from(Point { x: 0, y: 0 })).intersect(bounds);
        let parent = tree.arena[self.0].parent.unwrap();
        let screen_rect = rect.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_rect);
    }
 
    pub fn invalidate(
        self,
        tree: &mut WindowTree<State>
    ) {
        let bounds = tree.arena[self.0].bounds;
        let parent = tree.arena[self.0].parent.unwrap();
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }
}

pub struct WindowTree<State: ?Sized + 'static> {
    screen: Option<Box<dyn Screen>>,
    arena: Arena<WindowNode<State>>,
    root: Window<State>,
    focused: Window<State>,
    cursor: Option<Point>,
}

fn root_palette() -> Palette {
    let mut p = Palette::new();
    p.set(0, Right((Fg::Blue, Bg::LightGray)));

    p.set(1, Right((Fg::Black, Bg::LightGray)));
    p.set(2, Right((Fg::DarkGray, Bg::LightGray)));
    p.set(3, Right((Fg::Red, Bg::LightGray)));
    p.set(4, Right((Fg::Black, Bg::Green)));
    p.set(5, Right((Fg::DarkGray, Bg::Green)));
    p.set(6, Right((Fg::Red, Bg::Green)));

    p.set(7, Right((Fg::LightGray, Bg::Blue)));
    p.set(8, Right((Fg::White, Bg::Blue)));
    p.set(9, Right((Fg::LightGreen, Bg::Blue)));

    p.set(10, Right((Fg::Black, Bg::LightGray)));
    p.set(11, Right((Fg::Yellow, Bg::LightGray)));
    p.set(12, Right((Fg::LightGray, Bg::Blue)));
    p.set(13, Right((Fg::LightGray, Bg::Red)));

    p
}

impl<State: ?Sized> WindowTree<State> {
    pub fn new(
        screen: Box<dyn Screen>,
        root_widget: Box<dyn Widget<State>>,
        root_data: Box<dyn Any>,
    ) -> Result<Self, Error> {
        let mut arena = Arena::new();
        arena.try_reserve().map_err(|_| Error::Oom)?;
        let screen_size = screen.size();
        let root = arena.insert(|window| (WindowNode {
            parent: None,
            prev: Window(window),
            next: Window(window),
            first_child: None,
            widget: root_widget,
            data: root_data,
            measure_size: Some((Some(screen_size.x), Some(screen_size.y))),
            desired_size: screen_size,
            arrange_bounds: Some(Rect { tl: Point { x: 0, y: 0 }, size: screen_size }),
            bounds: Rect { tl: Point { x: 0, y: 0 }, size: screen_size },
            h_align: None,
            v_align: None,
            margin: Thickness::all(0),
            min_size: Vector::null(),
            max_size: Vector { x: -1, y: -1 },
            palette: root_palette(),
        }, Window(window)));
        Ok(WindowTree {
            screen: Some(screen),
            arena,
            root,
            focused: root,
            cursor: None
        })
    }

    pub fn root(&self) -> Window<State> { self.root }

    pub fn focused(&self) -> Window<State> { self.focused }

    fn screen(&mut self) -> &mut dyn Screen {
        self.screen.as_mut().expect("WindowTree is in invalid state").as_mut()
    }

    fn render_window(&mut self, window: Window<State>, offset: Vector, render_state: &mut State) {
        let bounds = self.arena[window.0].bounds.offset(offset);
        let screen = self.screen();
        if !rect_invalidated(screen, bounds) { return; }
        let offset = bounds.tl.offset_from(Point { x: 0, y: 0 });
        let screen = self.screen.take().expect("WindowTree is in invalid state");
        let mut port = RenderPort {
            screen,
            cursor: self.cursor,
            offset,
            size: bounds.size,
        };
        let widget = self.arena[window.0].widget.clone();
        widget.render(self, window, &mut port, render_state);
        self.screen.replace(port.screen);
        self.cursor = port.cursor;
        if let Some(first_child) = self.arena[window.0].first_child {
            let mut child = first_child;
            loop {
                self.render_window(child, offset, render_state);
                child = self.arena[child.0].next;
                if child == first_child { break; }
            }
        }
    }

    pub fn update(&mut self, wait: bool, state: &mut State) -> Result<(), Error> {
        let root = self.root;
        let screen = self.screen.as_mut().expect("WindowTree is in invalid state");
        let screen_size = screen.size();
        root.measure(self, Some(screen_size.x), Some(screen_size.y), state);
        root.arrange(self, Rect { tl: Point { x: 0, y: 0 }, size: screen_size }, state);
        if let Some(cursor) = self.cursor {
            let screen = self.screen();
            if rect_invalidated(screen, Rect { tl: cursor, size: Vector { x: 1, y: 1 } }) {
                self.cursor = None;
            }
        }
        self.render_window(self.root, Vector::null(), state);
        let screen = self.screen.as_mut().expect("WindowTree is in invalid state");
        if let Some(screen_Event::Key(n, key)) = screen.update(self.cursor, wait)? {
            let mut handled = false;
            self.focused.update(self, Event::Key(n, key), true, &mut handled, state);
            self.focused.update(self, Event::Key(n, key), false, &mut handled, state);
        }
        Ok(())
    }
}
