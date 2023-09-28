#![feature(effects)]
#![feature(never_type)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::non_canonical_partial_ord_impl)]
#![allow(clippy::type_complexity)]

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use components_arena::{Arena, Component, ComponentId, Id, NewtypeComponentId, RawId};
use core::cmp::{max, min};
use core::mem::replace;
use core::num::NonZeroU16;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use educe::Educe;
use either::{Either, Left, Right};
use macro_attr_2018::macro_attr;
use timer_no_std::{MonoClock, MonoTime};
use tuifw_screen_base::{Bg, Error, Fg, Key, Point, Rect, Screen, Vector};
use tuifw_screen_base::Event as screen_Event;
use tuifw_screen_base::{HAlign, VAlign, Thickness, Range1d};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Event {
    Key(NonZeroU16, Key),
    PreviewKey(NonZeroU16, Key),
    Cmd(u16),
    PreviewCmd(u16),
}

impl Event {
    pub fn is_preview(self) -> bool {
        match self {
            Event::Key(_, _) => false,
            Event::PreviewKey(_, _) => true,
            Event::Cmd(_) => false,
            Event::PreviewCmd(_) => true,
        }
    }

    fn preview(self) -> Self {
        match self {
            Event::Key(n, k) => Event::PreviewKey(n, k),
            Event::Cmd(n) => Event::PreviewCmd(n),
            _ => unreachable!(),
        }
    }
}

pub const CMD_GOT_PRIMARY_FOCUS: u16 = 0;

pub const CMD_LOST_PRIMARY_FOCUS: u16 = 1;

pub const CMD_GOT_SECONDARY_FOCUS: u16 = 2;

pub const CMD_LOST_SECONDARY_FOCUS: u16 = 3;

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

    pub fn fill_bg(&mut self, bg: Bg) {
        self.fill(|rp, p| rp.out(p, Fg::LightGray, bg, " "));
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
        event_source: Window<State>,
        state: &mut State,
    ) -> bool;
}

clone_trait_object!(<State: ?Sized> Widget<State>);

pub trait WidgetData<State: ?Sized>: Downcast {
    fn drop_widget_data(&mut self, _tree: &mut WindowTree<State>, _state: &mut State) { }
}

impl_downcast!(WidgetData<State> where State: ?Sized);

pub trait Layout: Downcast { }

impl_downcast!(Layout);

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

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EventHandler<State: ?Sized>: DynClone {
    fn invoke(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        event_source: Window<State>,
        state: &mut State,
    ) -> bool;
}

clone_trait_object!(<State> EventHandler<State> where State: ?Sized);

macro_attr! {
    #[derive(Component!(class=WindowNodeClass))]
    struct WindowNode<State: ?Sized> {
        parent: Option<Window<State>>,
        prev: Window<State>,
        next: Window<State>,
        first_child: Option<Window<State>>,
        widget: Box<dyn Widget<State>>,
        data: Box<dyn WidgetData<State>>,
        layout: Option<Box<dyn Layout>>,
        palette: Palette,
        measure_size: Option<(Option<i16>, Option<i16>)>,
        desired_size: Vector,
        arrange_size: Option<Vector>,
        render_bounds: Rect,
        window_bounds: Rect,
        h_align: Option<HAlign>,
        v_align: Option<VAlign>,
        margin: Thickness,
        min_size: Vector,
        max_size: Vector,
        event_handler: Option<Box<dyn EventHandler<State>>>,
        next_focus: Window<State>,
        contains_primary_focus: bool,
    }
}

