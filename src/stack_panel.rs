use crate::{prop_value_measure, widget};
use tuifw_screen_base::{Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree};

pub struct StackPanel {
    vertical: bool,
}

impl<State: ?Sized> WidgetData<State> for StackPanel { }

impl StackPanel {
    pub fn new() -> Self {
        StackPanel { vertical: true }
    }

    widget!(StackPanelWidget);
    prop_value_measure!(vertical: bool);
}

impl Default for StackPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct StackPanelWidget;

impl<State: ?Sized> Widget<State> for StackPanelWidget {
    fn render(
        &self,
        _tree: &WindowTree<State>,
        _window: Window<State>,
        _rp: &mut RenderPort,
        _state: &mut State,
    ) { }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        let vertical = window.data::<StackPanel>(tree).vertical;
        if vertical {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, available_width, None, state);
                    size += Vector { x: 0, y: child.desired_size(tree).y };
                    size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        } else {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, None, available_height, state);
                    size += Vector { x: child.desired_size(tree).x, y: 0 };
                    size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        let vertical = window.data::<StackPanel>(tree).vertical;
        if vertical {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_size = Vector { x: final_inner_bounds.w(), y: child.desired_size(tree).y };
                    child.arrange(tree, Rect { tl: pos, size: child_size }, state);
                    pos = pos.offset(Vector { x: 0, y: child_size.y });
                    size += Vector { x: 0, y: child_size.y };
                    size = size.max(Vector { x: child_size.x, y: 0 });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        } else {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_size = Vector { x: child.desired_size(tree).x, y: final_inner_bounds.h() };
                    child.arrange(tree, Rect { tl: pos, size: child_size }, state);
                    pos = pos.offset(Vector { x: child_size.x, y: 0 });
                    size += Vector { x: child_size.x, y: 0 };
                    size = size.max(Vector { x: 0, y: child_size.y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        }
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
