use crate::{prop_string_measure, prop_value, prop_value_render, widget};
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Key, Point, Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS, label_width, label};
use tuifw_window::{COLOR_TEXT, COLOR_HOTKEY, COLOR_DISABLED};

pub const CMD_RADIO_BUTTON_CLICK: u16 = 100;

pub struct RadioButton {
    is_on: bool,
    allow_turn_off: bool,
    cmd: u16,
    text: String,
}

impl<State: ?Sized> WidgetData<State> for RadioButton { }

impl RadioButton {
    pub fn new() -> Self {
        RadioButton {
            is_on: false,
            allow_turn_off: false,
            cmd: CMD_RADIO_BUTTON_CLICK,
            text: String::new(),
        }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(COLOR_TEXT));
            palette.set(1, Left(COLOR_HOTKEY));
            palette.set(2, Left(COLOR_DISABLED));
        });
    }

    widget!(RadioButtonWidget; init_palette);
    prop_value_render!(is_on: bool);
    prop_value!(allow_turn_off: bool);
    prop_value!(cmd: u16);
    prop_string_measure!(text);

    fn click<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, state: &mut State) -> bool {
        let data = window.data_mut::<RadioButton>(tree);
        if !data.is_on || data.allow_turn_off {
            data.is_on = !data.is_on;
            let cmd = data.cmd;
            if data.is_on {
                let mut sibling = window.next(tree);
                while sibling != window {
                    sibling.data_mut::<RadioButton>(tree).is_on = false;
                    sibling.invalidate_render(tree);
                    sibling = sibling.next(tree);
                }
            }
            window.invalidate_render(tree);
            window.raise(tree, Event::Cmd(cmd), state);
            true
        } else {
            false
        }
    }
}

impl Default for RadioButton {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct RadioButtonWidget;

impl<State: ?Sized> Widget<State> for RadioButtonWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let is_enabled = window.actual_is_enabled(tree);
        let focused = window.is_focused(tree);
        let data = window.data::<RadioButton>(tree);
        let color = window.color(tree, if is_enabled { 0 } else { 2 });
        let color_hotkey = window.color(tree, if is_enabled { 1 } else { 2 });
        rp.text(Point { x: 1, y: 0 }, color, if data.is_on { "•" } else { " " });
        rp.text(Point { x: 0, y: 0 }, color, "(");
        rp.text(Point { x: 2, y: 0 }, color, ")");
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
        let data = window.data::<RadioButton>(tree);
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
        let data = window.data::<RadioButton>(tree);
        if data.text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
        }
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
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) | Event::Cmd(CMD_LOST_PRIMARY_FOCUS) => {
                window.invalidate_render(tree);
                false
            },
            Event::Key(Key::Char(' ')) => {
                if window.actual_is_enabled(tree) {
                    RadioButton::click(tree, window, state)
                } else {
                    false
                }
            },
            Event::PostProcessKey(Key::Alt(c)) | Event::PostProcessKey(Key::Char(c)) => {
                if window.actual_is_enabled(tree) {
                    let data = window.data_mut::<RadioButton>(tree);
                    let label = label(&data.text);
                    if Some(c) == label {
                        window.set_focused_primary(tree, true);
                        RadioButton::click(tree, window, state);
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
