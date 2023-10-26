use crate::widget;
use alloc::boxed::Box;
use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
use tuifw_screen_base::{Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Timer};

pub trait Data: Downcast + DynClone { }

impl_downcast!(Data);

clone_trait_object!(Data);

widget! {
    #[widget(ContentPresenterWidget)]
    pub struct ContentPresenter {
        #[property(obj, on_changed=update_tree)]
        content: Option<Box<dyn Data>>,
        #[property(value, on_changed=update_tree)]
        content_template: Option<Window>,
        update_tree_timer: Option<Timer>,
    }
}

impl ContentPresenter {
    fn update_tree(tree: &mut WindowTree, window: Window) {
        let update_tree_timer = Timer::new(tree, 0, Box::new(move |tree, app| {
            let data = window.data_mut::<ContentPresenter>(tree);
            data.update_tree_timer = None;
            if let Some(child) = window.first_child(tree) {
                child.drop_window(tree, app);
            }
            if let Some(content_template) = ContentPresenter::content_template(tree, window) {
                let _child = content_template.new_instance(tree, Some(window), None);
            }
        }));
        let data = window.data_mut::<ContentPresenter>(tree);
        if let Some(old_update_tree_timer) = data.update_tree_timer.replace(update_tree_timer) {
            old_update_tree_timer.drop_timer(tree);
        }
    }
}

#[derive(Clone, Default)]
pub struct ContentPresenterWidget;

impl Widget for ContentPresenterWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(ContentPresenter {
            content: None, content_template: None, update_tree_timer: None
        })
    }

    fn clone_data(&self, tree: &mut WindowTree, source: Window, dest: Window) {
        ContentPresenter::clone(tree, source, dest);
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
        if let Some(child) = window.first_child(tree) {
            child.measure(tree, available_width, available_height, app);
            child.desired_size(tree)
        } else {
            Vector::null()
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        app: &mut dyn App,
    ) -> Vector {
        if let Some(child) = window.first_child(tree) {
            child.arrange(tree, final_inner_bounds, app);
            child.render_bounds(tree).size
        } else {
            Vector::null()
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
