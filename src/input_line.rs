use crate::{prop_obj_render, prop_string_render, widget};
use alloc::boxed::Box;
use alloc::string::String;
use core::ops::Range;
use core::str::FromStr;
use either::Left;
use tuifw_screen_base::{Key, Point, Rect, Vector, char_width, text_width};
use tuifw_screen_base::{Thickness};
use tuifw_window::{Event, RenderPort, Timer, Widget, WidgetData, Window, WindowTree};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS, CMD_LOST_ATTENTION};

pub const CMD_INPUT_LINE_IS_VALID_CHANGED: u16 = 110;

pub trait Validator {
    fn is_numeric(&self) -> bool;

    fn is_valid(&self, editing: bool, text: &str) -> bool;
}

pub struct IntValidator {
    pub min: i32,
    pub max: i32,
}

impl Validator for IntValidator {
    fn is_numeric(&self) -> bool { true }

    fn is_valid(&self, editing: bool, text: &str) -> bool {
        if editing && text.is_empty() { return true; }
        if let Ok(value) = i32::from_str(text) {
            (self.min ..= self.max).contains(&value)
        } else {
            false
        }
    }
}

pub struct FloatValidator {
    pub min: f64,
    pub max: f64,
}

impl Validator for FloatValidator {
    fn is_numeric(&self) -> bool { true }

    fn is_valid(&self, editing: bool, text: &str) -> bool {
        if editing && text.is_empty() { return true; }
        let text = if editing && (text.ends_with('e') || text.ends_with('E')) {
            let text = &text[.. text.len() - 1];
            if text.contains(|c| c == 'e' || c == 'E') { return false; }
            text
        } else {
            text
        };
        if let Ok(value) = f64::from_str(text) {
            (self.min ..= self.max).contains(&value)
        } else {
            false
        }
    }
}

pub struct InputLine {
    validator: Option<Box<dyn Validator>>,
    text: String,
    is_valid: bool,
    editing: bool,
    view_padding: i16,
    view: Range<usize>,
    cursor: usize,
    width: i16,
    is_valid_timer: Option<Timer>,
}

impl<State: ?Sized> WidgetData<State> for InputLine {
    fn drop_widget_data(&mut self, tree: &mut WindowTree<State>, _state: &mut State) {
        if let Some(timer) = self.is_valid_timer.take() {
            timer.drop_timer(tree);
        }
    }
}

