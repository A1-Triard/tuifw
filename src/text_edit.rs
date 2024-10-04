use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::Range;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Vector, Rect, Point, Fg, Bg, text_width, Key};
use tuifw_window::{App, Window, Event, WindowTree, RenderPort, Widget, WidgetData};
use unicode_width::UnicodeWidthChar;
use crate::widget;

struct Line {
    range: Range<usize>,
    view: Range<usize>,
    has_line_break: bool,
    padding: u16,
    has_line_break_place: bool,
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
    }
}

impl TextEdit {
    fn reset_view(tree: &mut WindowTree, window: Window) {
        let data = window.data_mut::<TextEdit>(tree);
        data.cursor = 0;
        data.column = 0;
        data.lines.clear();
        Self::recalc_view(tree, window);
    }

    fn recalc_view(tree: &mut WindowTree, window: Window) {
        let data = window.data_mut::<TextEdit>(tree);
        let text = if size_of::<usize>() <= size_of::<u32>() || data.text.len() <= usize::try_from(u32::MAX).unwrap() {
            &data.text[..]
        } else {
            &data.text[.. usize::try_from(u32::MAX).unwrap()]
        };
        let mut line_start = data.lines.first().map_or(0, |x| x.range.start);
        data.lines.clear();
        for _ in 0 .. data.size.y as u16 {
            let (line_end, has_line_break) = text[line_start ..]
                .find(&data.line_break)
                .map_or((text.len(), false), |x| (line_start + x, true));
            let mut line_view_start = line_start;
            let line_padding;
            let mut width = 0;
            loop {
                if width == data.column {
                    line_padding = 0;
                    break;
                }
                let Some(c) = text[line_view_start .. line_end].chars().next() else {
                    line_padding = 0;
                    break;
                };
                let c_width = if c == '\0' { 0 } else { c.width().unwrap_or(0) as u64 };
                if width + c_width > data.column {
                    line_padding = width + c_width - data.column;                    
                    line_view_start += c.len_utf8();
                    break;
                }
                line_view_start += c.len_utf8();
                width += c_width;
            }
            let mut has_line_break_place = false;
            let mut line_view_end = line_view_start;
            let mut width = 0;
            loop {
                if width >= data.size.x as u16 as u64 {
                    break;
                }
                let Some(c) = text[line_view_end .. line_end].chars().next() else {
                    has_line_break_place = true;
                    break;
                };
                let c_width = if c == '\0' { 0 } else { c.width().unwrap_or(0) as u64 };
                line_view_end += c.len_utf8();
                width += c_width;
            }
            data.lines.push(Line {
                range: line_start .. line_end,
                view: line_view_start .. line_view_end,
                has_line_break,
                padding: u16::try_from(line_padding).unwrap(),
                has_line_break_place,
            });
            line_start = line_end + if has_line_break { data.line_break.len() } else { 0 };
        }
        window.invalidate_render(tree);
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
            if !line.has_line_break && line.range.end == line.range.start {
                rp.text(Point { x: line.padding as i16, y }, (Fg::DarkGray, Bg::None), "~");
            } else {
                rp.text(Point { x: line.padding as i16, y }, (Fg::LightGray, Bg::None), &data.text[line.view.clone()]);
                if !line.has_line_break {
                    rp.text(
                        Point {
                            x: (line.padding as i16).wrapping_add(text_width(&data.text[line.view.clone()])),
                            y
                        }, (Fg::DarkGray, Bg::None), "%"
                    );
                }
            }
            if
                focused && (line.range.contains(&data.cursor) ||
                line.has_line_break && data.cursor == line.range.end)
            {
                if
                    line.view.contains(&data.cursor) ||
                    line.has_line_break_place && data.cursor == line.range.end
                {
                    rp.cursor(Point {
                        x:
                            (line.padding as i16).wrapping_add(
                            text_width(&data.text[line.view.start .. data.cursor])),
                        y
                    });
                }
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
        TextEdit::recalc_view(tree, window);
        final_inner_bounds.size
    }

    fn update(
        &self,
        _tree: &mut WindowTree,
        _window: Window,
        event: Event,
        _event_source: Window,
        _app: &mut dyn App,
    ) -> bool {
        match event {
            Event::Key(Key::Right) => {
                false
            },
            _ => false
        }
    }
}