fn offset_from_root<State: ?Sized>(
    mut window: Window<State>,
    tree: &WindowTree<State>
) -> Vector {
    let mut offset = Vector::null();
    loop {
        offset += tree.arena[window.0].window_bounds.tl.offset_from(Point { x: 0, y: 0 });
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
        data: Box<dyn WidgetData<State>>,
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
                event_handler: None,
                widget,
                data,
                layout: None,
                palette: Palette::new(),
                measure_size: None,
                desired_size: Vector::null(),
                arrange_size: None,
                render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                window_bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                h_align: None,
                v_align: None,
                margin: Thickness::all(0),
                min_size: Vector::null(),
                max_size: Vector { x: -1, y: -1 },
                next_focus: Window(window),
                contains_primary_focus: false,
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
            let old_arrange_size = node.arrange_size.take();
            if old_arrange_size.is_none() { break; }
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
        node.desired_size = node.margin.expand_rect_size(measured_size.min(node.max_size).max(node.min_size));
        self.invalidate_arrange(tree);
    }

    pub fn arrange(self, tree: &mut WindowTree<State>, final_bounds: Rect, state: &mut State) {
        let node = &mut tree.arena[self.0];
        let margined_bounds = node.margin.shrink_rect(final_bounds);
        let arrange_size = Vector {
            x: if node.h_align.is_none() { margined_bounds.w() } else { node.desired_size.x },
            y: if node.v_align.is_none() { margined_bounds.h() } else { node.desired_size.y }
        };
        let arrange_size = arrange_size.min(node.max_size).max(node.min_size);
        if node.arrange_size == Some(arrange_size) { return; }
        node.arrange_size = Some(arrange_size);
        let widget = node.widget.clone();
        let arranged_size = widget.arrange(
            tree,
            self,
            Rect { tl: Point { x: 0, y: 0 }, size: arrange_size },
            state
        );
        let node = &mut tree.arena[self.0];
        let arranged_size = arranged_size.min(node.max_size).max(node.min_size);
        let arranged_bounds = Thickness::align(
            arranged_size,
            margined_bounds.size,
            node.h_align.unwrap_or(HAlign::Left),
            node.v_align.unwrap_or(VAlign::Top)
        ).shrink_rect(margined_bounds).intersect(margined_bounds);
        node.render_bounds = final_bounds;
        self.move_xy_raw(tree, arranged_bounds);
    }

    pub fn set_event_handler(
        self,
        tree: &mut WindowTree<State>,
        handler: Option<Box<dyn EventHandler<State>>>
    ) {
        tree.arena[self.0].event_handler = handler;
    }

    pub fn desired_size(
        self,
        tree: &WindowTree<State>
    ) -> Vector {
        tree.arena[self.0].desired_size
    }

    pub fn render_bounds(
        self,
        tree: &WindowTree<State>
    ) -> Rect {
        tree.arena[self.0].render_bounds
    }

    pub fn inner_bounds(
        self,
        tree: &WindowTree<State>
    ) -> Rect {
        let window_bounds = tree.arena[self.0].window_bounds;
        Rect { tl: Point { x: 0, y: 0 }, size: window_bounds.size }
    }

    pub fn data<'a, T: WidgetData<State> + 'static>(
        self,
        tree: &'a WindowTree<'_, State>
    ) -> &'a T {
        tree.arena[self.0].data.downcast_ref::<T>().expect("wrong type")
    }

    pub fn data_mut<'a, T: WidgetData<State> + 'static>(
        self,
        tree: &'a mut WindowTree<'_, State>
    ) -> &'a mut T {
        tree.arena[self.0].data.downcast_mut::<T>().expect("wrong type")
    }

    pub fn layout<'a, T: Layout + 'static>(
        self,
        tree: &'a WindowTree<'_, State>
    ) -> Option<&'a T> {
        tree.arena[self.0].layout.as_ref().and_then(|x| x.downcast_ref::<T>())
    }

    pub fn layout_mut<R>(
        self,
        tree: &mut WindowTree<State>,
        f: impl FnOnce(&mut Option<Box<dyn Layout>>) -> R
    ) -> R {
        let layout = &mut tree.arena[self.0].layout;
        let res = f(layout);
        if let Some(parent) = self.parent(tree) {
            parent.invalidate_measure(tree);
        }
        res
    }

    pub fn next_focus(self, tree: &WindowTree<State>) -> Self {
        tree.arena[self.0].next_focus
    }

    pub fn set_next_focus(self, tree: &mut WindowTree<State>, value: Self) {
        tree.arena[self.0].next_focus = value;
    }

    pub fn palette<'a>(self, tree: &'a WindowTree<'_, State>) -> &'a Palette {
        &tree.arena[self.0].palette
    }

    pub fn palette_mut<T>(self, tree: &mut WindowTree<State>, f: impl FnOnce(&mut Palette) -> T) -> T {
        let res = f(&mut tree.arena[self.0].palette);
        self.invalidate_render(tree);
        res
    }

    pub fn color(self, tree: &WindowTree<State>, i: u8) -> (Fg, Bg) {
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

    pub fn is_focused(self, tree: &WindowTree<State>) -> bool {
        tree.primary_focused == self || tree.secondary_focused == self
    }

    pub fn focus(
        self,
        tree: &mut WindowTree<State>,
        primary: bool,
        state: &mut State
    ) -> Option<Self> {
        if primary {
            let old_focused = tree.primary_focused;
            if self == old_focused { return None; }
            let handled = self.raise(tree, Event::Cmd(CMD_GOT_PRIMARY_FOCUS), state);
            if !handled { return None; }

            let mut window = tree.primary_focused;
            loop {
                tree.arena[window.0].contains_primary_focus = false;
                if let Some(parent) = window.parent(tree) {
                    window = parent;
                } else {
                    break;
                }
            }
            tree.primary_focused = self;
            let mut window = tree.primary_focused;
            loop {
                tree.arena[window.0].contains_primary_focus = true;
                if let Some(parent) = window.parent(tree) {
                    window = parent;
                } else {
                    break;
                }
            }

            old_focused.raise(tree, Event::Cmd(CMD_LOST_PRIMARY_FOCUS), state);
            Some(old_focused)
        } else {
            let old_focused = tree.secondary_focused;
            if self == old_focused { return None; }
            let handled = self.raise(tree, Event::Cmd(CMD_GOT_SECONDARY_FOCUS), state);
            if !handled { return None; }
            tree.secondary_focused = self;
            old_focused.raise(tree, Event::Cmd(CMD_LOST_SECONDARY_FOCUS), state);
            Some(old_focused)
        }
    }

    pub fn raise(
        self,
        tree: &mut WindowTree<State>,
        event: Event,
        state: &mut State
    ) -> bool {
        self.raise_raw(tree, event, false, state)
    }

    fn raise_raw(
        self,
        tree: &mut WindowTree<State>,
        event: Event,
        secondary: bool,
        state: &mut State
    ) -> bool {
        let mut handled = false;
        self.raise_core(tree, event.preview(), self, secondary, &mut handled, state);
        if !handled {
            self.raise_core(tree, event, self, secondary, &mut handled, state);
        }
        handled
    }

    fn raise_core(
        self,
        tree: &mut WindowTree<State>,
        event: Event,
        event_source: Window<State>,
        secondary: bool,
        handled: &mut bool,
        state: &mut State
    ) {
        if secondary && tree.arena[self.0].contains_primary_focus { return; }
        let parent = self.parent(tree);
        if !*handled && event.is_preview() {
            if let Some(parent) = parent {
                parent.raise_core(tree, event, event_source, secondary, handled, state);
            }
        }
        if !*handled {
            let node = &tree.arena[self.0];
            let widget = node.widget.clone();
            let event_handler = node.event_handler.clone();
            *handled = widget.update(tree, self, event, event_source, state);
            if !*handled {
                if let Some(event_handler) = event_handler {
                    *handled = event_handler.invoke(tree, self, event, event_source, state);
                }
            }
        }
        if !*handled && !event.is_preview() {
            if let Some(parent) = parent {
                parent.raise_core(tree, event, event_source, secondary, handled, state);
            }
        }
    }

    fn move_xy_raw(
        self,
        tree: &mut WindowTree<State>,
        window_bounds: Rect
    ) {
        let Some(parent) = tree.arena[self.0].parent else { return; };
        let screen_bounds = window_bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
        let window_bounds = replace(&mut tree.arena[self.0].window_bounds, window_bounds);
        let screen_bounds = window_bounds.offset(offset_from_root(parent, tree));
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
        let bounds = tree.arena[self.0].window_bounds;
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
        tree: &mut WindowTree<State>,
        state: &mut State,
    ) {
        let parent = self.detach(tree);
        let mut node = tree.arena.remove(self.0);
        node.data.drop_widget_data(tree, state);
        let screen_bounds = node.window_bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
        Self::drop_node_tree(node.first_child, tree, state);
    }

    fn drop_node_tree(
        first_child: Option<Window<State>>,
        tree: &mut WindowTree<State>,
        state: &mut State,
    ) {
        if let Some(first_child) = first_child {
            let mut child = first_child;
            loop {
                let mut child_node = tree.arena.remove(child.0);
                child_node.data.drop_widget_data(tree, state);
                child = child_node.next;
                Self::drop_node_tree(child_node.first_child, tree, state);
                if child == first_child { break; }
            }
        }
    }

    pub fn invalidate_rect(
        self,
        tree: &mut WindowTree<State>,
        rect: Rect
    ) {
        let bounds = tree.arena[self.0].window_bounds;
        let rect = rect.offset(bounds.tl.offset_from(Point { x: 0, y: 0 })).intersect(bounds);
        let screen_rect = if let Some(parent) = tree.arena[self.0].parent {
            rect.offset(offset_from_root(parent, tree))
        } else {
            rect
        };
        invalidate_rect(tree.screen(), screen_rect);
    }
 
    pub fn invalidate_render(
        self,
        tree: &mut WindowTree<State>
    ) {
        let bounds = tree.arena[self.0].window_bounds;
        let screen_bounds = if let Some(parent) = tree.arena[self.0].parent {
            bounds.offset(offset_from_root(parent, tree))
        } else {
            bounds
        };
        invalidate_rect(tree.screen(), screen_bounds);
    }
}

