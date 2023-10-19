use crate::{prop_string_measure, prop_value, prop_value_render, widget};
use alloc::boxed::Box;
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Key, Point, Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS, label_width, label};
use tuifw_window::{COLOR_TEXT, COLOR_HOTKEY, COLOR_DISABLED};

pub const CMD_CHECK_BOX_CLICK: u16 = 100;

pub struct CheckBox {
    is_on: bool,
    cmd: u16,
    text: String,
}

impl WidgetData for CheckBox { }

impl CheckBox {
    fn init_palette(tree: &mut WindowTree, window: Window) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(COLOR_TEXT));
            palette.set(1, Left(COLOR_HOTKEY));
            palette.set(2, Left(COLOR_DISABLED));
        });
    }

    widget!(CheckBoxWidget; init_palette);
    prop_value_render!(is_on: bool);
    prop_value!(cmd: u16);
    prop_string_measure!(text);

    fn click(tree: &mut WindowTree, window: Window, app: &mut dyn App) {
        let data = window.data_mut::<CheckBox>(tree);
        data.is_on = !data.is_on;
        let cmd = data.cmd;
        window.invalidate_render(tree);
        window.raise(tree, Event::Cmd(cmd), app);
    }
}

#[derive(Clone, Default)]
pub struct CheckBoxWidget;

impl Widget for CheckBoxWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(CheckBox {
            is_on: false,
            cmd: CMD_CHECK_BOX_CLICK,
            text: String::new(),
        })
    }

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
    ) {
        let focused = window.is_focused(tree);
        let is_enabled = window.actual_is_enabled(tree);
        let data = window.data::<CheckBox>(tree);
        let color = window.color(tree, if is_enabled { 0 } else { 2 });
        let color_hotkey = window.color(tree, if is_enabled { 1 } else { 2 });
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
        tree: &mut WindowTree,
        window: Window,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _app: &mut dyn App,
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
        tree: &mut WindowTree,
        window: Window,
        _final_inner_bounds: Rect,
        _app: &mut dyn App,
    ) -> Vector {
        let data = window.data::<CheckBox>(tree);
        if data.text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
        }
    }

    fn update(
        &self,
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        _event_source: Window,
        app: &mut dyn App,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) | Event::Cmd(CMD_LOST_PRIMARY_FOCUS) => {
                window.invalidate_render(tree);
                false
            },
            Event::Key(Key::Char(' ')) => {
                if window.actual_is_enabled(tree) {
                    CheckBox::click(tree, window, app);
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
