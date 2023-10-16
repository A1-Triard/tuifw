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
#![allow(clippy::partialeq_to_none)]
#![allow(clippy::type_complexity)]

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;
use components_arena::{Arena, Component, ComponentId, Id, NewtypeComponentId, RawId};
use core::cmp::{max, min};
use core::mem::replace;
use core::ops::RangeInclusive;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use educe::Educe;
use either::{Either, Left, Right};
use iter_identify_first_last::IteratorIdentifyFirstLastExt;
use macro_attr_2018::macro_attr;
use timer_no_std::{MonoClock, MonoTime};
use tuifw_screen_base::{Bg, Error, Fg, Key, Point, Rect, Screen, Vector};
use tuifw_screen_base::Event as screen_Event;
use tuifw_screen_base::{HAlign, VAlign, Thickness, Range1d, text_width};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Event {
    Key(Key),
    PreviewKey(Key),
    PreProcessKey(Key),
    PostProcessKey(Key),
    Cmd(u16),
    PreviewCmd(u16),
}

impl Event {
    pub fn is_preview(self) -> bool {
        match self {
            Event::Key(_) => false,
            Event::PreviewKey(_) => true,
            Event::Cmd(_) => false,
            Event::PreviewCmd(_) => true,
            Event::PreProcessKey(_) => false,
            Event::PostProcessKey(_) => false,
        }
    }

    fn preview(self) -> Self {
        match self {
            Event::Key(k) => Event::PreviewKey(k),
            Event::Cmd(n) => Event::PreviewCmd(n),
            _ => unreachable!(),
        }
    }
}

pub const CMD_GOT_PRIMARY_FOCUS: u16 = 0;

pub const CMD_LOST_PRIMARY_FOCUS: u16 = 1;

pub const CMD_GOT_SECONDARY_FOCUS: u16 = 2;

pub const CMD_LOST_SECONDARY_FOCUS: u16 = 3;

pub const CMD_LOST_ATTENTION: u16 = 4;

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
    pub fn text(&mut self, p: Point, color: (Fg, Bg), text: &str) {
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
            let out = self.screen.out(p, color.0, color.1, text, chunk.clone(), row.clone());
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

    pub fn label(&mut self, mut p: Point, color: (Fg, Bg), color_hotkey: (Fg, Bg), text: &str) {
        let mut hotkey = false;
        for (first, last, text) in text.split('~').identify_first_last() {
            if !first && !text.is_empty() {
                hotkey = !hotkey;
            }
            let actual_text = if !first && !last && text.is_empty() { "~" } else { text };
            self.text(p, if hotkey { color_hotkey } else { color }, actual_text);
            p = p.offset(Vector { x: text_width(actual_text), y: 0 });
            if !first && text.is_empty() {
                hotkey = !hotkey;
            }
        }
    }

    pub fn fill_bg(&mut self, bg: Bg) {
        self.fill(|rp, p| rp.text(p, (Fg::LightGray, bg), " "));
    }

    pub fn h_line(&mut self, start: Point, len: i16, double: bool, color: (Fg, Bg)) {
        let s = if double { "═" } else { "─" };
        for x in Range1d::new(start.x, start.x.wrapping_add(len)) {
            self.text(Point { x, y: start.y }, color, s);
        }
    }

    pub fn v_line(&mut self, start: Point, len: i16, double: bool, color: (Fg, Bg)) {
        let s = if double { "║" } else { "│" };
        for y in Range1d::new(start.y, start.y.wrapping_add(len)) {
            self.text(Point { x: start.x, y }, color, s);
        }
    }

    pub fn tl_edge(&mut self, p: Point, double: bool, color: (Fg, Bg)) {
        self.text(p, color, if double { "╔" } else { "┌" });
    }

    pub fn tr_edge(&mut self, p: Point, double: bool, color: (Fg, Bg)) {
        self.text(p, color, if double { "╗" } else { "┐" });
    }

    pub fn bl_edge(&mut self, p: Point, double: bool, color: (Fg, Bg)) {
        self.text(p, color, if double { "╚" } else { "└" });
    }

    pub fn br_edge(&mut self, p: Point, double: bool, color: (Fg, Bg)) {
        self.text(p, color, if double { "╝" } else { "┘" });
    }
}

