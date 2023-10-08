use crate::{prop_string_measure, prop_value, prop_value_render, widget};
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Key, Point, Rect, Vector, text_width};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS};

pub const CMD_CHECK_BOX_CLICK: u16 = 100;

pub struct CheckBox {
    is_on: bool,
    cmd: u16,
    text: String,
}

impl<State: ?Sized> WidgetData<State> for CheckBox { }

impl CheckBox {
    pub fn new() -> Self {
        CheckBox {
            is_on: false,
            cmd: CMD_CHECK_BOX_CLICK,
            text: String::new(),
        }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(12));
            palette.set(1, Left(13));
        });
    }

    widget!(CheckBoxWidget; init_palette);
    prop_value_render!(is_on: bool);
    prop_value!(cmd: u16);
    prop_string_measure!(text);
}

impl Default for CheckBox {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct CheckBoxWidget;

impl<State: ?Sized> Widget<State> for CheckBoxWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let focused = window.is_focused(tree);
        let data = window.data::<CheckBox>(tree);
        let color_text = window.color(tree, 0);
        let color_label = window.color(tree, 1);
        rp.out(Point { x: 1, y: 0 }, color_text.0, color_text.1, if data.is_on { "X" } else { " " });
        rp.out(
            Point { x: 0, y: 0 },
            color_text.0,
            color_text.1,
            "["
        );
        rp.out(
            Point { x: 2, y: 0 },
            color_text.0,
            color_text.1,
            "]"
        );
        if !data.text.is_empty() {
            rp.out(Point { x: 3, y: 0 }, color_text.0, color_text.1, " ");
            let mut text_parts = data.text.split("~");
            let text_1 = text_parts.next().unwrap_or("");
            let label = text_parts.next().unwrap_or("");
            let text_2 = text_parts.next().unwrap_or("");
            let text_1_width = text_width(text_1);
            let label_width = text_width(label);
            rp.out(Point { x: 4, y: 0 }, color_text.0, color_text.1, text_1);
            rp.out(Point { x: text_1_width.wrapping_add(4), y: 0 }, color_label.0, color_label.1, label);
            rp.out(
                Point { x: text_1_width.wrapping_add(label_width).wrapping_add(4), y: 0 },
                color_text.0,
                color_text.1,
                text_2
            );
        }
        if focused { rp.cursor(Point { x: 1, y: 0 }); }
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<CheckBox>(tree);
        if data.text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            let mut text_parts = data.text.split("~");
            let text_1 = text_parts.next().unwrap_or("");
            let label = text_parts.next().unwrap_or("");
            let text_2 = text_parts.next().unwrap_or("");
            let text_1_width = text_width(text_1);
            let label_width = text_width(label);
            let text_2_width = text_width(text_2);
            Vector {
                x: text_1_width.wrapping_add(label_width).wrapping_add(text_2_width).wrapping_add(4),
                y: 1
            }
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<CheckBox>(tree);
        if data.text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            let mut text_parts = data.text.split("~");
            let text_1 = text_parts.next().unwrap_or("");
            let label = text_parts.next().unwrap_or("");
            let text_2 = text_parts.next().unwrap_or("");
            let text_1_width = text_width(text_1);
            let label_width = text_width(label);
            let text_2_width = text_width(text_2);
            Vector {
                x: text_1_width.wrapping_add(label_width).wrapping_add(text_2_width).wrapping_add(4),
                y: 1
            }
        }
    }

    fn focusable(&self, primary_focus: bool) -> bool { primary_focus }

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        state: &mut State,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) | Event::Cmd(CMD_LOST_PRIMARY_FOCUS) => {
                window.invalidate_render(tree);
                false
            },
            Event::Key(Key::Char(' ')) => {
                let data = window.data_mut::<CheckBox>(tree);
                data.is_on = !data.is_on;
                let cmd = data.cmd;
                window.invalidate_render(tree);
                window.raise(tree, Event::Cmd(cmd), state);
                true
            },
            Event::PostProcessKey(Key::Alt(c)) | Event::PostProcessKey(Key::Char(c)) => {
                let data = window.data_mut::<CheckBox>(tree);
                let label = data.text
                    .split("~").skip(1).next().unwrap_or("")
                    .chars().next().and_then(|x| x.to_lowercase().next());
                if Some(c) == label {
                    let cmd = data.cmd;
                    window.raise(tree, Event::Cmd(cmd), state);
                    let focus = window.actual_focus_tab(tree);
                    focus.set_focused_primary(tree, true);
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }

    fn post_process(&self) -> bool { true }
}
