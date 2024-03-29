use crate::widget;
use alloc::boxed::Box;
use alloc::string::String;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Key, Point, Rect, Vector, Error};
use tuifw_window::{Event, RenderPort, Timer, Widget, WidgetData, Window, WindowTree, label_width, label};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS, App, Color};
use tuifw_window::{CMD_GOT_SECONDARY_FOCUS, CMD_LOST_SECONDARY_FOCUS};
use tuifw_window::{COLOR_BUTTON, COLOR_HOTKEY, COLOR_DISABLED, COLOR_BUTTON_FOCUSED};
use tuifw_window::{COLOR_BUTTON_FOCUSED_HOTKEY, COLOR_BUTTON_FOCUSED_DISABLED, COLOR_BUTTON_PRESSED};

pub const CMD_BUTTON_CLICK: u16 = 100;

widget! {
    #[widget(ButtonWidget, init=init_palette, drop=drop_timers)]
    pub struct Button {
        #[property(str, measure)]
        text: String,
        click_timer: Option<Timer>,
        release_timer: Option<Timer>,
        #[property(copy)]
        cmd: u16,
    }
}

impl Button {
    fn init_palette(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        window.palette_mut(tree, |palette| {
            palette.set(0, Color::Palette(COLOR_BUTTON));
            palette.set(1, Color::Palette(COLOR_HOTKEY));
            palette.set(2, Color::Palette(COLOR_DISABLED));
            palette.set(3, Color::Palette(COLOR_BUTTON_FOCUSED));
            palette.set(4, Color::Palette(COLOR_BUTTON_FOCUSED_HOTKEY));
            palette.set(5, Color::Palette(COLOR_BUTTON_FOCUSED_DISABLED));
            palette.set(6, Color::Palette(COLOR_BUTTON_PRESSED));
        });
        Ok(())
    }

    fn drop_timers(&mut self, tree: &mut WindowTree, _app: &mut dyn App) {
        if let Some(release_timer) = self.release_timer.take() {
            release_timer.drop_timer(tree);
        }
        if let Some(click_timer) = self.click_timer.take() {
            click_timer.drop_timer(tree);
        }
    }

    fn click(tree: &mut WindowTree, window: Window) {
        let click_timer = Timer::new(tree, 0, Box::new(move |tree, app| {
            let data = window.data_mut::<Button>(tree);
            data.click_timer = None;
            if window.actual_is_enabled(tree) {
                let release_timer = Timer::new(tree, 100, Box::new(move |tree, _app| {
                    let data = window.data_mut::<Button>(tree);
                    data.release_timer = None;
                    window.invalidate_render(tree);
                }));
                let data = window.data_mut::<Button>(tree);
                let cmd = data.cmd;
                if let Some(old_release_timer) = data.release_timer.replace(release_timer) {
                    old_release_timer.drop_timer(tree);
                }
                window.invalidate_render(tree);
                window.raise(tree, Event::Cmd(cmd), app);
            }
        }));
        let data = window.data_mut::<Button>(tree);
        if let Some(old_click_timer) = data.click_timer.replace(click_timer) {
            old_click_timer.drop_timer(tree);
        }
    }
}

#[derive(Clone, Default)]
pub struct ButtonWidget;

impl_supports_interfaces!(ButtonWidget);

impl Widget for ButtonWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(Button {
            text: String::new(),
            release_timer: None,
            click_timer: None,
            cmd: CMD_BUTTON_CLICK,
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        Button::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
    ) {
        let bounds = window.inner_bounds(tree);
        let focused = window.is_focused(tree);
        let is_enabled = window.actual_is_enabled(tree);
        let data = window.data::<Button>(tree);
        let pressed = data.release_timer.is_some();
        let (color, color_hotkey) = if pressed {
            (6, 6)
        } else if focused {
            if !is_enabled { (5, 5) } else { (3, 4) }
        } else {
            if !is_enabled { (2, 2) } else { (0, 1) }
        };
        let color = window.color(tree, color);
        let color_hotkey = window.color(tree, color_hotkey);
        rp.fill_bg(color.1);
        rp.label(Point { x: 1, y: 0 }, color, color_hotkey, &data.text);
        rp.text(Point { x: 0, y: 0 }, color, if pressed { " " } else { "[" });
        rp.text(Point { x: bounds.r_inner(), y: 0 }, color, if pressed { " " } else { "]" });
    }

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _app: &mut dyn App,
    ) -> Vector {
        let data = window.data::<Button>(tree);
        Vector { x: label_width(&data.text).wrapping_add(2), y: 1 }
    }

    fn arrange(
        &self,
        _tree: &mut WindowTree,
        _window: Window,
        final_inner_bounds: Rect,
        _app: &mut dyn App,
    ) -> Vector {
        final_inner_bounds.size
    }

    fn secondary_focusable(&self) -> bool { true }

    fn update(
        &self,
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        _event_source: Window,
        _app: &mut dyn App,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) | Event::Cmd(CMD_LOST_PRIMARY_FOCUS) |
            Event::Cmd(CMD_GOT_SECONDARY_FOCUS) | Event::Cmd(CMD_LOST_SECONDARY_FOCUS) => {
                window.invalidate_render(tree);
                false
            },
            Event::Key(Key::Enter) => {
                if window.actual_is_enabled(tree) {
                    Button::click(tree, window);
                    true
                } else {
                    false
                }
            },
            Event::PostProcessKey(Key::Alt(c)) | Event::PostProcessKey(Key::Char(c)) => {
                if window.actual_is_enabled(tree) {
                    let data = window.data_mut::<Button>(tree);
                    let label = label(&data.text);
                    if Some(c) == label {
                        Button::click(tree, window);
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
