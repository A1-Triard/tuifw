use crate::widget;
use alloc::boxed::Box;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Layout};

#[derive(Clone)]
struct StretchLayout {
    pub stretch: f32,
}

impl Layout for StretchLayout { }

impl Default for StretchLayout {
    fn default() -> Self {
        StretchLayout {
            stretch: 1.0,
        }
    }
}

widget! {
    #[widget(StretchPanelWidget)]
    pub struct StretchPanel {
        #[property(copy, measure)]
        vertical: bool,
    }
}

impl StretchPanel {
    pub fn stretch(tree: &WindowTree, window: Window) -> f32 {
        window.layout::<StretchLayout>(tree).map_or(1.0, |x| x.stretch)
    }

    pub fn set_stretch(tree: &mut WindowTree, window: Window, value: f32) {
        window.layout_mut(tree, |x: &mut StretchLayout| x.stretch = value);
    }
}

#[derive(Clone, Default)]
struct StretchPanelWidget;

impl_supports_interfaces!(StretchPanelWidget);

impl Widget for StretchPanelWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(StretchPanel {
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
        StretchPanel::clone(tree, source, dest, clone_window);
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
        let mut stretch_sum = 0.0_f32;
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                let child_stretch = StretchPanel::stretch(tree, child);
                stretch_sum += child_stretch;
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        let vertical = window.data::<StretchPanel>(tree).vertical;
        if vertical {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_stretch = StretchPanel::stretch(tree, child) / stretch_sum;
                    let child_height = available_height.map(|x|
                        (f32::from(x as u16) * child_stretch).min(f32::from(u16::MAX)) as u16 as i16
                    );
                    child.measure(tree, available_width, child_height, app);
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
                    let child_stretch = StretchPanel::stretch(tree, child) / stretch_sum;
                    let child_width = available_width.map(|x|
                        (f32::from(x as u16) * child_stretch).min(f32::from(u16::MAX)) as u16 as i16
                    );
                    child.measure(tree, child_width, available_height, app);
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
        let mut stretch_sum = 0.0_f32;
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                let child_stretch = StretchPanel::stretch(tree, child);
                stretch_sum += child_stretch;
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        let vertical = window.data::<StretchPanel>(tree).vertical;
        if vertical {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_stretch = StretchPanel::stretch(tree, child) / stretch_sum;
                    let child_height =
                        (f32::from(final_inner_bounds.h() as u16) * child_stretch)
                            .min(f32::from(u16::MAX)) as u16 as i16
                    ;
                    let child_size = Vector { x: final_inner_bounds.w(), y: child_height };
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
                    let child_stretch = StretchPanel::stretch(tree, child) / stretch_sum;
                    let child_width =
                        (f32::from(final_inner_bounds.w() as u16) * child_stretch)
                            .min(f32::from(u16::MAX)) as u16 as i16
                    ;
                    let child_size = Vector { x: child_width, y: final_inner_bounds.h() };
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
