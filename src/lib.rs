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

mod stack_panel;
pub use stack_panel::*;

mod static_text;
pub use static_text::*;

mod background;
pub use background::*;

use alloc::boxed::Box;
use alloc::string::String;
use core::ops::Range;
use core::str::FromStr;
use tuifw_screen_base::{Bg, Error, Fg, Key, Point, Rect, Screen, Vector};
use tuifw_window::{Event, RenderPort, Widget, Window, WindowTree};
use unicode_width::UnicodeWidthChar;

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
        rp.fill_bg(color.1);
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
