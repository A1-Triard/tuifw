use crate::widget;
use alloc::boxed::Box;
use alloc::string::String;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Point, Rect, Vector, text_width, Error};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Color};
use tuifw_window::{COLOR_LABEL, COLOR_DISABLED};

widget! {
    #[widget(StaticTextWidget, init=init_palette)]
    pub struct StaticText {
        #[property(str, measure)]
        text: String,
    }
}

impl StaticText {
    fn init_palette(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        window.palette_mut(tree, |palette| {
            palette.set(0, Color::Palette(COLOR_LABEL));
            palette.set(1, Color::Palette(COLOR_DISABLED));
        });
        Ok(())
    }
}

#[derive(Clone, Default)]
struct StaticTextWidget;

impl_supports_interfaces!(StaticTextWidget);

impl Widget for StaticTextWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(StaticText {
            text: String::new()
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        StaticText::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
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
        _app: &mut dyn App,
    ) -> Vector {
        let data = window.data::<StaticText>(tree);
        Vector { x: text_width(&data.text), y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        _final_inner_bounds: Rect,
        _app: &mut dyn App,
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
        _app: &mut dyn App,
    ) -> bool {
        false
    }
}
