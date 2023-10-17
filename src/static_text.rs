use crate::{prop_string_measure, widget};
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Point, Rect, Vector, text_width};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, State};
use tuifw_window::{COLOR_TEXT, COLOR_DISABLED};

pub struct StaticText {
    text: String,
}

impl WidgetData for StaticText { }

impl StaticText {
    pub fn new() -> Self {
        StaticText { text: String::new() }
    }

    fn init_palette(tree: &mut WindowTree, window: Window) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(COLOR_TEXT));
            palette.set(1, Left(COLOR_DISABLED));
        });
    }

    widget!(StaticTextWidget; init_palette);
    prop_string_measure!(text);
}

impl Default for StaticText {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct StaticTextWidget;

impl Widget for StaticTextWidget {
    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _state: &mut dyn State,
    ) {
        let is_enabled = window.actual_is_enabled(tree);
        let color = window.color(tree, if is_enabled { 0 } else { 1 });
        let data = window.data::<StaticText>(tree);
        rp.text(Point { x: 0, y: 0 }, color, &data.text);
    }

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut dyn State,
    ) -> Vector {
        let data = window.data::<StaticText>(tree);
        Vector { x: text_width(&data.text), y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        _final_inner_bounds: Rect,
        _state: &mut dyn State,
    ) -> Vector {
        let data = window.data::<StaticText>(tree);
        Vector { x: text_width(&data.text), y: 1 }
    }

    fn update(
        &self,
        _tree: &mut WindowTree,
        _window: Window,
        _event: Event,
        _event_source: Window,
        _state: &mut dyn State,
    ) -> bool {
        false
    }
}
