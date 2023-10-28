use crate::widget;
use alloc::boxed::Box;
use tuifw_screen_base::{Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App};

widget! {
    #[widget(StackPanelWidget)]
    pub struct StackPanel {
        #[property(copy, measure)]
        vertical: bool,
    }
}

#[derive(Clone, Default)]
pub struct StackPanelWidget;

impl Widget for StackPanelWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(StackPanel {
            vertical: true
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        StackPanel::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        _tree: &WindowTree,
        _window: Window,
        _rp: &mut RenderPort,
        _app: &mut dyn App,
    ) { }

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        available_width: Option<i16>,
        available_height: Option<i16>,
        app: &mut dyn App,
    ) -> Vector {
        let vertical = window.data::<StackPanel>(tree).vertical;
        if vertical {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, available_width, None, app);
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
                    child.measure(tree, None, available_height, app);
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
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        app: &mut dyn App,
    ) -> Vector {
        let vertical = window.data::<StackPanel>(tree).vertical;
        if vertical {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_size = Vector { x: final_inner_bounds.w(), y: child.desired_size(tree).y };
                    child.arrange(tree, Rect { tl: pos, size: child_size }, app);
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
                    child.arrange(tree, Rect { tl: pos, size: child_size }, app);
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
        _tree: &mut WindowTree,
        _window: Window,
        _event: Event,
        _event_source: Window,
        _app: &mut dyn App,
    ) -> bool {
        false
    }
}
