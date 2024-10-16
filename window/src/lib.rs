#![feature(never_type)]
#![feature(ptr_metadata)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::assertions_on_constants)]
#![allow(clippy::blocks_in_conditions)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::multiple_bound_locations)]
#![allow(clippy::non_canonical_partial_ord_impl)]
#![allow(clippy::option_map_unit_fn)]
#![allow(clippy::partialeq_to_none)]
#![allow(clippy::type_complexity)]

#![no_std]

extern crate alloc;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use components_arena::{Arena, Component, Id, NewtypeComponentId};
use core::cmp::{max, min};
use core::mem::replace;
use core::ptr::{DynMetadata, Pointee};
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use dynamic_cast::{SupportsInterfaces, dyn_cast_box};
use iter_identify_first_last::IteratorIdentifyFirstLastExt;
use macro_attr_2018::macro_attr;
use timer_no_std::{MonoClock, MonoTime};
use tuifw_screen_base::{Bg, Error, Fg, Key, Point, Rect, Screen, Vector};
use tuifw_screen_base::Event as screen_Event;
use tuifw_screen_base::{HAlign, VAlign, Thickness, Range1d, text_width};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Event {
    Key(Key),
    PreviewKey(Key),
    PreProcessKey(Key),
    PostProcessKey(Key),
    Cmd(u16),
    PreviewCmd(u16),
    LmbDown(Point),
    PreviewLmbDown(Point),
    LmbUp,
    PreviewLmbUp,
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
            Event::LmbDown(_) => false,
            Event::PreviewLmbDown(_) => true,
            Event::LmbUp => false,
            Event::PreviewLmbUp => true,
        }
    }

    fn preview(self) -> Self {
        match self {
            Event::Key(k) => Event::PreviewKey(k),
            Event::Cmd(n) => Event::PreviewCmd(n),
            Event::LmbDown(p) => Event::PreviewLmbDown(p),
            Event::LmbUp => Event::PreviewLmbUp,
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
    bounds: Rect,
    cursor: Option<Point>,
}

impl RenderPort {
    pub fn text(&mut self, p: Point, color: (Fg, Bg), text: &str) {
        let screen_size = self.screen.size();
        let p = p.offset(self.offset);
        if !self.bounds.v_range().contains(p.y) || self.bounds.size.x == 0 { return; }
        if p.y < 0 || p.y >= screen_size.y { return; }
        let row = self.screen.line_invalidated_range(p.y).clone();
        if p.x >= row.end { return; }

        let window_start = self.bounds.l();
        let window_end = self.bounds.r();
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

    pub fn fill_bg(&mut self, color: (Fg, Bg)) {
        self.fill(|rp, p| rp.text(p, color, " "));
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

pub trait Data: Downcast + DynClone { }

impl_downcast!(Data);

clone_trait_object!(Data);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Visibility {
    Visible,
    Hidden,
    Collapsed
}

pub trait Widget: DynClone + SupportsInterfaces {
    #[allow(clippy::wrong_self_convention)]
    #[allow(clippy::new_ret_no_self)]
    fn new(&self) -> Box<dyn WidgetData>;

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    );

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        app: &mut dyn App,
    );

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        available_width: Option<i16>,
        available_height: Option<i16>,
        app: &mut dyn App,
    ) -> Vector;

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        app: &mut dyn App,
    ) -> Vector;

    fn update(
        &self,
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        event_source: Window,
        app: &mut dyn App,
    ) -> bool;

    fn secondary_focusable(&self) -> bool { false }

    fn pre_process(&self) -> bool { false }

    fn post_process(&self) -> bool { false }

    fn bring_into_view(&self, _tree: &mut WindowTree, _window: Window, _rect: Rect) -> bool {
        false
    }
}

clone_trait_object!(Widget);

pub trait WidgetData: Downcast {
    fn drop_widget_data(&mut self, _tree: &mut WindowTree, _app: &mut dyn App) { }
}

impl_downcast!(WidgetData);

pub trait Layout: Downcast + DynClone { }

impl_downcast!(Layout);

clone_trait_object!(Layout);

pub trait App: Downcast { }