impl InputLine {
    pub fn new() -> Self {
        InputLine {
            validator: None,
            text: String::new(),
            is_valid: true,
            editing: false,
            view_padding: 0,
            view: 0 .. 0,
            cursor: 0,
            width: 0,
            is_valid_timer: None,
        }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(12));
            palette.set(1, Left(13));
            palette.set(2, Left(15));
            palette.set(3, Left(16));
            palette.set(4, Left(17));
        });
    }

    widget!(InputLineWidget; init_palette);
    prop_string_render!(text; on_text_changed);
    prop_obj_render!(validator: Option<Box<dyn Validator>>);

    pub fn is_valid<State: ?Sized>(tree: &WindowTree<State>, window: Window<State>) -> bool {
        window.data::<InputLine>(tree).is_valid
    }

    fn is_numeric_raw(&self) -> bool {
        self.validator.as_deref().map_or(false, |x| x.is_numeric())
    }

    pub fn is_numeric<State: ?Sized>(tree: &WindowTree<State>, window: Window<State>) -> bool {
        window.data::<InputLine>(tree).is_numeric_raw()
    }

    fn update_is_valid<State: ?Sized>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        state: Option<&mut State>
    ) {
        let data = window.data_mut::<InputLine>(tree);
        let is_valid = data.validator.as_deref().map_or(true, |x| x.is_valid(data.editing, &data.text));
        if is_valid != data.is_valid {
            data.is_valid = is_valid;
            if let Some(state) = state {
                window.raise(tree, Event::Cmd(CMD_INPUT_LINE_IS_VALID_CHANGED), state);
            } else {
                let is_valid_timer = Timer::new(tree, 0, Box::new(move |tree, state| {
                    window.data_mut::<InputLine>(tree).is_valid_timer = None;
                    window.raise(tree, Event::Cmd(CMD_INPUT_LINE_IS_VALID_CHANGED), state);
                }));
                let data = window.data_mut::<InputLine>(tree);
                if let Some(timer) = data.is_valid_timer.replace(is_valid_timer) {
                    timer.drop_timer(tree);
                }
            }
        }
    }

    fn reset_view(&mut self, focused: bool) {
        self.cursor = self.text.len();
        if focused || self.is_numeric_raw() {
            self.calc_view_start(self.text.len());
        } else {
            self.calc_view_end(0);
        }
    }

    fn calc_view_start(&mut self, view_end: usize) {
        let mut width = 0;
        let view_start = 'r: {
            for (i, c) in self.text[.. view_end].char_indices().rev() {
                let c_width = char_width(c);
                if (self.width.wrapping_sub(width) as u16) < c_width as u16 {
                    break 'r i + c.len_utf8();
                }
                width = width.wrapping_add(c_width);
            }
            0
        };
        self.view = view_start ..  view_end;
        if self.is_numeric_raw() {
            self.view_padding = self.width.wrapping_sub(width);
        } else {
            self.view_padding = 0;
        }
    }

    fn calc_view_end(&mut self, view_start: usize) {
        let mut width = 0;
        let view_end = 'r: {
            for (i, c) in self.text[view_start ..].char_indices() {
                let c_width = char_width(c);
                if (self.width.wrapping_sub(width) as u16) < c_width as u16 {
                    break 'r view_start + i;
                }
                width = width.wrapping_add(c_width);
            }
            self.text.len()
        };
        self.view = view_start ..  view_end;
        if self.is_numeric_raw() {
            self.view_padding = self.width.wrapping_sub(width);
        } else {
            self.view_padding = 0;
        }
    }

    fn cursor_left(&mut self) {
        let Some(c) = self.text[.. self.cursor].chars().next_back() else { return; };
        self.cursor -= c.len_utf8();
        if self.cursor < self.view.start {
            self.calc_view_end(self.cursor);
        }
    }

    fn cursor_right(&mut self) {
        let Some(c) = self.text[self.cursor ..].chars().next() else { return; };
        self.cursor += c.len_utf8();
        if self.cursor >= self.view.end {
            let view_end = if let Some(c) = self.text[self.cursor ..].chars().next() {
                self.cursor + c.len_utf8()
            } else {
                self.text.len()
            };
            self.calc_view_start(view_end);
        }
    }

    fn on_text_changed<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        let focused = window.is_focused(tree);
        let data = &mut window.data_mut::<InputLine>(tree);
        data.reset_view(focused);
        Self::update_is_valid(tree, window, None);
        let data = &mut window.data_mut::<InputLine>(tree);
        if data.is_valid && !data.editing && focused {
            data.editing = true;
            Self::update_is_valid(tree, window, None);
        }
    }
}

impl Default for InputLine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct InputLineWidget;

