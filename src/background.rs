use crate::{prop_string_render, prop_value_render, widget};
use alloc::string::{String, ToString};
use either::Left;
use tuifw_screen_base::{Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree};

pub struct Background {
    pattern_even: String,
    pattern_odd: String,
    show_pattern: bool,
}

impl<State: ?Sized> WidgetData<State> for Background { }

impl Background {
    pub fn new() -> Self {
        Background { pattern_even: "░".to_string(), pattern_odd: "░".to_string(), show_pattern: false }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| palette.set(0, Left(11)));
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

impl<State: ?Sized> Widget<State> for BackgroundWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
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
        tree: &mut WindowTree<State>,
        window: Window<State>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
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
        tree: &mut WindowTree<State>,
        window: Window<State>,
        final_inner_bounds: Rect,
        state: &mut State,
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
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        _event: Event,
        _event_source: Window<State>,
        _state: &mut State,
    ) -> bool {
        false
    }
}
