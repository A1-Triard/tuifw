use alloc::boxed::Box;
use alloc::string::String;
use core::ops::{Range, RangeInclusive};
use core::str::FromStr;
use either::{Either, Left, Right};
use tuifw_screen_base::{Error, Key, Point, Rect, Screen, Vector, char_width, text_width, is_text_fit_in};
use tuifw_screen_base::{Thickness};
use tuifw_window::{Event, RenderPort, Widget, Window, WindowTree};

#[derive(Debug, Clone)]
pub enum InputLineValueRange {
    Any,
    Integer(RangeInclusive<i64>),
    Float(RangeInclusive<f64>),
}

pub struct InputLine {
    value_range: InputLineValueRange,
    value: String,
    view: Either<usize, usize>,
    cursor: usize,
    width: i16,
}

impl InputLine {
    pub fn new() -> Self {
        InputLine {
            value_range: InputLineValueRange::Any,
            value: String::new(),
            view: Left(0),
            cursor: 0,
            width: 0
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

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<State>, Error> {
        let mut tree = WindowTree::new(screen, Box::new(InputLineWidget), Box::new(self))?;
        let w = tree.root();
        Self::set_palette(&mut tree, w);
        Ok(tree)
    }

    pub fn error(&self) -> bool {
        if self.value.is_empty() { return false; }
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

    fn calc_value_padding_view_and_is_tail_cursor_fit(&self) -> (i16, Range<usize>, bool) {
        match self.view {
            Left(view_start) => 'r: {
                let mut width = self.width;
                for (i, c) in self.value[view_start ..].char_indices() {
                    let c_width = char_width(c);
                    if c_width as u16 > width as u16 {
                        break 'r (0, view_start .. i, false);
                    }
                    width = width.wrapping_sub(c_width);
                }
                (0, view_start .. self.value.len(), width != 0)
            },
            Right(view_end) => 'r: {
                let (value_width, value_view_end, is_tail_cursor_fit) = if view_end == self.value.len() {
                    (
                        (self.width as u16).saturating_sub(1) as i16,
                        self.value.len(),
                        self.width != 0
                    )
                } else {
                    (self.width, view_end + 1, false)
                };
                let mut width = value_width;
                for (i, c) in self.value[.. value_view_end].char_indices().rev() {
                    let c_width = char_width(c);
                    if c_width as u16 > width as u16 {
                        break 'r (width, i + c.len_utf8() .. value_view_end, is_tail_cursor_fit);
                    }
                    width -= c_width;
                }
                unreachable!()
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
        window.invalidate(tree);
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn value_mut<State: ?Sized, T>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: impl FnOnce(&mut String) -> T
    ) -> T {
        let focused = tree.focused() == window;
        let data = &mut window.data_mut::<InputLine>(tree);
        let res = value(&mut data.value);
        data.cursor = data.value.len();
        let text_fit_width = (data.width as u16).saturating_sub(1) as i16;
        if focused || is_text_fit_in(text_fit_width, &data.value) {
            data.view = Left(0);
        } else {
            data.view = Right(data.cursor);
        }
        window.invalidate(tree);
        res
    }

    pub fn cursor(&self) -> usize { self.cursor }

    pub fn set_cursor<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, value: usize) {
        let data = &mut window.data_mut::<InputLine>(tree);
        assert!(value <= data.value.len());
        data.cursor = value;
        window.invalidate(tree);
    }

    pub fn view(&self) -> Either<usize, usize> { self.view }

    pub fn set_view<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, value: Either<usize, usize>) {
        let data = &mut window.data_mut::<InputLine>(tree);
        assert!(value.map(|x| x <= data.value.len()).into_inner());
        data.view = value;
        window.invalidate(tree);
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
        let focused = tree.focused() == window;
        let data = window.data::<InputLine>(tree);
        let color = if data.error() { 1 } else { 0 };
        let color = window.color(tree, color);
        rp.fill_bg(color.1);
        let data = window.data::<InputLine>(tree);
        let (padding, view, is_tail_cursor_fit) = data.calc_value_padding_view_and_is_tail_cursor_fit();
        rp.out(Point { x: padding.wrapping_add(1), y: 0 }, color.0, color.1, &data.value[view.clone()]);
        if focused && (view.contains(&data.cursor) || data.cursor == data.value.len() && is_tail_cursor_fit) {
            let cursor_x = text_width(&data.value[view.start .. data.cursor]);
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
        let data = window.data_mut::<InputLine>(tree);
        data.width = Thickness::new(1, 0, 1, 0).shrink_rect(final_inner_bounds).w();
        let text_fit_width = if data.cursor == data.value.len() {
            (data.width as u16).saturating_sub(1) as i16
        } else {
            data.width
        };
        if is_text_fit_in(text_fit_width, &data.value) {
            data.view = Left(0);
        }
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
            Event::GotFocus => {
                window.invalidate(tree);
                true
            },
            Event::LostFocus => {
                window.invalidate(tree);
                true
            },
            Event::Key(n, key) => match key {
                Key::Char(c) => {
                    let data = window.data_mut::<InputLine>(tree);
                    for _ in 0 .. n.get() {
                        if data.value.try_reserve(c.len_utf8()).is_ok() {
                            data.value.insert(data.cursor, c);
                            data.cursor += c.len_utf8();
                            let (_, view, is_tail_cursor_fit) = data.calc_value_padding_view_and_is_tail_cursor_fit();
                            if
                                !view.contains(&data.cursor) && !(
                                    data.cursor == data.value.len() && is_tail_cursor_fit
                                )
                            {
                                data.view = Right(data.cursor);
                            }
                        }
                    }
                    window.invalidate(tree);
                    true
                },
                Key::Backspace => {
                    let data = window.data_mut::<InputLine>(tree);
                    for _ in 0 .. n.get() {
                        if let Some((i, c)) = data.value[.. data.cursor].char_indices().rev().next() {
                            data.value.remove(i);
                            data.cursor -= c.len_utf8();
                            let text_fit_width = if data.cursor == data.value.len() {
                                (data.width as u16).saturating_sub(1) as i16
                            } else {
                                data.width
                            };
                            if is_text_fit_in(text_fit_width, &data.value) {
                                data.view = Left(0);
                            } else {
                                data.view = Right(data.cursor);
                            }
                        }
                    }
                    window.invalidate(tree);
                    true
                },
                /*
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
                */
                _ => false,
            },
        }
    }
}