pub fn label_width(text: &str) -> i16 {
    let mut width = 0i16;
    let mut hotkey = false;
    for (first, last, text) in text.split('~').identify_first_last() {
        if !first && !text.is_empty() {
            hotkey = !hotkey;
        }
        let actual_text = if !first && !last && text.is_empty() { "~" } else { text };
        width = width.wrapping_add(text_width(actual_text));
        if !first && text.is_empty() {
            hotkey = !hotkey;
        }
    }
    width
}

pub fn label(text: &str) -> Option<char> {
    let mut hotkey = false;
    for (first, last, text) in text.split('~').identify_first_last() {
        if !first && !text.is_empty() {
            hotkey = !hotkey;
        }
        let actual_text = if !first && !last && text.is_empty() { "~" } else { text };
        if hotkey && !actual_text.is_empty() {
            return Some(actual_text.chars().next().unwrap().to_lowercase().next().unwrap());
        }
        if !first && text.is_empty() {
            hotkey = !hotkey;
        }
    }
    None
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapsed
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

    fn secondary_focusable(&self) -> bool { false }

    fn pre_process(&self) -> bool { false }

    fn post_process(&self) -> bool { false }
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
        width: Option<i16>,
        min_width: i16,
        max_width: i16,
        height: Option<i16>,
        min_height: i16,
        max_height: i16,
        event_handler: Option<Box<dyn EventHandler<State>>>,
        focus_tab: Window<State>,
        focus_tab_tag: u16,
        focus_right: Window<State>,
        focus_right_tag: u16,
        focus_left: Window<State>,
        focus_left_tag: u16,
        focus_up: Window<State>,
        focus_up_tag: u16,
        focus_down: Window<State>,
        focus_down_tag: u16,
        contains_primary_focus: bool,
        tag: u16,
        pre_process: Option<Id<PrePostProcess<State>>>,
        post_process: Option<Id<PrePostProcess<State>>>,
        is_enabled: bool,
        visibility: Visibility,
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
        let pre_process = widget.pre_process();
        let post_process = widget.post_process();
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
                width: None,
                min_width: 0,
                max_width: -1,
                height: None,
                min_height: 0,
                max_height: -1,
                focus_tab: Window(window),
                focus_tab_tag: 0,
                focus_right: Window(window),
                focus_right_tag: 0,
                focus_left: Window(window),
                focus_left_tag: 0,
                focus_up: Window(window),
                focus_up_tag: 0,
                focus_down: Window(window),
                focus_down_tag: 0,
                contains_primary_focus: false,
                tag: 0,
                pre_process: None,
                post_process: None,
                is_enabled: true,
                visibility: Visibility::Visible,
            }, Window(window))
        });
        window.attach(tree, parent, prev);
        if pre_process {
            let id = tree.pre_process.insert(|id| (PrePostProcess(window), id));
            tree.arena[window.0].pre_process = Some(id);
        }
        if post_process {
            let id = tree.post_process.insert(|id| (PrePostProcess(window), id));
            tree.arena[window.0].post_process = Some(id);
        }
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
        if node.visibility == Visibility::Collapsed {
            node.desired_size = Vector::null();
            return;
        }
        let min_size = Vector {
            x: node.width.unwrap_or(node.min_width),
            y: node.height.unwrap_or(node.min_height)
        };
        let max_size = Vector {
            x: node.width.unwrap_or(node.max_width),
            y: node.height.unwrap_or(node.max_height)
        };
        let available_size = Vector { x: available_width.unwrap_or(0), y: available_height.unwrap_or(0) };
        let measure_size = node.margin.shrink_rect_size(available_size).min(max_size).max(min_size);
        let measure_size = (available_width.map(|_| measure_size.x), available_height.map(|_| measure_size.y));
        if node.measure_size == Some(measure_size) { return; }
        node.measure_size = Some(measure_size);
        let widget = node.widget.clone();
        let measured_size = widget.measure(tree, self, measure_size.0, measure_size.1, state);
        let node = &mut tree.arena[self.0];
        node.desired_size = node.margin.expand_rect_size(measured_size.min(max_size).max(min_size));
        self.invalidate_arrange(tree);
    }

    pub fn arrange(self, tree: &mut WindowTree<State>, final_bounds: Rect, state: &mut State) {
        let node = &mut tree.arena[self.0];
        if node.visibility == Visibility::Collapsed {
            let bounds = Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() };
            node.render_bounds = bounds;
            self.move_xy_raw(tree, bounds);
            return;
        }
        let min_size = Vector {
            x: node.width.unwrap_or(node.min_width),
            y: node.height.unwrap_or(node.min_height)
        };
        let max_size = Vector {
            x: node.width.unwrap_or(node.max_width),
            y: node.height.unwrap_or(node.max_height)
        };
        let margined_bounds = node.margin.shrink_rect(final_bounds);
        let arrange_size = Vector {
            x: if node.h_align.is_none() { final_bounds.w() } else { node.desired_size.x },
            y: if node.v_align.is_none() { final_bounds.h() } else { node.desired_size.y }
        };
        let arrange_size = node.margin.shrink_rect_size(arrange_size).min(max_size).max(min_size);
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
        let arranged_size = arranged_size.min(max_size).max(min_size);
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

    pub fn actual_focus_tab(self, tree: &WindowTree<State>) -> Self {
        let node = &tree.arena[self.0];
        if node.focus_tab_tag == 0 {
            node.focus_tab
        } else {
            tree.window_by_tag(node.focus_tab_tag).unwrap()
        }
    }

    pub fn focus_tab(self, tree: &WindowTree<State>) -> Self {
        tree.arena[self.0].focus_tab
    }

    pub fn set_focus_tab(self, tree: &mut WindowTree<State>, value: Self) {
        tree.arena[self.0].focus_tab = value;
    }

    pub fn focus_tab_tag(self, tree: &WindowTree<State>) -> u16 {
        tree.arena[self.0].focus_tab_tag
    }

    pub fn set_focus_tab_tag(self, tree: &mut WindowTree<State>, value: u16) {
        tree.arena[self.0].focus_tab_tag = value;
    }

    pub fn actual_focus_right(self, tree: &WindowTree<State>) -> Self {
        let node = &tree.arena[self.0];
        if node.focus_right_tag == 0 {
            node.focus_right
        } else {
            tree.window_by_tag(node.focus_right_tag).unwrap()
        }
    }

    pub fn focus_right(self, tree: &WindowTree<State>) -> Self {
        tree.arena[self.0].focus_right
    }

    pub fn set_focus_right(self, tree: &mut WindowTree<State>, value: Self) {
        tree.arena[self.0].focus_right = value;
    }

    pub fn focus_right_tag(self, tree: &WindowTree<State>) -> u16 {
        tree.arena[self.0].focus_right_tag
    }

    pub fn set_focus_right_tag(self, tree: &mut WindowTree<State>, value: u16) {
        tree.arena[self.0].focus_right_tag = value;
    }

    pub fn actual_focus_left(self, tree: &WindowTree<State>) -> Self {
        let node = &tree.arena[self.0];
        if node.focus_left_tag == 0 {
            node.focus_left
        } else {
            tree.window_by_tag(node.focus_left_tag).unwrap()
        }
    }

    pub fn focus_left(self, tree: &WindowTree<State>) -> Self {
        tree.arena[self.0].focus_left
    }

    pub fn set_focus_left(self, tree: &mut WindowTree<State>, value: Self) {
        tree.arena[self.0].focus_left = value;
    }

    pub fn focus_left_tag(self, tree: &WindowTree<State>) -> u16 {
        tree.arena[self.0].focus_left_tag
    }

    pub fn set_focus_left_tag(self, tree: &mut WindowTree<State>, value: u16) {
        tree.arena[self.0].focus_left_tag = value;
    }

    pub fn actual_focus_up(self, tree: &WindowTree<State>) -> Self {
        let node = &tree.arena[self.0];
        if node.focus_up_tag == 0 {
            node.focus_up
        } else {
            tree.window_by_tag(node.focus_up_tag).unwrap()
        }
    }

    pub fn focus_up(self, tree: &WindowTree<State>) -> Self {
        tree.arena[self.0].focus_up
    }

    pub fn set_focus_up(self, tree: &mut WindowTree<State>, value: Self) {
        tree.arena[self.0].focus_up = value;
    }

    pub fn focus_up_tag(self, tree: &WindowTree<State>) -> u16 {
        tree.arena[self.0].focus_up_tag
    }

    pub fn set_focus_up_tag(self, tree: &mut WindowTree<State>, value: u16) {
        tree.arena[self.0].focus_up_tag = value;
    }

    pub fn actual_focus_down(self, tree: &WindowTree<State>) -> Self {
        let node = &tree.arena[self.0];
        if node.focus_down_tag == 0 {
            node.focus_down
        } else {
            tree.window_by_tag(node.focus_down_tag).unwrap()
        }
    }

    pub fn focus_down(self, tree: &WindowTree<State>) -> Self {
        tree.arena[self.0].focus_down
    }

    pub fn set_focus_down(self, tree: &mut WindowTree<State>, value: Self) {
        tree.arena[self.0].focus_down = value;
    }

    pub fn focus_down_tag(self, tree: &WindowTree<State>) -> u16 {
        tree.arena[self.0].focus_down_tag
    }

    pub fn set_focus_down_tag(self, tree: &mut WindowTree<State>, value: u16) {
        tree.arena[self.0].focus_down_tag = value;
    }

    pub fn is_focused(self, tree: &WindowTree<State>) -> bool {
        tree.primary_focused == Some(self) || tree.secondary_focused == Some(self)
    }

    pub fn set_focused_primary(self, tree: &mut WindowTree<State>, value: bool) {
        if value {
            tree.next_primary_focused = Some(Some(self));
        } else if
            tree.next_primary_focused == Some(Some(self)) ||
            tree.next_primary_focused == None && tree.primary_focused == Some(self)
        {
            tree.next_primary_focused = Some(None);
        }
    }

    pub fn set_focused_secondary(self, tree: &mut WindowTree<State>, value: bool) {
        if value {
            tree.next_secondary_focused = Some(Some(self));
        } else if
            tree.next_secondary_focused == Some(Some(self)) ||
            tree.next_secondary_focused == None && tree.secondary_focused == Some(self)
        {
            tree.next_secondary_focused = Some(None);
        }
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
                Left(i) => {
                    if let Some(parent) = window.parent(tree) {
                        window = parent;
                        index = i;
                    } else if let Right(color) = tree.palette().get(i) {
                        break color;
                    } else {
                        break (Fg::Red, Bg::Green);
                    }
                },
                Right(c) => break c,
            }
        }
    }

    pub fn is_enabled(self, tree: &WindowTree<State>) -> bool {
        tree.arena[self.0].is_enabled
    }

    pub fn set_is_enabled(self, tree: &mut WindowTree<State>, value: bool) {
        tree.arena[self.0].is_enabled = value;
        self.invalidate_render(tree);
    }

    pub fn actual_is_enabled(self, tree: &WindowTree<State>) -> bool {
        let mut window = self;
        loop {
            if !window.is_enabled(tree) { return false; }
            if let Some(parent) = window.parent(tree) {
                window = parent;
            } else {
                break;
            }
        }
        true
    }

    pub fn visibility(self, tree: &WindowTree<State>) -> Visibility {
        tree.arena[self.0].visibility
    }

    pub fn set_visibility(self, tree: &mut WindowTree<State>, value: Visibility) {
        if self == tree.root { return; }
        let old_value = replace(&mut tree.arena[self.0].visibility, value);
        match (old_value, value) {
            (Visibility::Visible, Visibility::Collapsed) => self.parent(tree).unwrap().invalidate_measure(tree),
            (Visibility::Visible, Visibility::Hidden) => self.invalidate_render(tree),
            (Visibility::Collapsed, Visibility::Visible) => self.parent(tree).unwrap().invalidate_measure(tree),
            (Visibility::Collapsed, Visibility::Hidden) => self.parent(tree).unwrap().invalidate_measure(tree),
            (Visibility::Hidden, Visibility::Visible) => self.invalidate_render(tree),
            (Visibility::Hidden, Visibility::Collapsed) => self.parent(tree).unwrap().invalidate_measure(tree),
            _ => { },
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

    pub fn raise(
        self,
        tree: &mut WindowTree<State>,
        event: Event,
        state: &mut State
    ) -> bool {
        self.raise_priv(tree, event, false, state)
    }

    fn raise_priv(
        self,
        tree: &mut WindowTree<State>,
        event: Event,
        secondary: bool,
        state: &mut State
    ) -> bool {
        let mut handled = false;
        self.raise_raw(tree, event.preview(), self, secondary, &mut handled, state);
        if !handled {
            self.raise_raw(tree, event, self, secondary, &mut handled, state);
        }
        handled
    }

    fn raise_raw(
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
                parent.raise_raw(tree, event, event_source, secondary, handled, state);
            }
        }
        if !*handled {
            *handled = self.raise_core(tree, event, event_source, state);
        }
        if !*handled && !event.is_preview() {
            if let Some(parent) = parent {
                parent.raise_raw(tree, event, event_source, secondary, handled, state);
            }
        }
    }

    fn raise_core(
        self,
        tree: &mut WindowTree<State>,
        event: Event,
        event_source: Window<State>,
        state: &mut State
    ) -> bool {
        let node = &tree.arena[self.0];
        let widget = node.widget.clone();
        let event_handler = node.event_handler.clone();
        let mut handled = widget.update(tree, self, event, event_source, state);
        if !handled {
            if let Some(event_handler) = event_handler {
                handled = event_handler.invoke(tree, self, event, event_source, state);
            }
        }
        handled
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
        if self == tree.root { return; }
        tree.arena[self.0].h_align = value;
        self.invalidate_measure(tree);
    }

    pub fn v_align(self, tree: &WindowTree<State>) -> Option<VAlign> {
        tree.arena[self.0].v_align
    }

    pub fn set_v_align(self, tree: &mut WindowTree<State>, value: Option<VAlign>) {
        if self == tree.root { return; }
        tree.arena[self.0].v_align = value;
        self.invalidate_measure(tree);
    }

    pub fn margin(self, tree: &WindowTree<State>) -> Thickness {
        tree.arena[self.0].margin
    }

    pub fn set_margin(self, tree: &mut WindowTree<State>, value: Thickness) {
        if self == tree.root { return; }
        tree.arena[self.0].margin = value;
        self.invalidate_measure(tree);
    }

    pub fn min_width(self, tree: &WindowTree<State>) -> i16 {
        tree.arena[self.0].min_width
    }

    pub fn set_min_width(self, tree: &mut WindowTree<State>, value: i16) {
        if self == tree.root { return; }
        tree.arena[self.0].min_width = value;
        self.invalidate_measure(tree);
    }

    pub fn min_height(self, tree: &WindowTree<State>) -> i16 {
        tree.arena[self.0].min_height
    }

    pub fn set_min_height(self, tree: &mut WindowTree<State>, value: i16) {
        if self == tree.root { return; }
        tree.arena[self.0].min_height = value;
        self.invalidate_measure(tree);
    }

    pub fn max_width(self, tree: &WindowTree<State>) -> i16 {
        tree.arena[self.0].max_width
    }

    pub fn set_max_width(self, tree: &mut WindowTree<State>, value: i16) {
        if self == tree.root { return; }
        tree.arena[self.0].max_width = value;
        self.invalidate_measure(tree);
    }

    pub fn max_height(self, tree: &WindowTree<State>) -> i16 {
        tree.arena[self.0].max_height
    }

    pub fn set_max_height(self, tree: &mut WindowTree<State>, value: i16) {
        if self == tree.root { return; }
        tree.arena[self.0].max_height = value;
        self.invalidate_measure(tree);
    }

    pub fn width(self, tree: &WindowTree<State>) -> Option<i16> {
        tree.arena[self.0].width
    }

    pub fn set_width(self, tree: &mut WindowTree<State>, value: Option<i16>) {
        if self == tree.root { return; }
        tree.arena[self.0].width = value;
        self.invalidate_measure(tree);
    }

    pub fn height(self, tree: &WindowTree<State>) -> Option<i16> {
        tree.arena[self.0].height
    }

    pub fn set_height(self, tree: &mut WindowTree<State>, value: Option<i16>) {
        if self == tree.root { return; }
        tree.arena[self.0].height = value;
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

    pub fn tag(self, tree: &WindowTree<State>) -> u16 {
        tree.arena[self.0].tag
    }

    pub fn set_tag(self, tree: &mut WindowTree<State>, value: u16) {
        let old_tag = tree.arena[self.0].tag;
        if old_tag != 0 {
            tree.tagged[usize::from(old_tag - 1)] = None;
        }
        tree.arena[self.0].tag = value;
        if value != 0 {
            if usize::from(value) > tree.tagged.len() {
                tree.tagged.resize(usize::from(value), None);
            }
            assert!(tree.tagged[usize::from(value - 1)].replace(self).is_none());
        }
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
        if let Some(pre_process) = node.pre_process {
            tree.pre_process.remove(pre_process);
        }
        if let Some(post_process) = node.post_process {
            tree.post_process.remove(post_process);
        }
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

pub const COLOR_BACKGROUND: u8 = 10;
pub const COLOR_TEXT: u8 = 11;
pub const COLOR_DISABLED: u8 = 12;
pub const COLOR_HOTKEY: u8 = 13;
pub const COLOR_INPUT_LINE_INVALID: u8 = 14;
pub const COLOR_INPUT_LINE_FOCUSED: u8 = 15;
pub const COLOR_INPUT_LINE_FOCUSED_INVALID: u8 = 16;
pub const COLOR_INPUT_LINE_FOCUSED_DISABLED: u8 = 17;
pub const COLOR_BUTTON_FOCUSED: u8 = 18;
pub const COLOR_BUTTON_FOCUSED_HOTKEY: u8 = 19;
pub const COLOR_BUTTON_FOCUSED_DISABLED: u8 = 20;
pub const COLOR_BUTTON_PRESSED: u8 = 21;
pub const COLOR_FRAME: u8 = 22;

pub const COLORS: RangeInclusive<u8> = 10 ..= 22;

pub const COLOR_IN_FRAME: u8 = 20;

fn root_palette() -> Palette {
    let mut p = Palette::new();

    p.set(COLOR_BACKGROUND, Right((Fg::LightGray, Bg::None)));
    p.set(COLOR_TEXT, Right((Fg::LightGray, Bg::None)));
    p.set(COLOR_DISABLED, Right((Fg::DarkGray, Bg::None)));
    p.set(COLOR_HOTKEY, Right((Fg::White, Bg::None)));
    p.set(COLOR_INPUT_LINE_INVALID, Right((Fg::Red, Bg::None)));
    p.set(COLOR_INPUT_LINE_FOCUSED, Right((Fg::LightGray, Bg::Blue)));
    p.set(COLOR_INPUT_LINE_FOCUSED_DISABLED, Right((Fg::DarkGray, Bg::Blue)));
    p.set(COLOR_INPUT_LINE_FOCUSED_INVALID, Right((Fg::LightGray, Bg::Red)));
    p.set(COLOR_BUTTON_FOCUSED, Right((Fg::LightGray, Bg::Blue)));
    p.set(COLOR_BUTTON_FOCUSED_HOTKEY, Right((Fg::White, Bg::Blue)));
    p.set(COLOR_BUTTON_FOCUSED_DISABLED, Right((Fg::DarkGray, Bg::Blue)));
    p.set(COLOR_BUTTON_PRESSED, Right((Fg::Blue, Bg::None)));
    p.set(COLOR_FRAME, Right((Fg::LightGray, Bg::Black)));

    p.set(COLOR_IN_FRAME + COLOR_BACKGROUND, Right((Fg::LightGray, Bg::Black)));
    p.set(COLOR_IN_FRAME + COLOR_TEXT, Right((Fg::LightGray, Bg::Black)));
    p.set(COLOR_IN_FRAME + COLOR_DISABLED, Right((Fg::DarkGray, Bg::Black)));
    p.set(COLOR_IN_FRAME + COLOR_HOTKEY, Right((Fg::White, Bg::Black)));
    p.set(COLOR_IN_FRAME + COLOR_INPUT_LINE_INVALID, Right((Fg::Red, Bg::Black)));
    p.set(COLOR_IN_FRAME + COLOR_INPUT_LINE_FOCUSED, Right((Fg::LightGray, Bg::Blue)));
    p.set(COLOR_IN_FRAME + COLOR_INPUT_LINE_FOCUSED_DISABLED, Right((Fg::DarkGray, Bg::Blue)));
    p.set(COLOR_IN_FRAME + COLOR_INPUT_LINE_FOCUSED_INVALID, Right((Fg::LightGray, Bg::Red)));
    p.set(COLOR_IN_FRAME + COLOR_BUTTON_FOCUSED, Right((Fg::LightGray, Bg::Blue)));
    p.set(COLOR_IN_FRAME + COLOR_BUTTON_FOCUSED_HOTKEY, Right((Fg::White, Bg::Blue)));
    p.set(COLOR_IN_FRAME + COLOR_BUTTON_FOCUSED_DISABLED, Right((Fg::DarkGray, Bg::Blue)));
    p.set(COLOR_IN_FRAME + COLOR_BUTTON_PRESSED, Right((Fg::Blue, Bg::Black)));
    p.set(COLOR_IN_FRAME + COLOR_FRAME, Right((Fg::LightGray, Bg::Black)));

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

macro_attr! {
    #[derive(Component!(class=PrePostProcessClass))]
    #[derive(Educe)]
    #[educe(Clone)]
    struct PrePostProcess<State: ?Sized>(Window<State>);
}

pub struct WindowTree<'clock, State: ?Sized + 'static> {
    screen: Option<Box<dyn Screen>>,
    arena: Arena<WindowNode<State>>,
    root: Window<State>,
    primary_focused: Option<Window<State>>,
    secondary_focused: Option<Window<State>>,
    next_primary_focused: Option<Option<Window<State>>>,
    next_secondary_focused: Option<Option<Window<State>>>,
    cursor: Option<Point>,
    quit: bool,
    timers: Arena<TimerData<State>>,
    clock: &'clock MonoClock,
    tagged: Vec<Option<Window<State>>>,
    palette: Palette,
    pre_process: Arena<PrePostProcess<State>>,
    post_process: Arena<PrePostProcess<State>>,
}

impl<'clock, State: ?Sized> WindowTree<'clock, State> {
    pub fn new(
        screen: Box<dyn Screen>,
        clock: &'clock MonoClock,
        root_widget: Box<dyn Widget<State>>,
        root_data: Box<dyn WidgetData<State>>,
    ) -> Result<Self, Error> {
        let pre_process = root_widget.pre_process();
        let post_process = root_widget.post_process();
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
            width: None,
            height: None,
            min_width: 0,
            min_height: 0,
            max_width: -1,
            max_height: -1,
            palette: Palette::new(),
            focus_tab: Window(window),
            focus_tab_tag: 0,
            focus_right: Window(window),
            focus_right_tag: 0,
            focus_left: Window(window),
            focus_left_tag: 0,
            focus_up: Window(window),
            focus_up_tag: 0,
            focus_down: Window(window),
            focus_down_tag: 0,
            contains_primary_focus: true,
            tag: 0,
            pre_process: None,
            post_process: None,
            is_enabled: true,
            visibility: Visibility::Visible,
        }, Window(window)));
        let mut tree = WindowTree {
            screen: Some(screen),
            arena,
            root,
            primary_focused: None,
            secondary_focused: None,
            next_primary_focused: None,
            next_secondary_focused: None,
            cursor: None,
            quit: false,
            clock,
            timers: Arena::new(),
            tagged: Vec::new(),
            palette: root_palette(),
            pre_process: Arena::new(),
            post_process: Arena::new(),
        };
        if pre_process {
            let id = tree.pre_process.insert(|id| (PrePostProcess(root), id));
            tree.arena[root.0].pre_process = Some(id);
        }
        if post_process {
            let id = tree.post_process.insert(|id| (PrePostProcess(root), id));
            tree.arena[root.0].post_process = Some(id);
        }
        Ok(tree)
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn palette_mut<T>(&mut self, f: impl FnOnce(&mut Palette) -> T) -> T {
        let res = f(&mut self.palette);
        self.root.invalidate_render(self);
        res
    }

    pub fn window_by_tag(&self, tag: u16) -> Option<Window<State>> {
        if tag != 0 && usize::from(tag) <= self.tagged.len() {
            self.tagged[usize::from(tag - 1)]
        } else {
            None
        }
    }

    pub fn root(&self) -> Window<State> { self.root }

    pub fn primary_focused(&self) -> Option<Window<State>> { self.primary_focused }

    pub fn secondary_focused(&self) -> Option<Window<State>> { self.secondary_focused }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    fn screen(&mut self) -> &mut dyn Screen {
        self.screen.as_mut().expect("WindowTree is in invalid state").as_mut()
    }

    fn render_window(&mut self, window: Window<State>, offset: Vector, render_state: &mut State) {
        if window.visibility(self) != Visibility::Visible {
            return;
        }
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
        if let Some(next_primary_focused) = self.next_primary_focused.take() {
            self.focus_primary(next_primary_focused, state);
        }
        if let Some(next_secondary_focused) = self.next_secondary_focused.take() {
            self.focus_secondary(next_secondary_focused, state);
        }
        let screen = self.screen.as_mut().expect("WindowTree is in invalid state");
        if let Some(screen_Event::Key(n, key)) = screen.update(self.cursor, wait)? {
            for _ in 0 .. n.get() {
                match key {
                    Key::Tab => {
                        if let Some(primary_focused) = self.primary_focused {
                            let focus = primary_focused.actual_focus_tab(self);
                            if self.focus_primary(Some(focus), state) { continue; }
                        }
                    },
                    Key::Left => {
                        if let Some(primary_focused) = self.primary_focused {
                            let focus = primary_focused.actual_focus_left(self);
                            if self.focus_primary(Some(focus), state) { continue; }
                        }
                        if let Some(secondary_focused) = self.secondary_focused {
                            let focus = secondary_focused.actual_focus_left(self);
                            if self.focus_secondary(Some(focus), state) { continue; }
                        }
                    },
                    Key::Right => {
                        if let Some(primary_focused) = self.primary_focused {
                            let focus = primary_focused.actual_focus_right(self);
                            if self.focus_primary(Some(focus), state) { continue; }
                        }
                        if let Some(secondary_focused) = self.secondary_focused {
                            let focus = secondary_focused.actual_focus_right(self);
                            if self.focus_secondary(Some(focus), state) { continue; }
                        }
                    },
                    Key::Up => {
                        if let Some(primary_focused) = self.primary_focused {
                            let focus = primary_focused.actual_focus_up(self);
                            if self.focus_primary(Some(focus), state) { continue; }
                        }
                        if let Some(secondary_focused) = self.secondary_focused {
                            let focus = secondary_focused.actual_focus_up(self);
                            if self.focus_secondary(Some(focus), state) { continue; }
                        }
                    },
                    Key::Down => {
                        if let Some(primary_focused) = self.primary_focused {
                            let focus = primary_focused.actual_focus_down(self);
                            if self.focus_primary(Some(focus), state) { continue; }
                        }
                        if let Some(secondary_focused) = self.secondary_focused {
                            let focus = secondary_focused.actual_focus_down(self);
                            if self.focus_secondary(Some(focus), state) { continue; }
                        }
                    },
                    _ => { },
                }
                let mut handled = false;
                for pre_process in self.pre_process.items().clone().values() {
                    handled = pre_process.0.raise_core(self, Event::PreProcessKey(key), pre_process.0, state);
                    if handled { break; }
                }
                if handled { continue; }
                handled = self.primary_focused.map_or(false, |x|
                    x.raise_priv(self, Event::Key(key), false, state)
                );
                if handled { continue; }
                handled = self.secondary_focused.map_or(false, |x|
                    x.raise_priv(self, Event::Key(key), true, state)
                );
                if handled {
                    self.primary_focused.map(|x|
                        x.raise_priv(self, Event::Cmd(CMD_LOST_ATTENTION), false, state)
                    );
                    continue;
                }
                for post_process in self.post_process.items().clone().values() {
                    handled =
                        post_process.0.raise_core(self, Event::PostProcessKey(key), post_process.0, state);
                    if handled { break; }
                }
            }
        }
        Ok(())
    }

    fn focus_primary(
        &mut self,
        window: Option<Window<State>>,
        state: &mut State
    ) -> bool {
        let old_focused = self.primary_focused;
        if window == old_focused { return false; }
        window.map(|x| x.raise(self, Event::Cmd(CMD_GOT_PRIMARY_FOCUS), state));

        if let Some(mut window) = self.primary_focused {
            loop {
                self.arena[window.0].contains_primary_focus = false;
                if let Some(parent) = window.parent(self) {
                    window = parent;
                } else {
                    break;
                }
            }
        }
        self.primary_focused = window;
        if let Some(mut window) = self.primary_focused {
            loop {
                self.arena[window.0].contains_primary_focus = true;
                if let Some(parent) = window.parent(self) {
                    window = parent;
                } else {
                    break;
                }
            }
        }

        old_focused.map(|x| x.raise(self, Event::Cmd(CMD_LOST_PRIMARY_FOCUS), state));
        true
    }

    fn focus_secondary(
        &mut self,
        window: Option<Window<State>>,
        state: &mut State
    ) -> bool {
        let old_focused = self.secondary_focused;
        if window == old_focused { return false; }
        let focusable = window.map_or(true, |x| self.arena[x.0].widget.secondary_focusable());
        if !focusable { return false; }
        window.map(|x| x.raise(self, Event::Cmd(CMD_GOT_SECONDARY_FOCUS), state));
        self.secondary_focused = window;
        old_focused.map(|x| x.raise(self, Event::Cmd(CMD_LOST_SECONDARY_FOCUS), state));
        true
    }
}
