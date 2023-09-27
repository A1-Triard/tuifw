use alloc::boxed::Box;
use alloc::string::String;
use either::Left;
use timer_no_std::MonoClock;
use tuifw_screen_base::{Error, Point, Rect, Screen, Vector, text_width};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree};

pub struct StaticText {
    text: String,
}

impl<State: ?Sized> WidgetData<State> for StaticText { }

impl StaticText {
    pub fn new() -> Self {
        StaticText { text: String::new() }
    }

    fn set_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| palette.set(0, Left(11)));
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        let w = Window::new(tree, Box::new(StaticTextWidget), Box::new(self), parent, prev)?;
        Self::set_palette(tree, w);
        Ok(w)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>,
        clock: &MonoClock,
    ) -> Result<WindowTree<State>, Error> {
        let mut tree = WindowTree::new(screen, clock, Box::new(StaticTextWidget), Box::new(self))?;
        let w = tree.root();
        Self::set_palette(&mut tree, w);
        Ok(tree)
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn text_mut<State: ?Sized, T>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: impl FnOnce(&mut String) -> T
    ) -> T {
        let data = &mut window.data_mut::<StaticText>(tree).text;
        let res = value(data);
        window.invalidate_measure(tree);
        res
    }
}

impl Default for StaticText {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct StaticTextWidget;

impl<State: ?Sized> Widget<State> for StaticTextWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let color = window.color(tree, 0);
        let data = window.data::<StaticText>(tree);
        rp.out(Point { x: 0, y: 0 }, color.0, color.1, &data.text);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<StaticText>(tree);
        Vector { x: text_width(&data.text), y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<StaticText>(tree);
        Vector { x: text_width(&data.text), y: 1 }
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
