use crate::{prop_string_measure, prop_value, widget};
use alloc::boxed::Box;
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Key, Point, Rect, Vector, text_width};
use tuifw_window::{Event, RenderPort, Timer, Widget, WidgetData, Window, WindowTree};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS};
use tuifw_window::{CMD_GOT_SECONDARY_FOCUS, CMD_LOST_SECONDARY_FOCUS};

pub const CMD_BUTTON_CLICK: u16 = 100;

pub struct Button {
    text: String,
    click_timer: Option<Timer>,
    release_timer: Option<Timer>,
    cmd: u16,
}

impl<State: ?Sized> WidgetData<State> for Button {
    fn drop_widget_data(&mut self, tree: &mut WindowTree<State>, _state: &mut State) {
        if let Some(release_timer) = self.release_timer.take() {
            release_timer.drop_timer(tree);
        }
        if let Some(click_timer) = self.click_timer.take() {
            click_timer.drop_timer(tree);
        }
    }
}

impl Button {
    pub fn new() -> Self {
        Button {
            text: String::new(),
            release_timer: None,
            click_timer: None,
            cmd: CMD_BUTTON_CLICK,
        }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(12));
            palette.set(1, Left(13));
            palette.set(2, Left(18));
            palette.set(3, Left(19));
        });
    }

    widget!(ButtonWidget; init_palette);
    prop_string_measure!(text);
    prop_value!(cmd: u16);
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct ButtonWidget;

impl<State: ?Sized> Widget<State> for ButtonWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let bounds = window.inner_bounds(tree);
        let focused = window.is_focused(tree);
        let is_enabled = window.actual_is_enabled(tree);
        let data = window.data::<Button>(tree);
        let pressed = data.release_timer.is_some();
        let color = if !is_enabled { 1 } else if pressed { 3 } else if focused { 2 } else { 0 };
        let color = window.color(tree, color);
        rp.fill_bg(color.1);
        rp.text(Point { x: 1, y: 0 }, color, &data.text);
        rp.text(Point { x: 0, y: 0 }, color, if pressed { " " } else { "[" });
        rp.text(Point { x: bounds.r_inner(), y: 0 }, color, if pressed { " " } else { "]" });
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<Button>(tree);
        Vector { x: text_width(&data.text).wrapping_add(2), y: 1 }
    }

    fn arrange(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        final_inner_bounds.size
    }

    fn focusable(&self, _primary_focus: bool) -> bool { true }

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        _state: &mut State,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) | Event::Cmd(CMD_LOST_PRIMARY_FOCUS) |
            Event::Cmd(CMD_GOT_SECONDARY_FOCUS) | Event::Cmd(CMD_LOST_SECONDARY_FOCUS) => {
                window.invalidate_render(tree);
                false
            },
            Event::Key(Key::Enter) => {
                if window.actual_is_enabled(tree) {
                    let click_timer = Timer::new(tree, 0, Box::new(move |tree, state| {
                        let data = window.data_mut::<Button>(tree);
                        data.click_timer = None;
                        if window.is_enabled(tree) {
                            let release_timer = Timer::new(tree, 100, Box::new(move |tree, _state| {
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
                            window.raise(tree, Event::Cmd(cmd), state);
                        }
                    }));
                    let data = window.data_mut::<Button>(tree);
                    if let Some(old_click_timer) = data.click_timer.replace(click_timer) {
                        old_click_timer.drop_timer(tree);
                    }
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }
}