impl<State: ?Sized> Widget<State> for InputLineWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let focused = window.is_focused(tree);
        let is_enabled = window.actual_is_enabled(tree);
        let bounds = window.inner_bounds(tree);
        let data = window.data::<InputLine>(tree);
        let color = if !is_enabled {
            1
        } else {
            if focused {
                if !data.is_valid { 4 } else { 3 }
            } else {
                if !data.is_valid { 2 } else { 0 }
            }
        };
        let color = window.color(tree, color);
        rp.fill_bg(color.1);
        rp.text(
            Point { x: data.view_padding.wrapping_add(1), y: 0 },
            color,
            &data.text[data.view.clone()]
        );
        if data.view.start > 0 {
            rp.text(Point { x: 0, y: 0 }, color, "◄");
        }
        if data.view.end < data.text.len() {
            rp.text(bounds.tr_inner(), color, "►");
        }
        if focused {
            let cursor_x = text_width(&data.text[data.view.start .. data.cursor]);
            rp.cursor(Point { x: cursor_x.wrapping_add(data.view_padding).wrapping_add(1), y: 0 });
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
        tree: &mut WindowTree<State>,
        window: Window<State>,
        final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let focused = window.is_focused(tree);
        let data = window.data_mut::<InputLine>(tree);
        data.width = Thickness::new(1, 0, 1, 0).shrink_rect(final_inner_bounds).w();
        data.reset_view(focused);
        Vector { x: final_inner_bounds.w(), y: 1 }
    }

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        state: &mut State,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) => {
                let data = window.data_mut::<InputLine>(tree);
                data.reset_view(true);
                if data.is_valid {
                    data.editing = true;
                    InputLine::update_is_valid(tree, window, Some(state));
                }
                window.invalidate_render(tree);
                false
            },
            Event::Cmd(CMD_LOST_PRIMARY_FOCUS) => {
                let data = window.data_mut::<InputLine>(tree);
                data.reset_view(false);
                data.editing = false;
                InputLine::update_is_valid(tree, window, Some(state));
                window.invalidate_render(tree);
                false
            },
            Event::Cmd(CMD_LOST_ATTENTION) => {
                let data = window.data_mut::<InputLine>(tree);
                data.editing = false;
                InputLine::update_is_valid(tree, window, Some(state));
                let data = window.data_mut::<InputLine>(tree);
                if data.is_valid {
                    data.editing = true;
                    InputLine::update_is_valid(tree, window, Some(state));
                }
                window.invalidate_render(tree);
                false
            },
            Event::Key(Key::Char(c)) => {
                if window.is_enabled(tree) {
                    let data = window.data_mut::<InputLine>(tree);
                    if data.text.try_reserve(c.len_utf8()).is_ok() {
                        data.text.insert(data.cursor, c);
                        data.calc_view_end(data.view.start);
                        data.cursor_right();
                        InputLine::update_is_valid(tree, window, Some(state));
                        let data = window.data_mut::<InputLine>(tree);
                        if data.is_valid && !data.editing {
                            data.editing = true;
                            InputLine::update_is_valid(tree, window, Some(state));
                        }
                        window.invalidate_render(tree);
                    }
                    true
                } else {
                    false
                }
            },
            Event::Key(Key::Backspace) => {
                if window.is_enabled(tree) {
                    let data = window.data_mut::<InputLine>(tree);
                    if !data.text.is_empty() {
                        data.cursor_left();
                        let c = data.text.remove(data.cursor);
                        data.calc_view_start(data.view.end - c.len_utf8());
                        InputLine::update_is_valid(tree, window, Some(state));
                        let data = window.data_mut::<InputLine>(tree);
                        if data.is_valid && !data.editing {
                            data.editing = true;
                            InputLine::update_is_valid(tree, window, Some(state));
                        }
                        window.invalidate_render(tree);
                    }
                    true
                } else {
                    false
                }
            },
            Event::Key(Key::Delete) => {
                if window.is_enabled(tree) {
                    let data = window.data_mut::<InputLine>(tree);
                    if data.cursor != data.text.len() {
                        let c = data.text.remove(data.cursor);
                        data.calc_view_start(data.view.end - c.len_utf8());
                        InputLine::update_is_valid(tree, window, Some(state));
                        let data = window.data_mut::<InputLine>(tree);
                        if data.is_valid && !data.editing {
                            data.editing = true;
                            InputLine::update_is_valid(tree, window, Some(state));
                        }
                        window.invalidate_render(tree);
                    }
                    true
                } else {
                    false
                }
            },
            Event::Key(Key::Left) => {
                let data = window.data_mut::<InputLine>(tree);
                data.cursor_left();
                window.invalidate_render(tree);
                true
            },
            Event::Key(Key::Right) => {
                let data = window.data_mut::<InputLine>(tree);
                data.cursor_right();
                window.invalidate_render(tree);
                true
            },
            Event::Key(Key::Home) => {
                let data = window.data_mut::<InputLine>(tree);
                data.cursor = 0;
                data.calc_view_end(0);
                window.invalidate_render(tree);
                true
            },
            Event::Key(Key::End) => {
                let data = window.data_mut::<InputLine>(tree);
                data.cursor = data.text.len();
                data.calc_view_start(data.text.len());
                window.invalidate_render(tree);
                true
            },
            _ => false,
        }
    }
}
