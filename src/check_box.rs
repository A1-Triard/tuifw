use crate::widget;
use alloc::boxed::Box;
use alloc::string::String;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Key, Point, Rect, Vector, Error};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Color};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS, label_width, label};
use tuifw_window::{COLOR_LABEL, COLOR_HOTKEY, COLOR_DISABLED};

pub const CMD_CHECK_BOX_CLICK: u16 = 110;

widget! {
    #[widget(CheckBoxWidget, init=init_palette)]
    pub struct CheckBox {
        #[property(copy, render)]
        is_on: bool,
        #[property(copy)]
        cmd: u16,
        #[property(str, measure)]
        text: String,
    }
}

impl CheckBox {
    fn init_palette(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        window.palette_mut(tree, |palette| {
            palette.set(0, Color::Palette(COLOR_LABEL));
            palette.set(1, Color::Palette(COLOR_HOTKEY));
            palette.set(2, Color::Palette(COLOR_DISABLED));
        });
        Ok(())
    }

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

impl_supports_interfaces!(CheckBoxWidget);

impl Widget for CheckBoxWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(CheckBox {
            is_on: false,
            cmd: CMD_CHECK_BOX_CLICK,
            text: String::new(),
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        CheckBox::clone(tree, source, dest, clone_window);
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
