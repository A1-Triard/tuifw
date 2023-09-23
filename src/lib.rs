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
use core::ops::Range;
use core::str::FromStr;
use tuifw_screen_base::{Bg, Error, Fg, Key, Point, Rect, Screen, Vector};
use tuifw_window::{Event, RenderPort, Widget, Window, WindowTree};
use unicode_width::UnicodeWidthChar;

pub struct StackPanel {
    pub vertical: bool,
}

impl StackPanel {
    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        Window::new(tree, Box::new(StackPanelWidget), Box::new(self), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<State>, Error> {
        WindowTree::new(screen, Box::new(StackPanelWidget), Box::new(self))
    }
}

#[derive(Clone)]
pub struct StackPanelWidget;

impl<State: ?Sized> Widget<State> for StackPanelWidget {
    fn render(
        &self,
        _tree: &WindowTree<State>,
        _window: Window<State>,
        _rp: &mut RenderPort,
        _state: &mut State,
    ) { }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        let vertical = window.data::<StackPanel>(tree).vertical;
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
        tree: &mut WindowTree<State>,
        window: Window<State>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        let vertical = window.data::<StackPanel>(tree).vertical;
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
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
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
    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        Window::new(tree, Box::new(StaticTextWidget), Box::new(self), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<State>, Error> {
        WindowTree::new(screen, Box::new(StaticTextWidget), Box::new(self))
    }
}

#[derive(Clone)]
pub struct StaticTextWidget;

impl<State: ?Sized> Widget<State> for StaticTextWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.data::<StaticText>(tree);
        rp.out(Point { x: 0, y: 0 }, data.color.0, data.color.1, &data.text);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<StaticText>(tree);
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<StaticText>(tree);
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }

    fn update(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
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
    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        Window::new(tree, Box::new(BackgroundWidget), Box::new(self), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<State>, Error> {
        WindowTree::new(screen, Box::new(BackgroundWidget), Box::new(self))
    }
}

#[derive(Clone)]
pub struct BackgroundWidget;

impl<State: ?Sized> Widget<State> for BackgroundWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.data::<Background>(tree);
        rp.fill_bg(data.bg, data.fg);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
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
        tree: &mut WindowTree<State>,
        window: Window<State>,
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
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
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
    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        Window::new(tree, Box::new(InputLineWidget), Box::new(self), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<State>, Error> {
        WindowTree::new(screen, Box::new(InputLineWidget), Box::new(self))
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
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.data::<InputLine>(tree);
        let color = if data.error() { data.error_color } else { data.normal_color };
        rp.fill_bg(color.1, None);
        rp.out(Point { x: 0, y: 0 }, color.0, color.1, &data.value[data.view_start ..]);
        if tree.focused() == window {
            rp.cursor(Point { x: data.cursor_x, y: 0 });
        }
    }

    fn measure(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        Vector { x: available_width.unwrap_or(1), y: 1 }
    }

    fn arrange(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        Vector { x: final_inner_bounds.w(), y: 1 }
    }

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        _preview: bool,
        _state: &mut State,
    ) -> bool {
        match event {
            Event::GotFocus => true,
            Event::Key(n, key) => match key {
                Key::Char(c)  => {
                    let width = window.bounds(tree).w();
                    let data = window.data_mut::<InputLine>(tree);
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
                    let data = window.data_mut::<InputLine>(tree);
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
