use crate::{prop_string_measure, prop_value, prop_value_render, widget};
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Key, Point, Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS, label_width, label};

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
            palette.set(2, Left(14));
        });
    }

    widget!(CheckBoxWidget; init_palette);
    prop_value_render!(is_on: bool);
    prop_value!(cmd: u16);
    prop_string_measure!(text);

    fn click<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, state: &mut State) {
        let data = window.data_mut::<CheckBox>(tree);
        data.is_on = !data.is_on;
        let cmd = data.cmd;
        window.invalidate_render(tree);
        window.raise(tree, Event::Cmd(cmd), state);
    }
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
        let is_enabled = window.actual_is_enabled(tree);
        let data = window.data::<CheckBox>(tree);
        let color = window.color(tree, if is_enabled { 0 } else { 1 });
        let color_hotkey = window.color(tree, if is_enabled { 2 } else { 1 });
        rp.text(Point { x: 1, y: 0 }, color, if data.is_on { "x" } else { " " });
        rp.text(Point { x: 0, y: 0 }, color, "[");
        rp.text(Point { x: 2, y: 0 }, color, "]");
        if !data.text.is_empty() {
            rp.text(Point { x: 3, y: 0 }, color, " ");
            rp.label(Point { x: 4, y: 0 }, color, color_hotkey, &data.text);
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
            Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
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
            Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
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
                if window.actual_is_enabled(tree) {
                    CheckBox::click(tree, window, state);
                    true
                } else {
                    false
                }
            },
            Event::PostProcessKey(Key::Alt(c)) | Event::PostProcessKey(Key::Char(c)) => {
                if window.actual_is_enabled(tree) {
                    let data = window.data_mut::<CheckBox>(tree);
                    let label = label(&data.text);
                    if Some(c) == label {
                        window.set_focused_primary(tree, true);
                        CheckBox::click(tree, window, state);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            _ => false
        }
    }

    fn post_process(&self) -> bool { true }
}
