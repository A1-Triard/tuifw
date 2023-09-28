use alloc::boxed::Box;
use alloc::string::String;
use core::ops::{Range, RangeInclusive};
use core::str::FromStr;
use either::{Either, Left, Right};
use tuifw_screen_base::{Error, Key, Point, Rect, Vector, char_width, text_width, is_text_fit_in};
use tuifw_screen_base::{Thickness};
use tuifw_window::{Event, RenderPort, Timer, Widget, WidgetData, Window, WindowTree};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS};

pub const CMD_IS_VALID_EMPTY_CHANGED: u16 = 110;

#[derive(Debug, Clone)]
pub enum InputLineValueRange {
    Any,
    Integer(RangeInclusive<i64>),
    Float(RangeInclusive<f64>),
}

pub struct InputLine {
    value_range: InputLineValueRange,
    default: String,
    text: String,
    is_valid: bool,
    is_empty: bool,
    view: Either<usize, usize>,
    cursor: usize,
    width: i16,
    is_valid_empty_timer: Option<Timer>,
}

impl<State: ?Sized> WidgetData<State> for InputLine {
    fn drop_widget_data(&mut self, tree: &mut WindowTree<State>, _state: &mut State) {
        if let Some(timer) = self.is_valid_empty_timer.take() {
            timer.drop_timer(tree);
        }
    }
}

impl InputLine {
    pub fn new() -> Self {
        InputLine {
            value_range: InputLineValueRange::Any,
            default: String::new(),
            text: String::new(),
            is_valid: true,
            is_empty: true,
            view: Left(0),
            cursor: 0,
            width: 0,
            is_valid_empty_timer: None,
        }
    }

