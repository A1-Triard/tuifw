use crate::{widget, StaticText};
use alloc::boxed::Box;
use alloc::string::ToString;
use either::Right;
use tuifw_screen_base::{Rect, Vector, Error, Fg, Bg};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Timer, Data};
use tuifw_window::Visibility;

pub const CMD_CONTENT_PRESENTER_BIND: u16 = 130;
pub const CMD_CONTENT_PRESENTER_UNBIND: u16 = 131;

widget! {
    #[widget(ContentPresenterWidget, init=init)]
    pub struct ContentPresenter {
        #[property(ref, on_changed=update)]
        content: Option<Box<dyn Data>>,
        #[property(copy, on_changed=update)]
        content_template: Option<Window>,
        update_timer: Option<Timer>,
        error: bool,
    }
}

impl ContentPresenter {
    fn init(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        let error_text = StaticText::new(tree, Some(window), None)?;
        error_text.set_visibility(tree, Visibility::Collapsed);
        error_text.palette_mut(tree, |palette| {
            palette.set(0, Right((Fg::BrightRed, Bg::Blue)));
        });
        Ok(())
    }

    fn error_text(tree: &WindowTree, window: Window) -> Window {
        let first_child = window.first_child(tree).unwrap();
        first_child.next(tree)
    }

    fn content_window(tree: &WindowTree, window: Window) -> Option<Window> {
        let first_child = window.first_child(tree).unwrap();
        if first_child.next(tree) == first_child {
            None
        } else {
            Some(first_child)
        }
    }

    fn show_error(tree: &mut WindowTree, window: Window, error: Error) {
        window.data_mut::<ContentPresenter>(tree).error = true;
        let error_text = Self::error_text(tree, window);
        StaticText::set_text(tree, error_text, error.to_string());
        error_text.set_visibility(tree, Visibility::Visible);
    }

    fn update(tree: &mut WindowTree, window: Window) {
        let update_timer = Timer::new(tree, 0, Box::new(move |tree, app| {
            let data = window.data_mut::<ContentPresenter>(tree);
            data.update_timer = None;
            if data.error { return; }
            if let Some(content_window) = Self::content_window(tree, window) {
                content_window.raise(tree, Event::Cmd(CMD_CONTENT_PRESENTER_UNBIND), app);
                content_window.drop_window(tree, app);
            }
            if ContentPresenter::content(tree, window).is_some() {
                if let Some(content_template) = ContentPresenter::content_template(tree, window) {
                    let content_window = match content_template.new_instance(tree, Some(window), None) {
                        Ok(content_window) => content_window,
                        Err(error) => return Self::show_error(tree, window, error),
                    };
                    content_window.raise(tree, Event::Cmd(CMD_CONTENT_PRESENTER_BIND), app);
                }
            }
        }));
        let data = window.data_mut::<ContentPresenter>(tree);
        if let Some(old_update_timer) = data.update_timer.replace(update_timer) {
            old_update_timer.drop_timer(tree);
        }
    }
}

#[derive(Clone, Default)]
pub struct ContentPresenterWidget;

impl Widget for ContentPresenterWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(ContentPresenter {
            content: None, content_template: None, update_timer: None, error: false
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        ContentPresenter::clone(tree, source, dest, clone_window);
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
        let mut size = Vector::null();
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.arrange(tree, final_inner_bounds, app);
                size = size.max(child.render_bounds(tree).size);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        size
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
