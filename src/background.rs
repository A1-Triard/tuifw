use alloc::boxed::Box;
use alloc::string::{String, ToString};
use timer_no_std::MonoClock;
use tuifw_screen_base::{Error, Rect, Screen, Vector};
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

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        Window::new(tree, Box::new(BackgroundWidget), Box::new(self), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>,
        clock: &MonoClock,
    ) -> Result<WindowTree<State>, Error> {
        WindowTree::new(screen, clock, Box::new(BackgroundWidget), Box::new(self))
    }

    pub fn show_pattern(&self) -> bool { self.show_pattern }

    pub fn set_show_pattern<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, value: bool) {
        window.data_mut::<Background>(tree).show_pattern = value;
        window.invalidate_render(tree);
    }

    pub fn pattern_even(&self) -> &String { &self.pattern_even }

    pub fn pattern_even_mut<State: ?Sized, T>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        f: impl FnOnce(&mut String) -> T
    ) -> T {
        let value = &mut window.data_mut::<Background>(tree).pattern_even;
        let res = f(value);
        window.invalidate_render(tree);
        res
    }

    pub fn pattern_odd(&self) -> &String { &self.pattern_odd }

    pub fn pattern_odd_mut<State: ?Sized, T>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        f: impl FnOnce(&mut String) -> T
    ) -> T {
        let value = &mut window.data_mut::<Background>(tree).pattern_odd;
        let res = f(value);
        window.invalidate_render(tree);
        res
    }
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
        rp.fill(|rp, p| rp.out(
            p,
            color.0,
            color.1,
            if !data.show_pattern {
                " "
            } else if p.x % 2 == 0 {
                data.pattern_even()
            } else {
                data.pattern_odd()
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