impl_downcast!(App);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Focus {
    Primary,
    Secondary
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Color {
    Parent,
    Value((Fg, Bg)),
    Palette(u8)
}

#[derive(Clone)]
pub struct Palette(Vec<Color>);

impl Palette {
    pub fn new() -> Palette {
        Palette(Vec::new())
    }

    pub fn get(&self, i: u8) -> Color {
        self.0.get(usize::from(i)).cloned().unwrap_or(Color::Parent)
    }

    pub fn set(&mut self, i: u8, o: Color) {
        if usize::from(i) >= self.0.len() {
            self.0.reserve(usize::from(i) - self.0.len() + 1);
            for _ in self.0.len() ..= usize::from(i) {
                self.0.push(Color::Parent);
            }
        }
        self.0[usize::from(i)] = o;
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new()
    }
}

pub trait EventHandler: DynClone {
    fn invoke(
        &self,
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        event_source: Window,
        app: &mut dyn App,
    ) -> bool;
}

clone_trait_object!(EventHandler);

macro_attr! {
    #[derive(Component!)]
    struct WindowNode {
        is_template: bool,
        source_index: Option<usize>,
        parent: Option<Window>,
        prev: Window,
        next: Window,
        first_child: Option<Window>,
        widget: Box<dyn Widget>,
        data: Box<dyn WidgetData>,
        layout: Option<Box<dyn Layout>>,
        palette: Palette,
        clip: Option<Rect>,
        measure_size: Option<(Option<i16>, Option<i16>)>,
        desired_size: Vector,
        arrange_size: Option<Vector>,
        arranged_size: Vector,
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
        event_handler: Option<Box<dyn EventHandler>>,
        focus_tab: Window,
        focus_right: Window,
        focus_left: Window,
        focus_up: Window,
        focus_down: Window,
        contains_primary_focus: bool,
        focus_click: Option<Focus>,
        name: String,
        pre_process: Option<Id<PrePostProcess>>,
        post_process: Option<Id<PrePostProcess>>,
        is_enabled: bool,
        visibility: Visibility,
        cloning: Option<Window>,
    }
}

fn offset_from_root(
    window: Option<Window>,
    tree: &WindowTree
) -> Vector {
    let mut offset = Vector::null();
    let Some(mut window) = window else { return offset; };
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
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Window(Id<WindowNode>);
}

impl Window {
    pub fn new(
        tree: &mut WindowTree,
        widget: Box<dyn Widget>,
        parent: Option<Self>,
        prev: Option<Self>,
    ) -> Result<Self, Error> {
        let is_template = if let Some(parent) = parent {
            tree.arena[parent.0].is_template
        } else {
            false
        };
        Self::new_raw(tree, widget, parent, prev, is_template)
    }

    pub fn new_template(
        tree: &mut WindowTree,
        widget: Box<dyn Widget>,
    ) -> Result<Self, Error> {
        Self::new_raw(tree, widget, None, None, true)
    }

    pub fn is_template(self, tree: &WindowTree) -> bool {
        tree.arena[self.0].is_template
    }

    pub fn new_instance(
        self,
        tree: &mut WindowTree,
        parent: Option<Self>,
        prev: Option<Self>
    ) -> Result<Self, Error> {
        assert!(self.is_template(tree), "cannot instantiate non-template window");
        let window = self.begin_cloning(tree, parent, prev)?;
        self.do_cloning(tree);
        self.end_cloning(tree);
        Ok(window)
    }

    fn begin_cloning(
        self,
        tree: &mut WindowTree,
        parent: Option<Self>,
        prev: Option<Self>,
    ) -> Result<Window, Error> {
        let widget = tree.arena[self.0].widget.clone();
        let clone = Self::new(tree, widget, parent, prev)?;
        assert!(tree.arena[self.0].cloning.replace(clone).is_none());
        if let Some(first_child) = self.first_child(tree) {
            let mut child = first_child;
            let mut prev = None;
            loop {
                prev = Some(child.begin_cloning(tree, Some(clone), prev)?);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        Ok(clone)
    }

    fn do_cloning(self, tree: &mut WindowTree) {
        let clone = tree.arena[self.0].cloning.unwrap();
        clone.set_palette(tree, self.palette(tree).clone());
        clone.set_visibility(tree, self.visibility(tree));
        let layout = self.layout_raw(tree).clone();
        clone.layout_raw_mut(tree, |x| replace(x, layout));
        clone.set_is_enabled(tree, self.is_enabled(tree));
        clone.set_h_align(tree, self.h_align(tree));
        clone.set_v_align(tree, self.v_align(tree));
        clone.set_margin(tree, self.margin(tree));
        clone.set_min_width(tree, self.min_width(tree));
        clone.set_min_height(tree, self.min_height(tree));
        clone.set_max_width(tree, self.max_width(tree));
        clone.set_max_height(tree, self.max_height(tree));
        clone.set_width(tree, self.width(tree));
        clone.set_height(tree, self.height(tree));
        let clone_window: Box<dyn Fn(&WindowTree, Window) -> Window> =
            Box::new(|tree: &WindowTree, window: Window| tree.arena[window.0].cloning.unwrap_or(window))
        ;
        clone.set_focus_tab(tree, clone_window(tree, self.focus_tab(tree)));
        clone.set_focus_right(tree, clone_window(tree, self.focus_right(tree)));
        clone.set_focus_left(tree, clone_window(tree, self.focus_left(tree)));
        clone.set_focus_up(tree, clone_window(tree, self.focus_up(tree)));
        clone.set_focus_down(tree, clone_window(tree, self.focus_down(tree)));
        clone.set_focus_click(tree, self.focus_click(tree));
        let widget = tree.arena[self.0].widget.clone();
        widget.clone_data(tree, self, clone, clone_window);
        if let Some(first_child) = self.first_child(tree) {
            let mut child = first_child;
            loop {
                child.do_cloning(tree);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
    }

    fn end_cloning(self, tree: &mut WindowTree) {
        assert!(tree.arena[self.0].cloning.take().is_some());
        if let Some(first_child) = self.first_child(tree) {
            let mut child = first_child;
            loop {
                child.end_cloning(tree);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
    }

    fn new_raw(
        tree: &mut WindowTree,
        widget: Box<dyn Widget>,
        parent: Option<Self>,
        prev: Option<Self>,
        is_template: bool,
    ) -> Result<Self, Error> {
        let data = widget.new();
        let pre_process = if is_template { false } else { widget.pre_process() };
        let post_process = if is_template { false } else { widget.post_process() };
        tree.arena.try_reserve().map_err(|_| Error::Oom)?;
        let window = tree.arena.insert(move |window| {
            (WindowNode {
                is_template,
                source_index: None,
                parent,
                prev: Window(window),
                next: Window(window),
                first_child: None,
                event_handler: None,
                widget,
                data,
                layout: None,
                palette: Palette::new(),
                clip: None,
                measure_size: None,
                desired_size: Vector::null(),
                arrange_size: None,
                arranged_size: Vector::null(),
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
                focus_right: Window(window),
                focus_left: Window(window),
                focus_up: Window(window),
                focus_down: Window(window),
                contains_primary_focus: false,
                focus_click: None,
                name: String::new(),
                pre_process: None,
                post_process: None,
                is_enabled: true,
                visibility: Visibility::Visible,
                cloning: None,
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

    pub fn widget_extension<DynInterface: ?Sized + 'static>(
        self,
        tree: &WindowTree
    ) -> Option<Box<DynInterface>> where
        DynInterface: Pointee<Metadata=DynMetadata<DynInterface>>
    {
        let widget = tree.arena[self.0].widget.clone();
        dyn_cast_box(widget)
    }

    pub fn source_index(self, tree: &WindowTree) -> Option<usize> {
        tree.arena[self.0].source_index
    }

    pub fn set_source_index(self, tree: &mut WindowTree, value: Option<usize>) {
        tree.arena[self.0].source_index = value;
    }

    pub fn set_clip(self, tree: &mut WindowTree, value: Option<Rect>) {
        tree.arena[self.0].clip = value;
        self.invalidate_render(tree);
    }

    pub fn invalidate_measure(self, tree: &mut WindowTree) {
        let mut window = self;
        loop {
            let node = &mut tree.arena[window.0];
            let old_measure_size = node.measure_size.take();
            if old_measure_size.is_none() { break; }
            let Some(parent) = node.parent else { break; };
            window = parent;
        }
    }

    pub fn invalidate_arrange(self, tree: &mut WindowTree) {
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
        tree: &mut WindowTree,
        available_width: Option<i16>,
        available_height: Option<i16>,
        app: &mut dyn App
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
        let measured_size = widget.measure(tree, self, measure_size.0, measure_size.1, app);
        let node = &mut tree.arena[self.0];
        node.desired_size = node.margin.expand_rect_size(measured_size.min(max_size).max(min_size));
        self.invalidate_arrange(tree);
    }

    pub fn arrange(self, tree: &mut WindowTree, final_bounds: Rect, app: &mut dyn App) {
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
        if node.arrange_size != Some(arrange_size) {
            node.arrange_size = Some(arrange_size);
            let widget = node.widget.clone();
            let arranged_size = widget.arrange(
                tree,
                self,
                Rect { tl: Point { x: 0, y: 0 }, size: arrange_size },
                app
            );
            let node = &mut tree.arena[self.0];
            node.arranged_size = arranged_size.min(max_size).max(min_size);
        }
        let node = &mut tree.arena[self.0];
        let arranged_bounds = Thickness::align(
            node.arranged_size,
            margined_bounds.size,
            node.h_align.unwrap_or(HAlign::Left),
            node.v_align.unwrap_or(VAlign::Top)
        ).shrink_rect(margined_bounds).intersect(margined_bounds);
        node.render_bounds = final_bounds;
        self.move_xy_raw(tree, arranged_bounds);
    }

    pub fn set_event_handler(
        self,
        tree: &mut WindowTree,
        handler: Option<Box<dyn EventHandler>>
    ) {
        tree.arena[self.0].event_handler = handler;
    }

    pub fn desired_size(
        self,
        tree: &WindowTree
    ) -> Vector {
        tree.arena[self.0].desired_size
    }

    pub fn render_bounds(
        self,
        tree: &WindowTree
    ) -> Rect {
        tree.arena[self.0].render_bounds
    }

    pub fn inner_bounds(
        self,
        tree: &WindowTree
    ) -> Rect {
        let window_bounds = tree.arena[self.0].window_bounds;
        Rect { tl: Point { x: 0, y: 0 }, size: window_bounds.size }
    }

    pub fn screen_bounds(self, tree: &WindowTree) -> Rect {
        let bounds = tree.arena[self.0].window_bounds;
        let parent = tree.arena[self.0].parent;
        bounds.offset(offset_from_root(parent, tree))
    }

    pub fn inner_point(self, screen_point: Point, tree: &WindowTree) -> Point {
        let offset = screen_point.offset_from(self.screen_bounds(tree).tl);
        Point { x: offset.x, y: offset.y }
    }

    pub fn data<'a, T: WidgetData + 'static>(
        self,
        tree: &'a WindowTree<'_>
    ) -> &'a T {
        tree.arena[self.0].data.downcast_ref::<T>().expect("wrong type")
    }

    pub fn data_mut<'a, T: WidgetData + 'static>(
        self,
        tree: &'a mut WindowTree<'_>
    ) -> &'a mut T {
        tree.arena[self.0].data.downcast_mut::<T>().expect("wrong type")
    }

    pub fn layout_raw<'a>(
        self,
        tree: &'a WindowTree<'_>
    ) -> &'a Option<Box<dyn Layout>> {
        &tree.arena[self.0].layout
    }

    pub fn layout<'a, T: Layout + 'static>(
        self,
        tree: &'a WindowTree<'_>
    ) -> Option<&'a T> {
        tree.arena[self.0].layout.as_ref().and_then(|x| x.downcast_ref::<T>())
    }

    pub fn layout_raw_mut<R>(
        self,
        tree: &mut WindowTree,
        f: impl FnOnce(&mut Option<Box<dyn Layout>>) -> R
    ) -> R {
        let layout = &mut tree.arena[self.0].layout;
        let res = f(layout);
        if let Some(parent) = self.parent(tree) {
            parent.invalidate_measure(tree);
        }
        res
    }

    pub fn layout_mut<T: Layout + Default + 'static, R>(
        self,
        tree: &mut WindowTree,
        f: impl FnOnce(&mut T) -> R
    ) -> R {
        self.layout_raw_mut(tree, |layout| {
            let layout = if let Some(layout) = layout.as_mut().and_then(|x| x.downcast_mut::<T>()) {
                layout
            } else {
                *layout = Some(Box::new(T::default()));
                layout.as_mut().unwrap().downcast_mut::<T>().unwrap()
            };
            f(layout)
        })
    }

    pub fn focus_click(self, tree: &WindowTree) -> Option<Focus> {
        tree.arena[self.0].focus_click
    }

    pub fn set_focus_click(self, tree: &mut WindowTree, value: Option<Focus>) {
        tree.arena[self.0].focus_click = value;
    }

    pub fn focus_tab(self, tree: &WindowTree) -> Self {
        tree.arena[self.0].focus_tab
    }

    pub fn set_focus_tab(self, tree: &mut WindowTree, value: Self) {
        tree.arena[self.0].focus_tab = value;
    }

    pub fn focus_right(self, tree: &WindowTree) -> Self {
        tree.arena[self.0].focus_right
    }

    pub fn set_focus_right(self, tree: &mut WindowTree, value: Self) {
        let old_value = replace(&mut tree.arena[self.0].focus_right, value);
        if old_value != value {
            value.set_focus_left(tree, self);
        }
    }

    pub fn focus_left(self, tree: &WindowTree) -> Self {
        tree.arena[self.0].focus_left
    }

    pub fn set_focus_left(self, tree: &mut WindowTree, value: Self) {
        let old_value = replace(&mut tree.arena[self.0].focus_left, value);
        if old_value != value {
            value.set_focus_right(tree, self);
        }
    }

    pub fn focus_up(self, tree: &WindowTree) -> Self {
        tree.arena[self.0].focus_up
    }

    pub fn set_focus_up(self, tree: &mut WindowTree, value: Self) {
        let old_value = replace(&mut tree.arena[self.0].focus_up, value);
        if old_value != value {
            value.set_focus_down(tree, self);
        }
    }

    pub fn focus_down(self, tree: &WindowTree) -> Self {
        tree.arena[self.0].focus_down
    }

    pub fn set_focus_down(self, tree: &mut WindowTree, value: Self) {
        let old_value = replace(&mut tree.arena[self.0].focus_down, value);
        if old_value != value {
            value.set_focus_up(tree, self);
        }
    }

    pub fn is_primary_focused(self, tree: &WindowTree) -> bool {
        tree.primary_focused == Some(self)
    }

    pub fn is_secondary_focused(self, tree: &WindowTree) -> bool {
        tree.secondary_focused == Some(self)
    }

    pub fn is_focused(self, tree: &WindowTree) -> bool {
        tree.primary_focused == Some(self) || tree.secondary_focused == Some(self)
    }

    pub fn set_focused_primary(self, tree: &mut WindowTree, value: bool) {
        assert!(!self.is_template(tree), "cannot focus template");
        if value {
            tree.next_primary_focused = Some(Some(self));
        } else if
            tree.next_primary_focused == Some(Some(self)) ||
            tree.next_primary_focused == None && tree.primary_focused == Some(self)
        {
            tree.next_primary_focused = Some(None);
        }
    }

    pub fn set_focused_secondary(self, tree: &mut WindowTree, value: bool) {
        assert!(!self.is_template(tree), "cannot focus template");
        if value {
            tree.next_secondary_focused = Some(Some(self));
        } else if
            tree.next_secondary_focused == Some(Some(self)) ||
            tree.next_secondary_focused == None && tree.secondary_focused == Some(self)
        {
            tree.next_secondary_focused = Some(None);
        }
    }

    pub fn palette<'a>(self, tree: &'a WindowTree<'_>) -> &'a Palette {
        &tree.arena[self.0].palette
    }

    pub fn palette_mut<T>(self, tree: &mut WindowTree, f: impl FnOnce(&mut Palette) -> T) -> T {
        let res = f(&mut tree.arena[self.0].palette);
        self.invalidate_render(tree);
        res
    }

    pub fn set_palette(self, tree: &mut WindowTree, value: Palette) {
        self.palette_mut(tree, |palette| replace(palette, value));
    }

    pub fn set_color(self, tree: &mut WindowTree, i: u8, color: (Fg, Bg)) {
        self.palette_mut(tree, |palette| palette.set(i, Color::Value(color)));
    }

    pub fn color(self, tree: &WindowTree, i: u8) -> (Fg, Bg) {
        let mut window = self;
        let mut index = i;
        loop {
            match window.palette(tree).get(index) {
                Color::Parent => {
                    if let Some(parent) = window.parent(tree) {
                        window = parent;
                    } else if let Color::Value(color) = tree.palette().get(index) {
                        break color;
                    } else {
                        break (Fg::Red, Bg::Green);
                    }
                },
                Color::Palette(i) => {
                    index = i;
                },
                Color::Value(c) => break c,
            }
        }
    }

    pub fn is_enabled(self, tree: &WindowTree) -> bool {
        tree.arena[self.0].is_enabled
    }

    pub fn set_is_enabled(self, tree: &mut WindowTree, value: bool) {
        tree.arena[self.0].is_enabled = value;
        self.invalidate_render(tree);
    }

    pub fn actual_is_enabled(self, tree: &WindowTree) -> bool {
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

    pub fn visibility(self, tree: &WindowTree) -> Visibility {
        tree.arena[self.0].visibility
    }

    pub fn set_visibility(self, tree: &mut WindowTree, value: Visibility) {
        let old_value = replace(&mut tree.arena[self.0].visibility, value);
        match (old_value, value) {
            (Visibility::Visible, Visibility::Collapsed) =>
                self.parent(tree).map(|x| x.invalidate_measure(tree)).unwrap_or(()),
            (Visibility::Visible, Visibility::Hidden) =>
                self.invalidate_render(tree),
            (Visibility::Collapsed, Visibility::Visible) =>
                self.parent(tree).map(|x| x.invalidate_measure(tree)).unwrap_or(()),
            (Visibility::Collapsed, Visibility::Hidden) =>
                self.parent(tree).map(|x| x.invalidate_measure(tree)).unwrap_or(()),
            (Visibility::Hidden, Visibility::Visible) =>
                self.invalidate_render(tree),
            (Visibility::Hidden, Visibility::Collapsed) =>
                self.parent(tree).map(|x| x.invalidate_measure(tree)).unwrap_or(()),
            _ => { },
        }
    }

    pub fn parent(
        self,
        tree: &WindowTree
    ) -> Option<Self> {
        tree.arena[self.0].parent
    }

    pub fn first_child(
        self,
        tree: &WindowTree
    ) -> Option<Self> {
        tree.arena[self.0].first_child
    }

    pub fn prev(
        self,
        tree: &WindowTree
    ) -> Self {
        tree.arena[self.0].prev
    }

    pub fn next(
        self,
        tree: &WindowTree
    ) -> Self {
        tree.arena[self.0].next
    }

    pub fn raise(
        self,
        tree: &mut WindowTree,
        event: Event,
        app: &mut dyn App
    ) -> bool {
        self.raise_priv(tree, event, false, app)
    }

    fn raise_priv(
        self,
        tree: &mut WindowTree,
        event: Event,
        secondary: bool,
        app: &mut dyn App
    ) -> bool {
        let mut handled = false;
        self.raise_raw(tree, event.preview(), self, secondary, &mut handled, app);
        if !handled {
            self.raise_raw(tree, event, self, secondary, &mut handled, app);
        }
        handled
    }

    fn raise_raw(
        self,
        tree: &mut WindowTree,
        event: Event,
        event_source: Window,
        secondary: bool,
        handled: &mut bool,
        app: &mut dyn App
    ) {
        if secondary && tree.arena[self.0].contains_primary_focus { return; }
        let parent = self.parent(tree);
        if !*handled && event.is_preview() {
            if let Some(parent) = parent {
                parent.raise_raw(tree, event, event_source, secondary, handled, app);
            }
        }
        if !*handled {
            if matches!(event, Event::LmbDown(_)) {
                match self.focus_click(tree) {
                    Some(Focus::Primary) => self.set_focused_primary(tree, true),
                    Some(Focus::Secondary) => self.set_focused_secondary(tree, true),
                    None => { },
                }
            }
            *handled = self.raise_core(tree, event, event_source, app);
        }
        if !*handled && !event.is_preview() {
            if let Some(parent) = parent {
                parent.raise_raw(tree, event, event_source, secondary, handled, app);
            }
        }
    }

    fn raise_core(
        self,
        tree: &mut WindowTree,
        event: Event,
        event_source: Window,
        app: &mut dyn App
    ) -> bool {
        let node = &tree.arena[self.0];
        let widget = node.widget.clone();
        let event_handler = node.event_handler.clone();
        let mut handled = widget.update(tree, self, event, event_source, app);
        if !handled {
            if let Some(event_handler) = event_handler {
                handled = event_handler.invoke(tree, self, event, event_source, app);
            }
        }
        handled
    }

    fn move_xy_raw(
        self,
        tree: &mut WindowTree,
        window_bounds: Rect
    ) {
        let parent = tree.arena[self.0].parent;
        let screen_bounds = window_bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
        let window_bounds = replace(&mut tree.arena[self.0].window_bounds, window_bounds);
        let screen_bounds = window_bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }

    pub fn h_align(self, tree: &WindowTree) -> Option<HAlign> {
        tree.arena[self.0].h_align
    }

    pub fn set_h_align(self, tree: &mut WindowTree, value: Option<HAlign>) {
        tree.arena[self.0].h_align = value;
        self.invalidate_measure(tree);
    }

    pub fn v_align(self, tree: &WindowTree) -> Option<VAlign> {
        tree.arena[self.0].v_align
    }

    pub fn set_v_align(self, tree: &mut WindowTree, value: Option<VAlign>) {
        tree.arena[self.0].v_align = value;
        self.invalidate_measure(tree);
    }

    pub fn margin(self, tree: &WindowTree) -> Thickness {
        tree.arena[self.0].margin
    }

    pub fn set_margin(self, tree: &mut WindowTree, value: Thickness) {
        tree.arena[self.0].margin = value;
        self.invalidate_measure(tree);
    }

    pub fn min_width(self, tree: &WindowTree) -> i16 {
        tree.arena[self.0].min_width
    }

    pub fn set_min_width(self, tree: &mut WindowTree, value: i16) {
        tree.arena[self.0].min_width = value;
        self.invalidate_measure(tree);
    }

    pub fn min_height(self, tree: &WindowTree) -> i16 {
        tree.arena[self.0].min_height
    }

    pub fn set_min_height(self, tree: &mut WindowTree, value: i16) {
        tree.arena[self.0].min_height = value;
        self.invalidate_measure(tree);
    }

    pub fn max_width(self, tree: &WindowTree) -> i16 {
        tree.arena[self.0].max_width
    }

    pub fn set_max_width(self, tree: &mut WindowTree, value: i16) {
        tree.arena[self.0].max_width = value;
        self.invalidate_measure(tree);
    }

    pub fn max_height(self, tree: &WindowTree) -> i16 {
        tree.arena[self.0].max_height
    }

    pub fn set_max_height(self, tree: &mut WindowTree, value: i16) {
        tree.arena[self.0].max_height = value;
        self.invalidate_measure(tree);
    }

    pub fn width(self, tree: &WindowTree) -> Option<i16> {
        tree.arena[self.0].width
    }

    pub fn set_width(self, tree: &mut WindowTree, value: Option<i16>) {
        tree.arena[self.0].width = value;
        self.invalidate_measure(tree);
    }

    pub fn height(self, tree: &WindowTree) -> Option<i16> {
        tree.arena[self.0].height
    }

    pub fn set_height(self, tree: &mut WindowTree, value: Option<i16>) {
        tree.arena[self.0].height = value;
        self.invalidate_measure(tree);
    }

    pub fn move_z(
        self,
        tree: &mut WindowTree,
        prev: Option<Self>
    ) {
        let parent = self.detach(tree);
        self.attach(tree, parent, prev);
        let bounds = tree.arena[self.0].window_bounds;
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }

    pub fn name<'a>(self, tree: &'a WindowTree) -> &'a String {
        &tree.arena[self.0].name
    }

    pub fn name_mut<T>(self, tree: &mut WindowTree, f: impl FnOnce(&mut String) -> T) -> T {
        f(&mut tree.arena[self.0].name)
    }

    pub fn set_name<'a>(self, tree: &mut WindowTree, value: impl Into<Cow<'a, str>>) {
        self.name_mut(tree, |name| replace(name, value.into().into_owned()));
    }

    fn detach(
        self,
        tree: &mut WindowTree
    ) -> Option<Self> {
        let node = &mut tree.arena[self.0];
        let parent = node.parent.take();
        let prev = replace(&mut node.prev, self);
        let next = replace(&mut node.next, self);
        tree.arena[prev.0].next = next;
        tree.arena[next.0].prev = prev;
        if let Some(parent) = parent {
            let parent_node = &mut tree.arena[parent.0];
            if parent_node.first_child.unwrap() == self {
                parent_node.first_child = if next == self { None } else { Some(next) };
            }
            parent.invalidate_measure(tree);
        } else {
            if tree.first_child.unwrap() == self {
                tree.first_child = if next == self { None } else { Some(next) };
            }
        }
        parent
    }

    fn attach(
        self,
        tree: &mut WindowTree,
        parent: Option<Self>,
        prev: Option<Self>
    ) {
        let (prev, next) = if let Some(prev) = prev {
            assert_eq!(tree.arena[prev.0].parent, parent);
            let next = replace(&mut tree.arena[prev.0].next, self);
            tree.arena[next.0].prev = self;
            (prev, next)
        } else if let Some(parent) = parent {
            let parent_node = &mut tree.arena[parent.0];
            let next = parent_node.first_child.replace(self).unwrap_or(self);
            let prev = replace(&mut tree.arena[next.0].prev, self);
            tree.arena[prev.0].next = self;
            (prev, next)
        } else {
            let next = tree.first_child.replace(self).unwrap_or(self);
            let prev = replace(&mut tree.arena[next.0].prev, self);
            tree.arena[prev.0].next = self;
            (prev, next)
        };
        let node = &mut tree.arena[self.0];
        node.parent = parent;
        node.prev = prev;
        node.next = next;
        parent.map(|x| x.invalidate_measure(tree));
    }

    pub fn drop_window(
        self,
        tree: &mut WindowTree,
        app: &mut dyn App,
    ) {
        self.cleanup(tree, app);
        let parent = self.detach(tree);
        let mut node = tree.arena.remove(self.0);
        if let Some(pre_process) = node.pre_process {
            tree.pre_process.remove(pre_process);
        }
        if let Some(post_process) = node.post_process {
            tree.post_process.remove(post_process);
        }
        node.data.drop_widget_data(tree, app);
        let screen_bounds = node.window_bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
        Self::drop_node_tree(node.first_child, tree, app);
    }

    fn cleanup(
        self,
        tree: &mut WindowTree,
        app: &mut dyn App,
    ) {
        if tree.click == Some(self) {
            tree.click = None;
        }
        if tree.next_primary_focused == Some(Some(self)) {
            tree.next_primary_focused = Some(None);
        }
        if tree.primary_focused == Some(self) {
            tree.focus_primary_raw(None, app);
        }
        if tree.next_secondary_focused == Some(Some(self)) {
            tree.next_secondary_focused = Some(None);
        }
        if tree.secondary_focused == Some(self) {
            tree.focus_secondary_raw(None, app);
        }
        if let Some(first_child) = self.first_child(tree) {
            let mut child = first_child;
            loop {
                child.cleanup(tree, app);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
    }

    fn drop_node_tree(
        first_child: Option<Window>,
        tree: &mut WindowTree,
        app: &mut dyn App,
    ) {
        if let Some(first_child) = first_child {
            let mut child = first_child;
            loop {
                let mut child_node = tree.arena.remove(child.0);
                if let Some(pre_process) = child_node.pre_process {
                    tree.pre_process.remove(pre_process);
                }
                if let Some(post_process) = child_node.post_process {
                    tree.post_process.remove(post_process);
                }
                child_node.data.drop_widget_data(tree, app);
                child = child_node.next;
                Self::drop_node_tree(child_node.first_child, tree, app);
                if child == first_child { break; }
            }
        }
    }

    pub fn invalidate_rect(
        self,
        tree: &mut WindowTree,
        rect: Rect
    ) {
        let bounds = tree.arena[self.0].window_bounds;
        let rect = rect.offset(bounds.tl.offset_from(Point { x: 0, y: 0 })).intersect(bounds);
        let parent = tree.arena[self.0].parent;
        let screen_rect = rect.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_rect);
    }
 
    pub fn invalidate_render(
        self,
        tree: &mut WindowTree
    ) {
        let bounds = tree.arena[self.0].window_bounds;
        let parent = tree.arena[self.0].parent;
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }

    pub fn bring_into_view(self, tree: &mut WindowTree) {
        let node = &tree.arena[self.0];
        if let Some(mut window) = node.parent {
            let mut rect = node.window_bounds;
            loop {
                let node = &tree.arena[window.0];
                let widget = node.widget.clone();
                let parent = node.parent;
                if widget.bring_into_view(tree, window, rect) {
                    if let Some(parent) = parent {
                        rect = tree.arena[window.0].window_bounds;
                        window = parent;
                    } else {
                        break;
                    }
                } else {
                    if let Some(parent) = parent {
                        let offset = tree.arena[window.0].window_bounds.tl.offset_from(Point { x: 0, y: 0 });
                        rect = rect.offset(offset);
                        window = parent;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn hit_test(self, point: Point, tree: &WindowTree) -> Option<Window> {
        let bounds = tree.arena[self.0].window_bounds;
        if !bounds.contains(point) { return None; }
        let offset = bounds.tl.offset_from(Point { x: 0, y: 0 });
        let point = point.offset(-offset);
        if let Some(first_child) = self.first_child(tree) {
            let mut child = first_child;
            loop {
                let child_hit_test = child.hit_test(point, tree);
                if child_hit_test.is_some() { return child_hit_test; }
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        Some(self)
    }
}

const FPS: u16 = 40;

pub const COLOR_DISABLED: u8 = 10;
pub const COLOR_HOTKEY: u8 = 11;
pub const COLOR_BACKGROUND: u8 = 12;
pub const COLOR_LABEL: u8 = 13;
pub const COLOR_INPUT_LINE: u8 = 14;
pub const COLOR_INPUT_LINE_INVALID: u8 = 15;
pub const COLOR_INPUT_LINE_FOCUSED: u8 = 16;
pub const COLOR_INPUT_LINE_FOCUSED_INVALID: u8 = 17;
pub const COLOR_INPUT_LINE_FOCUSED_DISABLED: u8 = 18;
pub const COLOR_BUTTON: u8 = 19;
pub const COLOR_BUTTON_FOCUSED: u8 = 20;
pub const COLOR_BUTTON_FOCUSED_HOTKEY: u8 = 21;
pub const COLOR_BUTTON_FOCUSED_DISABLED: u8 = 22;
pub const COLOR_BUTTON_PRESSED: u8 = 23;
pub const COLOR_FRAME: u8 = 24;

fn root_palette() -> Palette {
    let mut p = Palette::new();

    p.set(COLOR_DISABLED, Color::Value((Fg::DarkGray, Bg::None)));
    p.set(COLOR_HOTKEY, Color::Value((Fg::White, Bg::None)));
    p.set(COLOR_BACKGROUND, Color::Value((Fg::LightGray, Bg::None)));
    p.set(COLOR_LABEL, Color::Value((Fg::LightGray, Bg::None)));
    p.set(COLOR_INPUT_LINE, Color::Value((Fg::LightGray, Bg::None)));
    p.set(COLOR_INPUT_LINE_INVALID, Color::Value((Fg::Red, Bg::None)));
    p.set(COLOR_INPUT_LINE_FOCUSED, Color::Value((Fg::LightGray, Bg::Blue)));
    p.set(COLOR_INPUT_LINE_FOCUSED_DISABLED, Color::Value((Fg::DarkGray, Bg::Blue)));
    p.set(COLOR_INPUT_LINE_FOCUSED_INVALID, Color::Value((Fg::LightGray, Bg::Red)));
    p.set(COLOR_BUTTON, Color::Value((Fg::LightGray, Bg::None)));
    p.set(COLOR_BUTTON_FOCUSED, Color::Value((Fg::LightGray, Bg::Blue)));
    p.set(COLOR_BUTTON_FOCUSED_HOTKEY, Color::Value((Fg::White, Bg::Blue)));
    p.set(COLOR_BUTTON_FOCUSED_DISABLED, Color::Value((Fg::DarkGray, Bg::Blue)));
    p.set(COLOR_BUTTON_PRESSED, Color::Value((Fg::Blue, Bg::None)));
    p.set(COLOR_FRAME, Color::Value((Fg::LightGray, Bg::None)));

    p
}

macro_attr! {
    #[derive(Component!)]
    struct TimerData {
        start: MonoTime,
        span_ms: u16,
        alarm: Box<dyn FnOnce(&mut WindowTree, &mut dyn App)>,
    }
}

macro_attr! {
    #[derive(NewtypeComponentId!)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Timer(Id<TimerData>);
}

impl Timer {
    pub fn new(
        tree: &mut WindowTree,
        span_ms: u16,
        alarm: Box<dyn FnOnce(&mut WindowTree, &mut dyn App)>
    ) -> Self {
        let start = tree.clock.time();
        tree.timers.insert(move |id| (TimerData {
            start,
            span_ms,
            alarm
        }, Timer(id)))
    }

    pub fn drop_timer(self, tree: &mut WindowTree) {
        tree.timers.remove(self.0);
    }
}

macro_attr! {
    #[derive(Component!)]
    #[derive(Clone)]
    struct PrePostProcess(Window);
}

pub struct WindowTree<'clock> {
    screen: Option<Box<dyn Screen>>,
    arena: Arena<WindowNode>,
    first_child: Option<Window>,
    primary_focused: Option<Window>,
    secondary_focused: Option<Window>,
    next_primary_focused: Option<Option<Window>>,
    next_secondary_focused: Option<Option<Window>>,
    click: Option<Window>,
    cursor: Option<Point>,
    quit: bool,
    timers: Arena<TimerData>,
    clock: &'clock MonoClock,
    palette: Palette,
    pre_process: Arena<PrePostProcess>,
    post_process: Arena<PrePostProcess>,
}

impl<'clock> WindowTree<'clock> {
    pub fn new(
        screen: Box<dyn Screen>,
        clock: &'clock MonoClock,
    ) -> Result<Self, Error> {
        Ok(WindowTree {
            screen: Some(screen),
            arena: Arena::new(),
            first_child: None,
            primary_focused: None,
            secondary_focused: None,
            next_primary_focused: None,
            next_secondary_focused: None,
            click: None,
            cursor: None,
            quit: false,
            clock,
            timers: Arena::new(),
            palette: root_palette(),
            pre_process: Arena::new(),
            post_process: Arena::new(),
        })
    }

    pub fn palette(&self) -> &Palette {
        &self.palette
    }

    pub fn palette_mut<T>(&mut self, f: impl FnOnce(&mut Palette) -> T) -> T {
        let res = f(&mut self.palette);
        if let Some(first_child) = self.first_child {
            let mut child = first_child;
            loop {
                child.invalidate_render(self);
                child = child.next(self);
                if child == first_child { break; }
            }
        }
        res
    }

    pub fn first_child(&self) -> Option<Window> { self.first_child }

    pub fn primary_focused(&self) -> Option<Window> { self.primary_focused }

    pub fn secondary_focused(&self) -> Option<Window> { self.secondary_focused }

    pub fn quit(&mut self) {
        self.quit = true;
    }

    fn screen(&mut self) -> &mut dyn Screen {
        self.screen.as_mut().expect("WindowTree is in invalid state").as_mut()
    }

    fn render_window(&mut self, window: Window, offset: Vector, clip_bounds: Rect, app: &mut dyn App) {
        if window.visibility(self) != Visibility::Visible {
            return;
        }
        let bounds = self.arena[window.0].window_bounds.offset(offset);
        let offset = bounds.tl.offset_from(Point { x: 0, y: 0 });
        let clip = self.arena[window.0].clip.map(|x| x.offset(offset));
        let mut clipped_bounds = bounds.intersect(clip_bounds);
        if let Some(clip) = clip {
            clipped_bounds = clipped_bounds.intersect(clip);
        }
        let screen = self.screen();
        if !rect_invalidated(screen, clipped_bounds) { return; }
        let screen = self.screen.take().expect("WindowTree is in invalid state");
        let mut port = RenderPort {
            screen,
            cursor: self.cursor,
            offset,
            bounds: clipped_bounds,
        };
        let widget = self.arena[window.0].widget.clone();
        widget.render(self, window, &mut port, app);
        self.screen.replace(port.screen);
        self.cursor = port.cursor;
        if let Some(first_child) = self.arena[window.0].first_child {
            let mut child = first_child;
            loop {
                self.render_window(child, offset, clipped_bounds, app);
                child = self.arena[child.0].next;
                if child == first_child { break; }
            }
        }
    }

    pub fn run(
        &mut self,
        app: &mut dyn App,
        mut action: Option<Box<dyn FnMut(&mut WindowTree, &mut dyn App) -> Result<(), Error>>>
    ) -> Result<(), Error> {
        let mut time = self.clock.time();
        while !self.quit {
            if let Some(next_primary_focused) = self.next_primary_focused.take() {
                self.focus_primary_raw(next_primary_focused, app);
            }
            if let Some(next_secondary_focused) = self.next_secondary_focused.take() {
                self.focus_secondary_raw(next_secondary_focused, app);
            }
            let no_timers = self.timers.items().is_empty() && action.is_none();
            let timers_time = self.clock.time();
            loop {
                let timer = self.timers.items().iter()
                    .find(|(_, data)| timers_time.delta_ms_u16(data.start).unwrap_or(u16::MAX) >= data.span_ms)
                    .map(|(id, _)| id)
                ;
                if let Some(timer) = timer {
                    let alarm = self.timers.remove(timer).alarm;
                    alarm(self, app);
                } else {
                    break;
                }
            }
            action.as_mut().map_or(Ok(()), |x| x(self, app))?;
            if no_timers {
                self.update(true, app)?;
            } else {
                let ms = time.split_ms_u16(self.clock).unwrap_or(u16::MAX);
                self.update(false, app)?;
                assert!(FPS != 0 && u16::MAX / FPS > 8);
                self.clock.sleep_ms_u16((1000 / FPS).saturating_sub(ms));
            }
        }
        Ok(())
    }

    fn update(&mut self, wait: bool, app: &mut dyn App) -> Result<(), Error> {
        let screen = self.screen.as_mut().expect("WindowTree is in invalid state");
        let screen_size = screen.size();
        if let Some(first_child) = self.first_child {
            let mut child = first_child;
            loop {
                child.measure(self, Some(screen_size.x), Some(screen_size.y), app);
                child.arrange(self, Rect { tl: Point { x: 0, y: 0 }, size: screen_size }, app);
                child = child.next(self);
                if child == first_child { break; }
            }
        }
        if let Some(cursor) = self.cursor {
            let screen = self.screen();
            if rect_invalidated(screen, Rect { tl: cursor, size: Vector { x: 1, y: 1 } }) {
                self.cursor = None;
            }
        }
        if let Some(first_child) = self.first_child {
            let mut child = first_child;
            loop {
                let bounds = self.arena[child.0].window_bounds;
                let offset = bounds.tl.offset_from(Point { x: 0, y: 0 });
                self.render_window(child, offset, bounds, app);
                child = child.next(self);
                if child == first_child { break; }
            }
        }
        let screen = self.screen.as_mut().expect("WindowTree is in invalid state");
        match screen.update(self.cursor, wait)? {
            Some(screen_Event::Key(n, key)) => {
                for _ in 0 .. n.get() {
                    let mut handled = false;
                    for pre_process in self.pre_process.items().clone().values() {
                        handled = pre_process.0.raise_core(self, Event::PreProcessKey(key), pre_process.0, app);
                        if handled { break; }
                    }
                    if handled { continue; }
                    handled = self.primary_focused.map_or(false, |x|
                        x.raise_priv(self, Event::Key(key), false, app)
                    );
                    if handled { continue; }
                    handled = self.secondary_focused.map_or(false, |x|
                        x.raise_priv(self, Event::Key(key), true, app)
                    );
                    if handled {
                        self.primary_focused.map(|x|
                            x.raise_priv(self, Event::Cmd(CMD_LOST_ATTENTION), false, app)
                        );
                        continue;
                    }
                    for post_process in self.post_process.items().clone().values() {
                        handled =
                            post_process.0.raise_core(self, Event::PostProcessKey(key), post_process.0, app);
                        if handled { break; }
                    }
                    if handled { continue; }
                    match key {
                        Key::Tab => {
                            if let Some(primary_focused) = self.primary_focused {
                                let focus = primary_focused.focus_tab(self);
                                if self.focus_primary(focus) { continue; }
                            }
                        },
                        Key::Left => {
                            if let Some(primary_focused) = self.primary_focused {
                                let focus = primary_focused.focus_left(self);
                                if self.focus_primary(focus) { continue; }
                            }
                            if let Some(secondary_focused) = self.secondary_focused {
                                let focus = secondary_focused.focus_left(self);
                                if self.focus_secondary(focus) { continue; }
                            }
                        },
                        Key::Right => {
                            if let Some(primary_focused) = self.primary_focused {
                                let focus = primary_focused.focus_right(self);
                                if self.focus_primary(focus) { continue; }
                            }
                            if let Some(secondary_focused) = self.secondary_focused {
                                let focus = secondary_focused.focus_right(self);
                                if self.focus_secondary(focus) { continue; }
                            }
                        },
                        Key::Up => {
                            if let Some(primary_focused) = self.primary_focused {
                                let focus = primary_focused.focus_up(self);
                                if self.focus_primary(focus) { continue; }
                            }
                            if let Some(secondary_focused) = self.secondary_focused {
                                let focus = secondary_focused.focus_up(self);
                                if self.focus_secondary(focus) { continue; }
                            }
                        },
                        Key::Down => {
                            if let Some(primary_focused) = self.primary_focused {
                                let focus = primary_focused.focus_down(self);
                                if self.focus_primary(focus) { continue; }
                            }
                            if let Some(secondary_focused) = self.secondary_focused {
                                let focus = secondary_focused.focus_down(self);
                                if self.focus_secondary(focus) { continue; }
                            }
                        },
                        _ => { },
                    }
                }
            },
            Some(screen_Event::LmbUp(point)) => {
                if let Some(window) = self.click.take() {
                    window.raise_priv(self, Event::LmbUp, false, app);
                } else if let Some(window) = self.hit_test(point) {
                    window.raise_priv(self, Event::LmbDown(point), false, app);
                    window.raise_priv(self, Event::LmbUp, false, app);
                }
            },
            Some(screen_Event::LmbDown(point)) => {
                if let Some(window) = self.hit_test(point) {
                    self.click = Some(window);
                    window.raise_priv(self, Event::LmbDown(point), false, app);
                }
            },
            _ => { }
        }
        Ok(())
    }

    fn hit_test(&self, point: Point) -> Option<Window> {
        if let Some(first_child) = self.first_child {
            let mut child = first_child;
            loop {
                let hit_test = child.hit_test(point, self);
                if hit_test.is_some() { return hit_test; }
                child = child.next(self);
                if child == first_child { break; }
            }
        }
        None
    }

    fn focus_primary(
        &mut self,
        window: Window,
    ) -> bool {
        if Some(window) == self.primary_focused { return false; }
        window.set_focused_primary(self, true);
        true
    }

    fn focus_primary_raw(
        &mut self,
        window: Option<Window>,
        app: &mut dyn App
    ) {
        let old_focused = self.primary_focused;
        if window == old_focused { return; }
        window.map(|x| x.raise(self, Event::Cmd(CMD_GOT_PRIMARY_FOCUS), app));

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

        old_focused.map(|x| x.raise(self, Event::Cmd(CMD_LOST_PRIMARY_FOCUS), app));
        window.map(|x| x.bring_into_view(self));
    }

    fn focus_secondary(
        &mut self,
        window: Window,
    ) -> bool {
        if Some(window) == self.secondary_focused { return false; }
        let focusable = self.arena[window.0].widget.secondary_focusable();
        if !focusable { return false; }
        window.set_focused_secondary(self, true);
        true
    }

    fn focus_secondary_raw(
        &mut self,
        window: Option<Window>,
        app: &mut dyn App
    ) {
        let old_focused = self.secondary_focused;
        if window == old_focused { return; }
        let focusable = window.map_or(true, |x| self.arena[x.0].widget.secondary_focusable());
        if !focusable { return; }
        window.map(|x| x.raise(self, Event::Cmd(CMD_GOT_SECONDARY_FOCUS), app));
        self.secondary_focused = window;
        old_focused.map(|x| x.raise(self, Event::Cmd(CMD_LOST_SECONDARY_FOCUS), app));
        window.map(|x| x.bring_into_view(self));
    }
}