const FPS: u16 = 40;

fn root_palette() -> Palette {
    let mut p = Palette::new();
    p.set(0, Right((Fg::LightGray, Bg::None))); // background

    p.set(11, Right((Fg::LightGray, Bg::None))); // static text

    p.set(12, Right((Fg::LightGray, Bg::Blue))); // input line normal
    p.set(13, Right((Fg::LightGray, Bg::Red))); // input line invalid

    p.set(14, Right((Fg::Cyan, Bg::None))); // button normal
    p.set(15, Right((Fg::Black, Bg::Cyan))); // button focused
    p.set(16, Right((Fg::Cyan, Bg::None))); // button pressed
    p.set(17, Right((Fg::LightGray, Bg::None))); // button disabled

    p
}

macro_attr! {
    #[derive(Component!(class=TimeDataClass))]
    struct TimerData<State: ?Sized + 'static> {
        start: MonoTime,
        span_ms: u16,
        alarm: Box<dyn FnOnce(&mut WindowTree<State>, &mut State)>,
    }
}

macro_attr! {
    #[derive(NewtypeComponentId!)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Timer(RawId);
}

impl Timer {
    pub fn new<State: ?Sized>(
        tree: &mut WindowTree<State>,
        span_ms: u16,
        alarm: Box<dyn FnOnce(&mut WindowTree<State>, &mut State)>
    ) -> Self {
        let start = tree.clock.time();
        tree.timers.insert(move |id| (TimerData {
            start,
            span_ms,
            alarm
        }, Timer(id.into_raw())))
    }

