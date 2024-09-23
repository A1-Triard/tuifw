use alloc::boxed::Box;
use dynamic_cast::impl_supports_interfaces;
use tuifw::widget;
use tuifw_screen::{Point, Rect, Vector, Fg, Bg};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App};

widget! {
    #[widget(FloatingFrameWidget)]
    pub struct FloatingFrame { }
}

#[derive(Clone, Default)]
struct FloatingFrameWidget;

impl_supports_interfaces!(FloatingFrameWidget);

impl Widget for FloatingFrameWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(FloatingFrame { })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        FloatingFrame::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        _tree: &WindowTree,
        _window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
    ) {
        rp.text(Point { x: 0, y: 0 }, (Fg::Green, Bg::None), "╔═══════════╗");
        rp.text(Point { x: 0, y: 1 }, (Fg::Green, Bg::None), "║     ↑     ║");
        rp.text(Point { x: 0, y: 2 }, (Fg::Green, Bg::None), "║     k     ║");
        rp.text(Point { x: 0, y: 3 }, (Fg::Green, Bg::None), "║ ←h     l→ ║");
        rp.text(Point { x: 0, y: 4 }, (Fg::Green, Bg::None), "║     j     ║");
        rp.text(Point { x: 0, y: 5 }, (Fg::Green, Bg::None), "║     ↓     ║");
        rp.text(Point { x: 0, y: 6 }, (Fg::Green, Bg::None), "╚═══════════╝");
    }

    fn measure(
        &self,
        _tree: &mut WindowTree,
        _window: Window,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _app: &mut dyn App,
    ) -> Vector {
        Vector { x: 13, y: 7 }
    }

    fn arrange(
        &self,
        _tree: &mut WindowTree,
        _window: Window,
        _final_inner_bounds: Rect,
        _app: &mut dyn App,
    ) -> Vector {
        Vector { x: 13, y: 7 }
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
