use crate::{prop_string_measure, prop_value, prop_value_render, widget};
use alloc::boxed::Box;
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Point, Rect, Vector, text_width, Key};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, Timer};

pub const CMD_LABEL_CLICK: u16 = 110;

pub struct Label {
    text: String,
    click_timer: Option<Timer>,
    cmd: u16,
    is_enabled: bool,
}

impl<State: ?Sized> WidgetData<State> for Label {
    fn drop_widget_data(&mut self, tree: &mut WindowTree<State>, _state: &mut State) {
        if let Some(click_timer) = self.click_timer.take() {
            click_timer.drop_timer(tree);
        }
    }
}

impl Label {
    pub fn new() -> Self {
        Label { text: String::new(), click_timer: None, cmd: CMD_LABEL_CLICK, is_enabled: true }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(12));
            palette.set(1, Left(13));
        });
    }

    widget!(LabelWidget; init_palette);
    prop_string_measure!(text);
    prop_value!(cmd: u16);
    prop_value_render!(is_enabled: bool);
}

impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct LabelWidget;

impl<State: ?Sized> Widget<State> for LabelWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.data::<Label>(tree);
        let color_text = window.color(tree, 0);
        let color_label = if data.is_enabled { window.color(tree, 1) } else { color_text };
        let mut text_parts = data.text.split("~");
        let text_1 = text_parts.next().unwrap_or("");
        let label = text_parts.next().unwrap_or("");
        let text_2 = text_parts.next().unwrap_or("");
        let text_1_width = text_width(text_1);
        let label_width = text_width(label);
        rp.out(Point { x: 0, y: 0 }, color_text.0, color_text.1, text_1);
        rp.out(Point { x: text_1_width, y: 0 }, color_label.0, color_label.1, label);
        rp.out(Point { x: text_1_width.wrapping_add(label_width), y: 0 }, color_text.0, color_text.1, text_2);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<Label>(tree);
        let mut text_parts = data.text.split("~");
        let text_1 = text_parts.next().unwrap_or("");
        let label = text_parts.next().unwrap_or("");
        let text_2 = text_parts.next().unwrap_or("");
        let text_1_width = text_width(text_1);
        let label_width = text_width(label);
        let text_2_width = text_width(text_2);
        Vector { x: text_1_width.wrapping_add(label_width).wrapping_add(text_2_width), y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<Label>(tree);
        let mut text_parts = data.text.split("~");
        let text_1 = text_parts.next().unwrap_or("");
        let label = text_parts.next().unwrap_or("");
        let text_2 = text_parts.next().unwrap_or("");
        let text_1_width = text_width(text_1);
        let label_width = text_width(label);
        let text_2_width = text_width(text_2);
        Vector { x: text_1_width.wrapping_add(label_width).wrapping_add(text_2_width), y: 1 }
    }

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        _state: &mut State,
    ) -> bool {
        let data = window.data::<Label>(tree);
        let label = data.text
            .split("~").skip(1).next().unwrap_or("")
            .chars().next().and_then(|x| x.to_lowercase().next());
        let Some(label) = label else { return false; };
        if event == Event::PostProcessKey(Key::Alt(label)) || event == Event::PostProcessKey(Key::Char(label)) {
            let data = window.data_mut::<Label>(tree);
            if data.is_enabled {
                let click_timer = Timer::new(tree, 0, Box::new(move |tree, state| {
                    let data = window.data_mut::<Label>(tree);
                    data.click_timer = None;
                    if data.is_enabled {
                        let data = window.data_mut::<Label>(tree);
                        let cmd = data.cmd;
                        window.raise(tree, Event::Cmd(cmd), state);
                        let next_focused = window.actual_next_focused(tree);
                        next_focused.set_focused_primary(tree, true);
                    }
                }));
                let data = window.data_mut::<Label>(tree);
                if let Some(old_click_timer) = data.click_timer.replace(click_timer) {
                    old_click_timer.drop_timer(tree);
                }
                return true;
            }
        }
        false
    }

    fn post_process(&self) -> bool { true }
}
