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
use alloc::string::String;
//use alloc::vec::Vec;
//use components_arena::{Arena, Id, Component};
use core::any::Any;
//use core::fmt::Debug;
use core::ops::Range;
use core::str::FromStr;
//use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
//use macro_attr_2018::macro_attr;
//use phantom_type::PhantomType;
use tuifw_screen_base::{Bg, Error, Fg, Key, Point, Range1d, Rect, Screen, Vector};
use tuifw_window::{Event, RenderPort, Window, WindowTree};
use unicode_width::UnicodeWidthChar;

pub trait RenderPortExt {
    fn fill_bg(&mut self, bg: Bg, fg: Option<Fg>);
    fn h_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg);
    fn v_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg);
    fn tl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn tr_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn bl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn br_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
}

impl RenderPortExt for RenderPort {
    fn fill_bg(&mut self, bg: Bg, fg: Option<Fg>) {
        self.fill(|rp, p| rp.out(p, fg.unwrap_or(Fg::LightGray), bg, if fg.is_some() { "░" } else { " " }));
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

pub struct WidgetData<State: ?Sized> {
    pub widget: Box<dyn Widget<State>>,
    pub data: Box<dyn Any>,
    pub layout: Box<dyn Any>,
}

pub trait Widget<State: ?Sized>: DynClone {
    fn render(
        &self,
        tree: &WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        rp: &mut RenderPort,
        state: &mut State,
    );

    fn measure(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector;

    fn arrange(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector;

    fn update(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        event: Event,
        preview: bool,
        state: &mut State,
    ) -> bool;
}

clone_trait_object!(<State: ?Sized> Widget<State>);

pub fn widget_render<State: ?Sized>(
    tree: &WindowTree<WidgetData<State>, State>,
    window: Window<WidgetData<State>>,
    rp: &mut RenderPort,
    state: &mut State,
) {
    let widget = window.data(tree).widget.clone();
    widget.render(tree, window, rp, state)
}

pub fn widget_measure<State: ?Sized>(
    tree: &mut WindowTree<WidgetData<State>, State>,
    window: Window<WidgetData<State>>,
    available_width: Option<i16>,
    available_height: Option<i16>,
    state: &mut State,
) -> Vector {
    let widget = window.data(tree).widget.clone();
    widget.measure(tree, window, available_width, available_height, state)
}

pub fn widget_arrange<State: ?Sized>(
    tree: &mut WindowTree<WidgetData<State>, State>,
    window: Window<WidgetData<State>>,
    final_inner_bounds: Rect,
    state: &mut State,
) -> Vector {
    let widget = window.data(tree).widget.clone();
    widget.arrange(tree, window, final_inner_bounds, state)
}

pub fn widget_update<State: ?Sized>(
    tree: &mut WindowTree<WidgetData<State>, State>,
    window: Window<WidgetData<State>>,
    event: Event,
    preview: bool,
    state: &mut State,
) -> bool {
    let widget = window.data(tree).widget.clone();
    widget.update(tree, window, event, preview, state)
}

pub struct StackPanel {
    pub vertical: bool,
}

impl StackPanel {
    pub fn widget_data<State: ?Sized>(self) -> WidgetData<State> {
        WidgetData {
            widget: Box::new(StackPanelWidget),
            data: Box::new(self),
            layout: Box::new(()),
        }
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        parent: Window<WidgetData<State>>,
        prev: Option<Window<WidgetData<State>>>
    ) -> Result<Window<WidgetData<State>>, Error> {
        Window::new(tree, self.widget_data(), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<WidgetData<State>, State>, Error> {
        WindowTree::new(screen, widget_render, widget_measure, widget_arrange, widget_update, self.widget_data())
    }
}

#[derive(Clone)]
pub struct StackPanelWidget;

impl<State: ?Sized> Widget<State> for StackPanelWidget {
    fn render(
        &self,
        _tree: &WindowTree<WidgetData<State>, State>,
        _window: Window<WidgetData<State>>,
        _rp: &mut RenderPort,
        _state: &mut State,
    ) { }

    fn measure(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        let vertical = window.data(tree).data.downcast_ref::<StackPanel>().expect("StackPanel").vertical;
        if vertical {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, available_width, None, state);
                    size += Vector { x: 0, y: child.desired_size(tree).y };
                    size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        } else {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, None, available_height, state);
                    size += Vector { x: child.desired_size(tree).x, y: 0 };
                    size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        let vertical = window.data(tree).data.downcast_ref::<StackPanel>().expect("StackPanel").vertical;
        if vertical {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_desired_size = child.desired_size(tree);
                    child.arrange(tree, Rect { tl: pos, size: child_desired_size }, state);
                    pos = pos.offset(Vector { x: 0, y: child_desired_size.y });
                    size += Vector { x: 0, y: child.desired_size(tree).y };
                    size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        } else {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_desired_size = child.desired_size(tree);
                    child.arrange(tree, Rect { tl: pos, size: child_desired_size }, state);
                    pos = pos.offset(Vector { x: child_desired_size.x, y: 0 });
                    size += Vector { x: child.desired_size(tree).x, y: 0 };
                    size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        }
    }

    fn update(
        &self,
        _tree: &mut WindowTree<WidgetData<State>, State>,
        _window: Window<WidgetData<State>>,
        _event: Event,
        _preview: bool,
        _state: &mut State,
    ) -> bool {
        false
    }
}

pub struct StaticText {
    pub color: (Fg, Bg),
    pub text: String,
}

impl StaticText {
    pub fn widget_data<State: ?Sized>(self) -> WidgetData<State> {
        WidgetData {
            widget: Box::new(StaticTextWidget),
            data: Box::new(self),
            layout: Box::new(()),
        }
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        parent: Window<WidgetData<State>>,
        prev: Option<Window<WidgetData<State>>>
    ) -> Result<Window<WidgetData<State>>, Error> {
        Window::new(tree, self.widget_data(), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<WidgetData<State>, State>, Error> {
        WindowTree::new(screen, widget_render, widget_measure, widget_arrange, widget_update, self.widget_data())
    }
}

#[derive(Clone)]
pub struct StaticTextWidget;

impl<State: ?Sized> Widget<State> for StaticTextWidget {
    fn render(
        &self,
        tree: &WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.data(tree).data.downcast_ref::<StaticText>().expect("StaticText");
        rp.out(Point { x: 0, y: 0 }, data.color.0, data.color.1, &data.text);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.data(tree).data.downcast_ref::<StaticText>().expect("StaticText");
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        _final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let data = window.data(tree).data.downcast_ref::<StaticText>().expect("StaticText");
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }

    fn update(
        &self,
        _tree: &mut WindowTree<WidgetData<State>, State>,
        _window: Window<WidgetData<State>>,
        _event: Event,
        _preview: bool,
        _state: &mut State,
    ) -> bool {
        false
    }
}

pub struct Background {
    pub bg: Bg,
    pub fg: Option<Fg>,
}

impl Background {
    pub fn widget_data<State: ?Sized>(self) -> WidgetData<State> {
        WidgetData {
            widget: Box::new(BackgroundWidget),
            data: Box::new(self),
            layout: Box::new(()),
        }
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        parent: Window<WidgetData<State>>,
        prev: Option<Window<WidgetData<State>>>
    ) -> Result<Window<WidgetData<State>>, Error> {
        Window::new(tree, self.widget_data(), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<WidgetData<State>, State>, Error> {
        WindowTree::new(screen, widget_render, widget_measure, widget_arrange, widget_update, self.widget_data())
    }
}

#[derive(Clone)]
pub struct BackgroundWidget;

impl<State: ?Sized> Widget<State> for BackgroundWidget {
    fn render(
        &self,
        tree: &WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.data(tree).data.downcast_ref::<Background>().expect("Background");
        rp.fill_bg(data.bg, data.fg);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        let mut size = Vector::null();
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.measure(tree, available_width, available_height, state);
                size = size.max(child.desired_size(tree));
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        size
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.arrange(tree, final_inner_bounds, state);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        final_inner_bounds.size
    }

    fn update(
        &self,
        _tree: &mut WindowTree<WidgetData<State>, State>,
        _window: Window<WidgetData<State>>,
        _event: Event,
        _preview: bool,
        _state: &mut State,
    ) -> bool {
        false
    }
 }

#[derive(Debug, Clone)]
pub enum InputLineValueRange {
    Any,
    Integer(Range<i64>),
    Float(Range<f64>),
}

pub struct InputLine {
    pub normal_color: (Fg, Bg),
    pub error_color: (Fg, Bg),
    pub value_range: InputLineValueRange,
    pub value: String,
    pub view_start: usize,
    pub cursor_index: usize,
    pub cursor_x: i16,
}

impl InputLine {
    pub fn widget_data<State: ?Sized>(self) -> WidgetData<State> {
        WidgetData {
            widget: Box::new(InputLineWidget),
            data: Box::new(self),
            layout: Box::new(()),
        }
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        parent: Window<WidgetData<State>>,
        prev: Option<Window<WidgetData<State>>>
    ) -> Result<Window<WidgetData<State>>, Error> {
        Window::new(tree, self.widget_data(), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<WidgetData<State>, State>, Error> {
        WindowTree::new(screen, widget_render, widget_measure, widget_arrange, widget_update, self.widget_data())
    }

    pub fn error(&self) -> bool {
        match &self.value_range {
            InputLineValueRange::Any => false,
            InputLineValueRange::Integer(range) => if let Ok(value) = i64::from_str(&self.value) {
                !range.contains(&value)
            } else {
                true
            },
            InputLineValueRange::Float(range) => if let Ok(value) = f64::from_str(&self.value) {
                !range.contains(&value)
            } else {
                true
            },
        }
    }
}

#[derive(Clone)]
pub struct InputLineWidget;

impl<State: ?Sized> Widget<State> for InputLineWidget {
    fn render(
        &self,
        tree: &WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.data(tree).data.downcast_ref::<InputLine>().expect("InputLine");
        let color = if data.error() { data.error_color } else { data.normal_color };
        rp.fill_bg(color.1, None);
        rp.out(Point { x: 0, y: 0 }, color.0, color.1, &data.value[data.view_start ..]);
        if tree.focused() == window {
            rp.cursor(Point { x: data.cursor_x, y: 0 });
        }
    }

    fn measure(
        &self,
        _tree: &mut WindowTree<WidgetData<State>, State>,
        _window: Window<WidgetData<State>>,
        available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        Vector { x: available_width.unwrap_or(1), y: 1 }
    }

    fn arrange(
        &self,
        _tree: &mut WindowTree<WidgetData<State>, State>,
        _window: Window<WidgetData<State>>,
        final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        Vector { x: final_inner_bounds.w(), y: 1 }
    }

    fn update(
        &self,
        tree: &mut WindowTree<WidgetData<State>, State>,
        window: Window<WidgetData<State>>,
        event: Event,
        _preview: bool,
        _state: &mut State,
    ) -> bool {
        match event {
            Event::GotFocus => true,
            Event::Key(n, key) => match key {
                Key::Char(c)  => {
                    let width = window.bounds(tree).w();
                    let data = window.data_mut(tree).data.downcast_mut::<InputLine>().expect("InputLine");
                    for _ in 0 .. n.get() {
                        if data.value.try_reserve(c.len_utf8()).is_ok() {
                            data.value.insert(data.cursor_index, c);
                            data.cursor_index += c.len_utf8();
                            data.cursor_x = data.cursor_x.wrapping_add(
                                if c == '\0' { 0 } else { i16::try_from(c.width().unwrap_or(0)).unwrap() }
                            );
                            while data.cursor_x as u16 >= width as u16 {
                                let c = data.value[data.view_start ..].chars().next().unwrap();
                                data.view_start += c.len_utf8();
                                data.cursor_x = data.cursor_x.wrapping_sub(
                                    if c == '\0' { 0 } else { i16::try_from(c.width().unwrap_or(0)).unwrap() }
                                );
                            }
                        }
                    }
                    window.invalidate(tree);
                    true
                },
                Key::Backspace => {
                    let width = window.bounds(tree).w();
                    let data = window.data_mut(tree).data.downcast_mut::<InputLine>().expect("InputLine");
                    for _ in 0 .. n.get() {
                        if let Some(i) = data.cursor_index.checked_sub(1) {
                            data.cursor_index = i;
                            let c = data.value.remove(data.cursor_index);
                            data.cursor_x = data.cursor_x.wrapping_sub(
                                if c == '\0' { 0 } else { i16::try_from(c.width().unwrap_or(0)).unwrap() }
                            );
                            while data.cursor_x as u16 >= width as u16 {
                                let c = data.value[.. data.view_start].chars().rev().next().unwrap();
                                data.view_start -= c.len_utf8();
                                data.cursor_x = data.cursor_x.wrapping_add(
                                    if c == '\0' { 0 } else { i16::try_from(c.width().unwrap_or(0)).unwrap() }
                                );
                            }
                        }
                    }
                    window.invalidate(tree);
                    true
                },
                _ => false,
            },
            _ => false
        }
    }
}
