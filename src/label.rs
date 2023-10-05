use crate::{prop_string_measure, widget};
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Point, Rect, Vector, text_width, Key};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree};

pub struct Label {
    text: String,
}

impl<State: ?Sized> WidgetData<State> for Label { }

impl Label {
    pub fn new() -> Self {
        Label { text: String::new() }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(12));
            palette.set(1, Left(13));
        });
    }

    widget!(LabelWidget; init_palette);
    prop_string_measure!(text);
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
        let color_text = window.color(tree, 0);
        let color_label = window.color(tree, 1);
        let data = window.data::<Label>(tree);
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
            let next_focused = window.actual_next_focused(tree);
            next_focused.set_focused_primary(tree, true);
            return true;
        }
        false
    }

    fn post_process(&self) -> bool { true }
}