    pub fn drop_timer<State: ?Sized>(self, tree: &mut WindowTree<State>) {
        tree.timers.remove(Id::from_raw(self.0));
    }
}

pub struct WindowTree<'clock, State: ?Sized + 'static> {
    screen: Option<Box<dyn Screen>>,
    arena: Arena<WindowNode<State>>,
    root: Window<State>,
    primary_focused: Window<State>,
    secondary_focused: Window<State>,
    cursor: Option<Point>,
    quit: bool,
    timers: Arena<TimerData<State>>,
    clock: &'clock MonoClock,
}

impl<'clock, State: ?Sized> WindowTree<'clock, State> {
    pub fn new(
        screen: Box<dyn Screen>,
        clock: &'clock MonoClock,
        root_widget: Box<dyn Widget<State>>,
        root_data: Box<dyn WidgetData<State>>,
    ) -> Result<Self, Error> {
        let mut arena = Arena::new();
        arena.try_reserve().map_err(|_| Error::Oom)?;
        let screen_size = screen.size();
        let root = arena.insert(|window| (WindowNode {
            parent: None,
            prev: Window(window),
            next: Window(window),
            first_child: None,
            event_handler: None,
            widget: root_widget,
            data: root_data,
            layout: None,
            measure_size: Some((Some(screen_size.x), Some(screen_size.y))),
            desired_size: screen_size,
            arrange_size: Some(screen_size),
            render_bounds: Rect { tl: Point { x: 0, y: 0 }, size: screen_size },
            window_bounds: Rect { tl: Point { x: 0, y: 0 }, size: screen_size },
            h_align: None,
            v_align: None,
            margin: Thickness::all(0),
            min_size: Vector::null(),
            max_size: Vector { x: -1, y: -1 },
            palette: root_palette(),
            next_focus: Window(window),
            contains_primary_focus: true,
        }, Window(window)));
        Ok(WindowTree {
            screen: Some(screen),
            arena,
            root,
            primary_focused: root,
            secondary_focused: root,
            cursor: None,
            quit: false,
            clock,
            timers: Arena::new(),
        })
    }

