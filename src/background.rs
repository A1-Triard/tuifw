use crate::widget;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use either::Left;
use tuifw_screen_base::{Rect, Vector, Error};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App};
use tuifw_window::COLOR_BACKGROUND;

widget! {
    #[widget(BackgroundWidget, init=init_palette)]
    pub struct Background {
        #[property(ref, render)]
        pattern_even: String,
        #[property(ref, render)]
        pattern_odd: String,
        #[property(value, render)]
        show_pattern: bool,
    }
}

impl Background {
    fn init_palette(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        window.palette_mut(tree, |palette| palette.set(0, Left(COLOR_BACKGROUND)));
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct BackgroundWidget;

impl Widget for BackgroundWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(Background {
            pattern_even: "░".to_string(), pattern_odd: "░".to_string(), show_pattern: false
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        Background::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
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
        tree: &mut WindowTree,
        window: Window,
        available_width: Option<i16>,
        available_height: Option<i16>,
        app: &mut dyn App,
    ) -> Vector {
        let mut size = Vector::null();
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.measure(tree, available_width, available_height, app);
                size = size.max(child.desired_size(tree));
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        size
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
                child.arrange(tree, final_inner_bounds, app);
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
