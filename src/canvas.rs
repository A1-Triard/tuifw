use crate::widget;
use alloc::boxed::Box;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Rect, Vector, Point};
use tuifw_window::{Event, Layout, RenderPort, Widget, WidgetData, Window, WindowTree, App};

#[derive(Clone)]
struct CanvasLayout {
    tl: Point,
}

impl Layout for CanvasLayout { }

impl Default for CanvasLayout {
    fn default() -> Self { CanvasLayout { tl: Point { x: 0, y: 0 } } }
}

widget! {
    #[widget(CanvasWidget)]
    pub struct Canvas { }
}

impl Canvas {
    pub fn tl(tree: &WindowTree, window: Window) -> Point {
        window.layout::<CanvasLayout>(tree).map(|x| x.tl).unwrap_or(Point { x: 0, y: 0 })
    }

    pub fn set_tl(tree: &mut WindowTree, window: Window, value: Point) {
        window.layout_mut(tree, |x: &mut CanvasLayout| x.tl = value);
    }
}

#[derive(Clone, Default)]
pub struct CanvasWidget;

impl_supports_interfaces!(CanvasWidget);

impl Widget for CanvasWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(Canvas { })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        Canvas::clone(tree, source, dest, clone_window);
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
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.measure(tree, None, None, app);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        Vector { x: available_width.unwrap_or(1), y: available_height.unwrap_or(1) }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        app: &mut dyn App,
    ) -> Vector {
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                let tl = Canvas::tl(tree, child);
                let size = child.desired_size(tree);
                child.arrange(tree, Rect { tl, size }, app);
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
        _app: &mut dyn App,
    ) -> bool {
        false
    }
}