    pub fn root(&self) -> Window<State> { self.root }

    pub fn primary_focused(&self) -> Window<State> { self.primary_focused }

    pub fn secondary_focused(&self) -> Window<State> { self.secondary_focused }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    fn screen(&mut self) -> &mut dyn Screen {
        self.screen.as_mut().expect("WindowTree is in invalid state").as_mut()
    }

    fn render_window(&mut self, window: Window<State>, offset: Vector, render_state: &mut State) {
        let bounds = self.arena[window.0].window_bounds.offset(offset);
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

    pub fn run(&mut self, state: &mut State) -> Result<(), Error> {
        let mut time = self.clock.time();
        while !self.quit {
            let timers_time = self.clock.time();
            loop {
                let timer = self.timers.items().iter()
                    .find(|(_, data)| timers_time.delta_ms_u16(data.start).unwrap_or(u16::MAX) >= data.span_ms)
                    .map(|(id, _)| id)
                ;
                if let Some(timer) = timer {
                    let alarm = self.timers.remove(timer).alarm;
                    alarm(self, state);
                } else {
                    break;
                }
            }
            let ms = time.split_ms_u16(self.clock).unwrap_or(u16::MAX);
            self.update(false, state)?;
            assert!(FPS != 0 && u16::MAX / FPS > 8);
            self.clock.sleep_ms_u16((1000 / FPS).saturating_sub(ms));
        }
        Ok(())
    }

    fn update(&mut self, wait: bool, state: &mut State) -> Result<(), Error> {
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
            if key == Key::Tab {
                let next_focus = self.primary_focused.next_focus(self);
                if next_focus.focus(self, true, state).is_some() { return Ok(()); }
            }
            let handled = self.primary_focused.raise_raw(self, Event::Key(n, key), false, state);
            if !handled {
                self.secondary_focused.raise_raw(self, Event::Key(n, key), true, state);
            }
        }
        Ok(())
    }
}
