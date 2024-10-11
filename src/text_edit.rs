use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cmp::{Ordering, min};
use core::ops::Range;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Vector, Rect, Point, Fg, Bg, text_width, Key};
use tuifw_window::{App, Window, Event, WindowTree, RenderPort, Widget, WidgetData};
use unicode_width::UnicodeWidthChar;
use crate::widget;

struct Line {
    range: Range<usize>,
    view: Range<usize>,
    filled: bool,
    padding: u16,
}

widget! {
    #[widget(TextEditWidget)]
    pub struct TextEdit {
        #[property(str, on_changed=reset_view)]
        text: String,
        #[property(str, on_changed=reset_view)]
        line_break: String,
        size: Vector,
        lines: Vec<Line>,
        column: u64,
        cursor: usize,
        cursor_line: Option<usize>,
    }
}

fn actual_text(text: &str) -> &str {
    if
        size_of::<usize>() <= size_of::<u32>() || text.len() <= usize::try_from(u32::MAX).unwrap()
    {
        text
    } else {
        &text[.. usize::try_from(u32::MAX).unwrap()]
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Char {
    Char(char),
    LineBreak,
}

impl Char {
    fn width(self) -> u64 {
        match self {
            Char::Char('\0') => 0,
            Char::Char(c) => c.width().unwrap_or(0) as u64,
            Char::LineBreak => 1,
        }
    }

    fn len(self, line_break: &str) -> usize {
        match self {
            Char::Char(c) => c.len_utf8(),
            Char::LineBreak => line_break.len(),
        }
    }
}

fn first_char(text: &str, line_break: &str) -> Option<Char> {
    if text.starts_with(line_break) {
        Some(Char::LineBreak)
    } else if let Some(c) = text.chars().next() {
        Some(Char::Char(c))
    } else {
        None
    }
}

fn last_char(text: &str, line_break: &str) -> Option<Char> {
    if text.ends_with(line_break) {
        Some(Char::LineBreak)
    } else if let Some(c) = text.chars().last() {
        Some(Char::Char(c))
    } else {
        None
    }
}

impl TextEdit {
    fn reset_view(tree: &mut WindowTree, window: Window) {
        let data = window.data_mut::<TextEdit>(tree);
        data.cursor = 0;
        data.column = 0;
        data.lines.clear();
        Self::recalc_view(tree, window, None);
    }

    fn recalc_view(tree: &mut WindowTree, window: Window, start: Option<usize>) {
        let data = window.data_mut::<TextEdit>(tree);
        let text = actual_text(&data.text);
        let mut line_start = start.unwrap_or_else(|| data.lines.first().map_or(0, |x| x.range.start));
        data.lines.clear();
        for _ in 0 .. data.size.y as u16 {
            let line_end = text[line_start ..]
                .find(&data.line_break)
                .map_or(text.len(), |x| line_start + x + data.line_break.len());
            let mut line_view_start = line_start;
            let line_padding;
            let mut width = 0;
            loop {
                if width == data.column {
                    line_padding = 0;
                    break;
                }
                let Some(c) = first_char(&text[line_view_start .. line_end], &data.line_break) else {
                    line_padding = 0;
                    break;
                };
                let c_width = c.width();
                if width + c_width > data.column {
                    line_padding = width + c_width - data.column;                    
                    line_view_start += c.len(&data.line_break);
                    break;
                }
                line_view_start += c.len(&data.line_break);
                width += c_width;
            }
            let line_filled;
            let mut line_view_end = line_view_start;
            let mut width = 0;
            loop {
                if width == data.size.x as u16 as u64 {
                    line_filled = true;
                    break;
                }
                let Some(c) = first_char(&text[line_view_end .. line_end], &data.line_break) else {
                    line_filled = false;
                    break;
                };
                let c_width = c.width();
                if width + c_width > data.size.x as u16 as u64 {
                    line_filled = true;
                    break;
                }
                line_view_end += c.len(&data.line_break);
                width += c_width;
            }
            data.lines.push(Line {
                range: line_start .. line_end,
                view: line_view_start .. line_view_end,
                filled: line_filled,
                padding: u16::try_from(line_padding).unwrap(),
            });
            line_start = line_end;
        }
        data.cursor_line = data.lines.binary_search_by(|line| {
            if line.range.contains(&data.cursor) || !text[line.range.clone()].ends_with(&data.line_break) && data.cursor == line.range.end {
                Ordering::Equal
            } else if data.cursor >= line.range.start {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }).ok();
        window.invalidate_render(tree);
    }

    fn cursor_left(tree: &mut WindowTree, window: Window) -> bool {
        let data = window.data_mut::<TextEdit>(tree);
        let text = actual_text(&data.text);
        if data.cursor_line.is_none() {
            let line_start =
                text[.. data.cursor].rfind(&data.line_break).map_or(0, |x| x + data.line_break.len());
            Self::recalc_view(tree, window, Some(line_start));
        }
        let data = window.data_mut::<TextEdit>(tree);
        let line = &data.lines[data.cursor_line.unwrap()];
        if data.cursor > line.view.end || data.cursor == line.view.end && line.filled {
            let text = actual_text(&data.text);
            data.column = text[line.range.start .. data.cursor].chars().map(|x| if x == '\0' { 0 } else { x.width().unwrap_or(0) as u64 }).sum();
            Self::recalc_view(tree, window, None);
        }
        let data = window.data_mut::<TextEdit>(tree);
        let text = actual_text(&data.text);
        let line = &data.lines[data.cursor_line.unwrap()];
        let Some(Char::Char(c)) = last_char(&text[line.range.start .. data.cursor], &data.line_break) else { return false; };
        data.cursor -= c.len_utf8();
        if data.cursor < line.view.start {
            let offset = text[data.cursor .. line.view.start].chars().map(|x| if x == '\0' { 0 } else { x.width().unwrap_or(0) as u64 }).sum();
            Self::scroll_left(tree, window, offset);
        }
        window.invalidate_render(tree);
        true
    }

    fn cursor_right(tree: &mut WindowTree, window: Window) -> bool {
        let data = window.data_mut::<TextEdit>(tree);
        let text = actual_text(&data.text);
        if data.cursor_line.is_none() {
            let line_start =
                text[.. data.cursor].rfind(&data.line_break).map_or(0, |x| x + data.line_break.len());
            Self::recalc_view(tree, window, Some(line_start));
        }
        let data = window.data_mut::<TextEdit>(tree);
        let line = &data.lines[data.cursor_line.unwrap()];
        if data.cursor < line.view.start {
            let text = actual_text(&data.text);
            data.column = text[line.range.start .. data.cursor].chars().map(|x| if x == '\0' { 0 } else { x.width().unwrap_or(0) as u64 }).sum();
            Self::recalc_view(tree, window, None);
        }
        let data = window.data_mut::<TextEdit>(tree);
        let text = actual_text(&data.text);
        let line = &data.lines[data.cursor_line.unwrap()];
        let Some(Char::Char(c)) = first_char(&text[data.cursor .. line.range.end], &data.line_break) else { return false; };
        data.cursor += c.len_utf8();
        if data.cursor > line.view.end || data.cursor == line.view.end && line.filled {
            let mut offset = text[line.view.end .. data.cursor].chars().map(|x| if x == '\0' { 0 } else { x.width().unwrap_or(0) as u64 }).sum();
            offset += first_char(&text[data.cursor .. line.range.end], &data.line_break).map_or(1, |x| x.width());
            if offset != 0 {
                Self::scroll_right(tree, window, offset);
            }
        }
        window.invalidate_render(tree);
        true
    }

    fn cursor_down(tree: &mut WindowTree, window: Window) -> bool {
        let data = window.data_mut::<TextEdit>(tree);
        let text = actual_text(&data.text);
        if data.cursor_line.is_none() {
            let line_start =
                text[.. data.cursor].rfind(&data.line_break).map_or(0, |x| x + data.line_break.len());
            Self::recalc_view(tree, window, Some(line_start));
        }
        let data = window.data_mut::<TextEdit>(tree);
        let line = &data.lines[data.cursor_line.unwrap()];
        if !line.view.contains(&data.cursor) {
            let text = actual_text(&data.text);
            data.column = text[line.range.start .. data.cursor].chars().map(|x| if x == '\0' { 0 } else { x.width().unwrap_or(0) as u64 }).sum();
            Self::recalc_view(tree, window, None);
        }
        let data = window.data_mut::<TextEdit>(tree);
        if data.cursor_line.unwrap() == data.lines.len() - 1 || data.lines[data.cursor_line.unwrap() + 1].range.is_empty() {
            if !Self::scroll_down(tree, window) { return false; }
        }
        let data = window.data_mut::<TextEdit>(tree);
        let text = actual_text(&data.text);
        let line = &data.lines[data.cursor_line.unwrap()];
        let cursor_column = text[line.range.start .. data.cursor].chars().map(|x| if x == '\0' { 0 } else { x.width().unwrap_or(0) as u64 }).sum();
        let next_line = &data.lines[data.cursor_line.unwrap() + 1];
        let mut new_cursor = next_line.range.start;
        let mut width = 0;
        loop {
            if width == cursor_column {
                break;
            }
            let Some(Char::Char(c)) = first_char(&text[new_cursor .. next_line.range.end], &data.line_break) else {
                break;
            };
            let c_width = if c == '\0' { 0 } else { c.width().unwrap_or(0) as u64 };
            if width + c_width > cursor_column {
                break;
            }
            new_cursor += c.len_utf8();
            width += c_width;
        }
        data.cursor = new_cursor;
        data.cursor_line = Some(data.cursor_line.unwrap() + 1);
        window.invalidate_render(tree);
        true
    }

    fn scroll_down(_tree: &mut WindowTree, _window: Window) -> bool {
        false
    }

    fn scroll_left(tree: &mut WindowTree, window: Window, delta: u64) {
        let data = window.data_mut::<TextEdit>(tree);
        data.column -= delta;
        let text = actual_text(&data.text);
        for line in &mut data.lines {
            let mut delta = delta;
            if line.padding != 0 {
                let c = last_char(&text[line.range.start .. line.view.start], &data.line_break).unwrap();
                let c_width = c.width();
                let padding_delta = u16::try_from(min(delta, c_width - u64::from(line.padding))).unwrap();
                line.padding += padding_delta;
                delta -= u64::from(padding_delta);
                if u64::from(line.padding) == c_width {
                    line.padding = 0;
                    line.view.start -= c.len(&data.line_break);
                }
            }
            let mut new_view_start = line.view.start;
            if delta != 0 {
                loop {
                    let Some(c) = last_char(&text[line.range.start .. new_view_start], &data.line_break) else {
                        line.padding = 0;
                        break;
                    };
                    let c_width = c.width();
                    if c_width > delta {
                        line.padding = u16::try_from(delta).unwrap();
                        break;
                    }
                    delta -= c_width;
                    new_view_start -= c.len(&data.line_break);
                }
            }
            let line_filled;
            let mut line_view_end = new_view_start;
            let mut width = 0;
            loop {
                if width == data.size.x as u16 as u64 {
                    line_filled = true;
                    break;
                }
                let Some(c) = first_char(&text[line_view_end .. line.range.end], &data.line_break) else {
                    line_filled = false;
                    break;
                };
                let c_width = c.width();
                if width + c_width > data.size.x as u16 as u64 {
                    line_filled = true;
                    break;
                }
                line_view_end += c.len(&data.line_break);
                width += c_width;
            }
            line.view = new_view_start .. line_view_end;
            line.filled = line_filled;
        }
    }

    fn scroll_right(tree: &mut WindowTree, window: Window, delta: u64) {
        let data = window.data_mut::<TextEdit>(tree);
        data.column += delta;
        let text = actual_text(&data.text);
        for line in &mut data.lines {
            let mut delta = delta;
            let padding_delta = u16::try_from(min(delta, u64::from(line.padding))).unwrap();
            line.padding -= padding_delta;
            delta -= u64::from(padding_delta);
            let mut new_view_start = line.view.start;
            if delta != 0 {
                loop {
                    let Some(c) = first_char(&text[new_view_start .. line.range.end], &data.line_break) else {
                        line.padding = 0;
                        break;
                    };
                    let c_width = c.width();
                    if c_width >= delta {
                        line.padding = u16::try_from(c_width - delta).unwrap();
                        new_view_start += c.len(&data.line_break);
                        break;
                    }
                    delta -= c_width;
                    new_view_start += c.len(&data.line_break);
                }
            }
            let line_filled;
            let mut line_view_end = new_view_start;
            let mut width = 0;
            loop {
                if width == data.size.x as u16 as u64 {
                    line_filled = true;
                    break;
                }
                let Some(c) = first_char(&text[line_view_end .. line.range.end], &data.line_break) else {
                    line_filled = false;
                    break;
                };
                let c_width = c.width();
                if width + c_width > data.size.x as u16 as u64 {
                    line_filled = true;
                    break;
                }
                line_view_end += c.len(&data.line_break);
                width += c_width;
            }
            line.view = new_view_start .. line_view_end;
            line.filled = line_filled;
        }
    }

    fn insert(tree: &mut WindowTree, window: Window, s: &str) {
        let data = window.data_mut::<TextEdit>(tree);
        debug_assert!(!s.contains(&data.line_break));
        let text = actual_text(&data.text);
        if data.cursor_line.is_none() {
            let line_start =
                text[.. data.cursor].rfind(&data.line_break).map_or(0, |x| x + data.line_break.len());
            Self::recalc_view(tree, window, Some(line_start));
        }
        let data = window.data_mut::<TextEdit>(tree);
        let line = &mut data.lines[data.cursor_line.unwrap()];
        if data.cursor < line.view.start {
            let text = actual_text(&data.text);
            data.column = text[line.range.start .. data.cursor].chars().map(|x| if x == '\0' { 0 } else { x.width().unwrap_or(0) as u64 }).sum();
            Self::recalc_view(tree, window, None);
        }
        let data = window.data_mut::<TextEdit>(tree);
        data.text.insert_str(data.cursor, s);
        let line = &mut data.lines[data.cursor_line.unwrap()];
        line.range.end += s.len();
        if data.cursor < line.view.end {
            let text = actual_text(&data.text);
            let line_filled;
            let mut line_view_end = line.view.start;
            let mut width = 0;
            loop {
                if width == data.size.x as u16 as u64 {
                    line_filled = true;
                    break;
                }
                let Some(c) = first_char(&text[line_view_end .. line.range.end], &data.line_break) else {
                    line_filled = false;
                    break;
                };
                let c_width = c.width();
                if width + c_width > data.size.x as u16 as u64 {
                    line_filled = true;
                    break;
                }
                line_view_end += c.len(&data.line_break);
                width += c_width;
            }
            line.view.end = line_view_end;
            line.filled = line_filled;
        }
        for line in data.lines[data.cursor_line.unwrap() ..].iter_mut().skip(1) {
            line.range.end += s.len();
            line.range.start += s.len();
            line.view.end += s.len();
            line.view.start += s.len();
        }
    }
}

#[derive(Clone, Default)]
struct TextEditWidget;

impl_supports_interfaces!(TextEditWidget);

impl Widget for TextEditWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(TextEdit {
            text: String::new(),
            line_break: "\n".to_string(),
            size: Vector::null(),
            lines: Vec::new(),
            column: 0,
            cursor: 0,
            cursor_line: None,
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        TextEdit::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
    ) {
        let focused = window.is_focused(tree);
        let data = window.data::<TextEdit>(tree);
        let mut y = 0;
        for line in &data.lines {
            if line.range.end == line.range.start {
                rp.text(Point { x: line.padding as i16, y }, (Fg::DarkGray, Bg::None), "~");
            } else {
                let view = if data.text[line.view.clone()].ends_with(&data.line_break) {
                    line.view.start .. line.view.end - data.line_break.len()
                } else {
                    line.view.clone()
                };
                rp.text(Point { x: line.padding as i16, y }, (Fg::LightGray, Bg::None), &data.text[view]);
                if line.view.end == line.range.end && !data.text[line.range.clone()].ends_with(&data.line_break) {
                    rp.text(
                        Point {
                            x: (line.padding as i16).wrapping_add(text_width(&data.text[line.view.clone()])),
                            y
                        }, (Fg::DarkGray, Bg::None), "%"
                    );
                }
            }
            if 
                focused && (
                    line.view.contains(&data.cursor) ||
                    line.view.end == line.range.end && !data.text[line.range.clone()].ends_with(&data.line_break) && data.cursor == line.range.end
                )
            {
                rp.cursor(Point {
                    x:
                        (line.padding as i16).wrapping_add(
                        text_width(&data.text[line.view.start .. data.cursor])),
                    y
                });
            }
            y = y.wrapping_add(1);
        }
    }

    fn measure(
        &self,
        _tree: &mut WindowTree,
        _window: Window,
        available_width: Option<i16>,
        available_height: Option<i16>,
        _app: &mut dyn App,
    ) -> Vector {
        Vector { x: available_width.unwrap_or(1), y: available_height.unwrap_or(1) }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        _app: &mut dyn App,
    ) -> Vector {
        window.data_mut::<TextEdit>(tree).size = final_inner_bounds.size;
        TextEdit::recalc_view(tree, window, None);
        final_inner_bounds.size
    }

    fn update(
        &self,
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        _event_source: Window,
        _app: &mut dyn App,
    ) -> bool {
        match event {
            Event::Key(Key::Right) => TextEdit::cursor_right(tree, window),
            Event::Key(Key::Left) => TextEdit::cursor_left(tree, window),
            Event::Key(Key::Down) => TextEdit::cursor_down(tree, window),
            Event::Key(Key::Char(c)) => {
                let mut b = [0; 4];
                let s = c.encode_utf8(&mut b);
                TextEdit::insert(tree, window, s);
                TextEdit::cursor_right(tree, window);
                true
            }
            _ => false
        }
    }
}
