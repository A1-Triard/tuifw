use crate::{prop_string_measure, prop_string_render, prop_value, prop_value_render, widget};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use either::Left;
use tuifw_screen_base::{Key, Point, Rect, Vector, text_width};
use tuifw_window::{Event, RenderPort, Timer, Widget, WidgetData, Window, WindowTree};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS};
use tuifw_window::{CMD_GOT_SECONDARY_FOCUS, CMD_LOST_SECONDARY_FOCUS};

pub const CMD_CLICK: u16 = 100;

pub struct Button {
    border_left: String,
    border_right: String,
    text: String,
    release_timer: Option<Timer>,
    cmd: u16,
    is_enabled: bool,
}

impl<State: ?Sized> WidgetData<State> for Button {
    fn drop_widget_data(&mut self, tree: &mut WindowTree<State>, _state: &mut State) {
        if let Some(release_timer) = self.release_timer.take() {
            release_timer.drop_timer(tree);
        }
    }
}

impl Button {
    pub fn new() -> Self {
        Button {
            border_left: "[".to_string(),
            border_right: "]".to_string(),
            text: String::new(),
            release_timer: None,
            cmd: CMD_CLICK,
            is_enabled: true,
        }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(15));
            palette.set(1, Left(16));
            palette.set(2, Left(17));
            palette.set(3, Left(18));
        });
    }

    widget!(ButtonWidget; init_palette);
    prop_string_measure!(text);
    prop_string_render!(border_left);
    prop_string_render!(border_right);
    prop_value!(cmd: u16);
    prop_value_render!(is_enabled: bool);
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
        let data = window.data::<Button>(tree);
        let pressed = data.release_timer.is_some();
        let color = if !data.is_enabled { 3 } else if pressed { 2 } else if focused { 1 } else { 0 };
        let color = window.color(tree, color);
        rp.out(Point { x: 1, y: 0 }, color.0, color.1, &data.text);
        rp.out(
            Point { x: 0, y: 0 },
            color.0,
            color.1,
            if pressed { " " } else { &data.border_left }
        );
        rp.out(
            Point { x: bounds.r_inner(), y: 0 },
            color.0,
            color.1,
            if pressed { " " } else { &data.border_right }
        );
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
        state: &mut State,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) | Event::Cmd(CMD_LOST_PRIMARY_FOCUS) |
            Event::Cmd(CMD_GOT_SECONDARY_FOCUS) | Event::Cmd(CMD_LOST_SECONDARY_FOCUS) => {
                window.invalidate_render(tree);
                false
            },
            Event::Key(_, Key::Enter) => {
                let data = window.data_mut::<Button>(tree);
                if data.is_enabled {
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
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }
}
