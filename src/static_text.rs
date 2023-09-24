use alloc::boxed::Box;
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Error, Point, Rect, Screen, Vector};
use tuifw_window::{Event, RenderPort, Widget, Window, WindowTree};
use unicode_width::UnicodeWidthChar;

pub struct StaticText {
    text: String,
}

impl StaticText {
    pub fn new() -> Self {
        StaticText { text: String::new() }
    }

    fn set_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| palette.set(0, Left(10)));
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
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<State>, Error> {
        let mut tree = WindowTree::new(screen, Box::new(StaticTextWidget), Box::new(self))?;
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

#[derive(Clone)]
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
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<StaticText>(tree);
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }

    fn update(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        _event: Event,
        _preview: bool,
        _state: &mut State,
    ) -> bool {
        false
    }
}