    fn set_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(12));
            palette.set(1, Left(13));
        });
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        let w = Window::new(tree, Box::new(InputLineWidget), Box::new(self), parent, prev)?;
        Self::set_palette(tree, w);
        Ok(w)
    }

    fn update_is_valid_empty<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        let data = window.data_mut::<InputLine>(tree);
        let (is_valid, is_empty) = if data.text.is_empty() {
            (true, true)
        } else {
            (match &data.value_range {
                InputLineValueRange::Any => true,
                InputLineValueRange::Integer(range) => if let Ok(value) = i64::from_str(&data.text) {
                    range.contains(&value)
                } else {
                    false
                },
                InputLineValueRange::Float(range) => if let Ok(value) = f64::from_str(&data.text) {
                    range.contains(&value)
                } else {
                    false
                },
            }, false)
        };
        if is_valid != data.is_valid || is_empty != data.is_empty {
            data.is_valid = is_valid;
            data.is_empty = is_empty;
            let is_valid_empty_timer = Timer::new(tree, 0, Box::new(move |tree, state| {
                window.data_mut::<InputLine>(tree).is_valid_empty_timer = None;
                window.raise(tree, Event::Cmd(CMD_IS_VALID_EMPTY_CHANGED), state);
            }));
            let data = window.data_mut::<InputLine>(tree);
            if let Some(timer) = data.is_valid_empty_timer.replace(is_valid_empty_timer) {
                timer.drop_timer(tree);
            }
        }
    }

    pub fn is_valid(&self) -> bool { self.is_valid }

    pub fn is_empty(&self) -> bool { self.is_empty }

    fn calc_value_padding_view_and_is_tail_cursor_fit(&self) -> (i16, Range<usize>, bool) {
        match self.view {
            Left(view_start) => 'r: {
                let mut width = self.width;
                for (i, c) in self.text[view_start ..].char_indices() {
                    let c_width = char_width(c);
                    if c_width as u16 > width as u16 {
                        break 'r (0, view_start .. i, false);
                    }
                    width = width.wrapping_sub(c_width);
                }
                (0, view_start .. self.text.len(), width != 0)
            },
            Right(view_end) => 'r: {
                let (text_width, value_view_end, is_tail_cursor_fit) = if view_end == self.text.len() {
                    (
                        (self.width as u16).saturating_sub(1) as i16,
                        self.text.len(),
                        self.width != 0
                    )
                } else {
                    (self.width, view_end + 1, false)
                };
                let mut width = text_width;
                for (i, c) in self.text[.. value_view_end].char_indices().rev() {
                    let c_width = char_width(c);
                    if c_width as u16 > width as u16 {
                        break 'r (width, i + c.len_utf8() .. value_view_end, is_tail_cursor_fit);
                    }
                    width -= c_width;
                }
                (width, 0 .. value_view_end, is_tail_cursor_fit)
            }
        }
    }

    pub fn value_range(&self) -> InputLineValueRange { self.value_range.clone() }

    pub fn set_value_range<State: ?Sized>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: InputLineValueRange
    ) {
        window.data_mut::<InputLine>(tree).value_range = value;
        window.invalidate_render(tree);
    }

    pub fn default(&self) -> &String {
        &self.default
    }

    pub fn default_mut<State: ?Sized, T>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: impl FnOnce(&mut String) -> T
    ) -> T {
        let data = &mut window.data_mut::<InputLine>(tree);
        value(&mut data.default)
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn text_mut<State: ?Sized, T>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: impl FnOnce(&mut String) -> T
    ) -> T {
        let focused = window.is_focused(tree);
        let data = &mut window.data_mut::<InputLine>(tree);
        let res = value(&mut data.text);
        data.cursor = data.text.len();
        if focused {
            let text_fit_width = (data.width as u16).saturating_sub(1) as i16;
            if is_text_fit_in(text_fit_width, &data.text) {
                data.view = if matches!(data.value_range, InputLineValueRange::Any) {
                    Left(0)
                } else {
                    Right(data.text.len())
                };
            } else {
                data.view = Right(data.cursor);
            }
        } else {
            data.view = if matches!(data.value_range, InputLineValueRange::Any) || data.text.is_empty() {
                Left(0)
            } else {
                Right(data.text.len() - 1)
            };
        }
        Self::update_is_valid_empty(tree, window);
        window.invalidate_render(tree);
        res
    }

    pub fn cursor(&self) -> usize { self.cursor }

    pub fn set_cursor<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, value: usize) {
        let data = &mut window.data_mut::<InputLine>(tree);
        assert!(value <= data.text.len());
        data.cursor = value;
        window.invalidate_render(tree);
    }

    pub fn view(&self) -> Either<usize, usize> { self.view }

    pub fn set_view<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, value: Either<usize, usize>) {
        let data = &mut window.data_mut::<InputLine>(tree);
        assert!(value.map(|x| x <= data.text.len()).into_inner());
        data.view = value;
        window.invalidate_render(tree);
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
        let data = window.data::<InputLine>(tree);
        let color = if !data.is_valid() { 1 } else { 0 };
        let color = window.color(tree, color);
        rp.fill_bg(color.1);
        let (padding, view, is_tail_cursor_fit) = data.calc_value_padding_view_and_is_tail_cursor_fit();
        rp.out(Point { x: padding.wrapping_add(1), y: 0 }, color.0, color.1, &data.text[view.clone()]);
        if focused && (view.contains(&data.cursor) || data.cursor == data.text.len() && is_tail_cursor_fit) {
            let cursor_x = text_width(&data.text[view.start .. data.cursor]);
            rp.cursor(Point { x: cursor_x.wrapping_add(padding).wrapping_add(1), y: 0 });
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
        let text_fit_width = if focused && data.cursor == data.text.len() {
            (data.width as u16).saturating_sub(1) as i16
        } else {
            data.width
        };
        if is_text_fit_in(text_fit_width, &data.text) {
            if focused && data.cursor == data.text.len() {
                data.view = if matches!(data.value_range, InputLineValueRange::Any) {
                    Left(0)
                } else {
                    Right(data.text.len())
                };
            } else {
                data.view = if matches!(data.value_range, InputLineValueRange::Any) || data.text.is_empty() {
                    Left(0)
                } else {
                    Right(data.text.len() - 1)
                };
            }
        }
        Vector { x: final_inner_bounds.w(), y: 1 }
    }

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        _state: &mut State,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) => {
                let data = window.data_mut::<InputLine>(tree);
                let text_fit_width = if data.cursor == data.text.len() {
                    (data.width as u16).saturating_sub(1) as i16
                } else {
                    data.width
                };
                if is_text_fit_in(text_fit_width, &data.text) {
                    if data.cursor == data.text.len() {
                        data.view = if matches!(data.value_range, InputLineValueRange::Any) {
                            Left(0)
                        } else {
                            Right(data.text.len())
                        };
                    } else {
                        data.view = if
                            matches!(data.value_range, InputLineValueRange::Any) || data.text.is_empty()
                        {
                            Left(0)
                        } else {
                            Right(data.text.len() - 1)
                        };
                    }
                } else {
                    data.view = Right(data.cursor);
                }
                window.invalidate_render(tree);
                true
            },
            Event::Cmd(CMD_LOST_PRIMARY_FOCUS) => {
                let data = window.data_mut::<InputLine>(tree);
                if data.text.is_empty() {
                    data.text = data.default.clone();
                    if !data.text.is_empty() {
                        data.cursor = data.text.len();
                    }
                }
                data.view = if matches!(data.value_range, InputLineValueRange::Any) || data.text.is_empty() {
                    Left(0)
                } else {
                    Right(data.text.len() - 1)
                };
                InputLine::update_is_valid_empty(tree, window);
                window.invalidate_render(tree);
                true
            },
            Event::Key(n, key) => match key {
                Key::Char(c) => {
                    let data = window.data_mut::<InputLine>(tree);
                    for _ in 0 .. n.get() {
                        if data.text.try_reserve(c.len_utf8()).is_ok() {
                            data.text.insert(data.cursor, c);
                            data.cursor += c.len_utf8();
                            let (_, view, is_tail_cursor_fit) =
                                data.calc_value_padding_view_and_is_tail_cursor_fit();
                            if
                                !view.contains(&data.cursor) && !(
                                    data.cursor == data.text.len() && is_tail_cursor_fit
                                )
                            {
                                data.view = Right(data.cursor);
                            }
                        }
                    }
                    InputLine::update_is_valid_empty(tree, window);
                    window.invalidate_render(tree);
                    true
                },
                Key::Backspace => {
                    let data = window.data_mut::<InputLine>(tree);
                    for _ in 0 .. n.get() {
                        if let Some((i, c)) = data.text[.. data.cursor].char_indices().next_back() {
                            data.text.remove(i);
                            data.cursor -= c.len_utf8();
                            let text_fit_width = if data.cursor == data.text.len() {
                                (data.width as u16).saturating_sub(1) as i16
                            } else {
                                data.width
                            };
                            if is_text_fit_in(text_fit_width, &data.text) {
                                if data.cursor == data.text.len() {
                                    data.view = if matches!(data.value_range, InputLineValueRange::Any) {
                                        Left(0)
                                    } else {
                                        Right(data.text.len())
                                    };
                                } else {
                                    data.view = if
                                        matches!(data.value_range, InputLineValueRange::Any) ||
                                        data.text.is_empty()
                                    {
                                        Left(0)
                                    } else {
                                        Right(data.text.len() - 1)
                                    };
                                }
                            } else {
                                data.view = Right(data.cursor);
                            }
                        }
                    }
                    InputLine::update_is_valid_empty(tree, window);
                    window.invalidate_render(tree);
                    true
                },
                _ => false,
            },
            _ => false,
        }
    }
}
