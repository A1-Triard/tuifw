use crate::{widget, StaticText};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use either::Right;
use tuifw_screen_base::{Rect, Vector, Error, Fg, Bg};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Timer, Data};
use tuifw_window::Visibility;

pub const CMD_ITEMS_PRESENTER_BIND: u16 = 140;
pub const CMD_ITEMS_PRESENTER_UNBIND: u16 = 141;

widget! {
    #[widget(ItemsPresenterWidget, init=init)]
    pub struct ItemsPresenter {
        #[property(ref, on_changed=update)]
        items: Vec<Box<dyn Data>>,
        #[property(copy, on_changed=on_templates_changed)]
        panel_template: Option<Window>,
        #[property(copy, on_changed=on_templates_changed)]
        item_template: Option<Window>,
        update_timer: Option<Timer>,
        templates_changed: bool,
        error: bool,
    }
}

impl ItemsPresenter {
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

    fn panel(tree: &WindowTree, window: Window) -> Option<Window> {
        let first_child = window.first_child(tree).unwrap();
        let second_child = first_child.next(tree);
        if first_child == second_child {
            None
        } else {
            Some(first_child)
        }
    }

    fn show_error(tree: &mut WindowTree, window: Window, error: Error) {
        window.data_mut::<ItemsPresenter>(tree).error = true;
        let error_text = Self::error_text(tree, window);
        StaticText::set_text(tree, error_text, error.to_string());
        error_text.set_visibility(tree, Visibility::Visible);
    }

    fn on_templates_changed(tree: &mut WindowTree, window: Window) {
        window.data_mut::<ItemsPresenter>(tree).templates_changed = true;
        Self::update(tree, window);
    }
        
    fn update(tree: &mut WindowTree, window: Window) {
        let update_timer = Timer::new(tree, 0, Box::new(move |tree, app| {
            let data = window.data_mut::<ItemsPresenter>(tree);
            data.update_timer = None;
            if data.error { return; }
            if data.templates_changed {
                data.templates_changed = false;
                if let Some(panel) = Self::panel(tree, window) {
                    if let Some(first_item_window) = panel.first_child(tree) {
                        let mut item_window = first_item_window;
                        loop {
                            item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_UNBIND), app);
                            item_window.set_source(tree, None);
                            item_window = item_window.next(tree);
                            if item_window == first_item_window { break; }
                        }
                    }
                    panel.drop_window(tree, app);
                }
                let data = window.data::<ItemsPresenter>(tree);
                if let Some(item_template) = data.item_template {
                    if let Some(panel_template) = data.panel_template {
                        let panel = match panel_template.new_instance(tree, Some(window), None) {
                            Ok(panel) => panel,
                            Err(error) => return Self::show_error(tree, window, error),
                        };
                        let mut item_index = 0;
                        let mut prev = None;
                        while
                            let Some(item) = window.data::<ItemsPresenter>(tree).items.get(item_index).cloned()
                        {
                            let item_window = match item_template.new_instance(tree, Some(panel), prev) {
                                Ok(item_window) => item_window,
                                Err(error) => return Self::show_error(tree, window, error),
                            };
                            item_window.set_source(tree, Some(item));
                            item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_BIND), app);
                            item_index += 1;
                            prev = Some(item_window);
                        }
                    }
                }
            } else if let Some(panel) = Self::panel(tree, window) {
                let mut last_item_window = None;
                let mut item_index = 0;
                if let Some(first_item_window) = panel.first_child(tree) {
                    let mut item_window = first_item_window;
                    let drop_tail = loop {
                        let item = window.data::<ItemsPresenter>(tree).items.get(item_index).cloned();
                        if let Some(item) = item {
                            item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_UNBIND), app);
                            item_window.set_source(tree, Some(item));
                            item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_BIND), app);
                            item_index += 1;
                        } else {
                            break true;
                        }
                        last_item_window = Some(item_window);
                        item_window = item_window.next(tree);
                        if item_window == first_item_window { break false; }
                    };
                    if drop_tail {
                        loop {
                            item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_UNBIND), app);
                            item_window.set_source(tree, None);
                            let next = item_window.next(tree);
                            item_window.drop_window(tree, app);
                            item_window = next;
                            if item_window == first_item_window { break; }
                        }
                    }
                }
                let mut prev = last_item_window;
                let item_template = window.data::<ItemsPresenter>(tree).item_template.unwrap();
                while let Some(item) = window.data::<ItemsPresenter>(tree).items.get(item_index).cloned() {
                    let item_window = match item_template.new_instance(tree, Some(panel), prev) {
                        Ok(item_window) => item_window,
                        Err(error) => return Self::show_error(tree, window, error),
                    };
                    item_window.set_source(tree, Some(item));
                    item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_BIND), app);
                    item_index += 1;
                    prev = Some(item_window);
                }
            }
        }));
        let data = window.data_mut::<ItemsPresenter>(tree);
        if let Some(old_update_timer) = data.update_timer.replace(update_timer) {
            old_update_timer.drop_timer(tree);
        }
    }
}

#[derive(Clone, Default)]
pub struct ItemsPresenterWidget;

impl Widget for ItemsPresenterWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(ItemsPresenter {
            panel_template: None,
            item_template: None,
            update_timer: None,
            error: false,
            items: Vec::new(),
            templates_changed: false,
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        ItemsPresenter::clone(tree, source, dest, clone_window);
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
