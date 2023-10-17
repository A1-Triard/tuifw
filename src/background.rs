use crate::{prop_string_render, prop_value_render, widget};
use alloc::string::{String, ToString};
use either::Left;
use tuifw_screen_base::{Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, State};
use tuifw_window::COLOR_BACKGROUND;

pub struct Background {
    pattern_even: String,
    pattern_odd: String,
    show_pattern: bool,
}

impl WidgetData for Background { }

impl Background {
    pub fn new() -> Self {
        Background { pattern_even: "░".to_string(), pattern_odd: "░".to_string(), show_pattern: false }
    }

    fn init_palette(tree: &mut WindowTree, window: Window) {
        window.palette_mut(tree, |palette| palette.set(0, Left(COLOR_BACKGROUND)));
    }

    widget!(BackgroundWidget; init_palette);
    prop_value_render!(show_pattern: bool);
    prop_string_render!(pattern_even);
    prop_string_render!(pattern_odd);
}

impl Default for Background {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct BackgroundWidget;

impl Widget for BackgroundWidget {
    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _state: &mut dyn State,
    ) {
        let color = window.color(tree, 0);
        let data = window.data::<Background>(tree);
        rp.fill(|rp, p| rp.text(
            p,
            color,
            if !data.show_pattern {
                " "
            } else if p.x % 2 == 0 {
                &data.pattern_even
            } else {
                &data.pattern_odd
            }
        ));
    }

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut dyn State,
    ) -> Vector {
        let mut size = Vector::null();
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.measure(tree, available_width, available_height, state);
                size = size.max(child.desired_size(tree));
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        size
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        state: &mut dyn State,
    ) -> Vector {
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.arrange(tree, final_inner_bounds, state);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        final_inner_bounds.size
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
