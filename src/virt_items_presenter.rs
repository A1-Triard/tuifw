use crate::{widget, StaticText, StackPanel};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::max;
use either::Right;
use tuifw_screen_base::{Rect, Vector, Error, Fg, Bg};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Timer, Data};
use tuifw_window::Visibility;

pub const CMD_VIRT_ITEMS_PRESENTER_BIND: u16 = 150;
pub const CMD_VIRT_ITEMS_PRESENTER_UNBIND: u16 = 151;

widget! {
    #[widget(VirtItemsPresenterWidget, init=init)]
    pub struct VirtItemsPresenter {
        #[property(ref, on_changed=update)]
        items: Vec<Box<dyn Data>>,
        #[property(copy, measure, on_changed=on_templates_changed)]
        vertical: bool,
        #[property(copy, on_changed=on_templates_changed)]
        item_template: Option<Window>,
        update_timer: Option<Timer>,
        templates_changed: bool,
        error: bool,
        max_items_count: usize,
    }
}

impl VirtItemsPresenter {
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
        window.data_mut::<VirtItemsPresenter>(tree).error = true;
        let error_text = Self::error_text(tree, window);
        StaticText::set_text(tree, error_text, error.to_string());
        error_text.set_visibility(tree, Visibility::Visible);
    }

    fn on_templates_changed(tree: &mut WindowTree, window: Window) {
        window.data_mut::<VirtItemsPresenter>(tree).templates_changed = true;
        Self::update(tree, window);
    }
        
    fn update(tree: &mut WindowTree, window: Window) {
        let update_timer = Timer::new(tree, 0, Box::new(move |tree, app| {
            let data = window.data_mut::<VirtItemsPresenter>(tree);
            data.update_timer = None;
            if data.error { return; }
            if data.templates_changed {
                data.templates_changed = false;
                if let Some(panel) = Self::panel(tree, window) {
                    if let Some(first_item_window) = panel.first_child(tree) {
                        let mut item_window = first_item_window;
                        loop {
                            item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_UNBIND), app);
                            item_window.set_source(tree, None);
                            item_window = item_window.next(tree);
                            if item_window == first_item_window { break; }
                        }
                    }
                    panel.drop_window(tree, app);
                }
                let data = window.data::<VirtItemsPresenter>(tree);
                if let Some(item_template) = data.item_template {
                    let vertical = data.vertical;
                    let max_items_count = data.max_items_count;
                    let panel = match StackPanel::new(tree, Some(window), None) {
                        Ok(panel) => panel,
                        Err(error) => return Self::show_error(tree, window, error),
                    };
                    StackPanel::set_vertical(tree, panel, vertical);
                    let mut prev = None;
                    for item_index in 0 .. max_items_count {
                        let item = window.data::<VirtItemsPresenter>(tree).items.get(item_index).cloned();
                        let item_window = match item_template.new_instance(tree, Some(panel), prev) {
                            Ok(item_window) => item_window,
                            Err(error) => return Self::show_error(tree, window, error),
                        };
                        if let Some(item) = item {
                            item_window.set_source(tree, Some(item));
                            item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND), app);
                        } else {
                            item_window.set_visibility(tree, Visibility::Collapsed);
                        }
                        prev = Some(item_window);
                    }
                }
            } else if let Some(panel) = Self::panel(tree, window) {
                if let Some(first_item_window) = panel.first_child(tree) {
                    let mut item_index = 0;
                    let mut item_window = first_item_window;
                    loop {
                        let item = window.data::<VirtItemsPresenter>(tree).items.get(item_index).cloned();
                        if let Some(item) = item {
                            if item_window.source(tree).is_some() {
                                item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_UNBIND), app);
                            } else {
                                item_window.set_visibility(tree, Visibility::Visible);
                            }
                            item_window.set_source(tree, Some(item));
                            item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND), app);
                        } else if item_window.source(tree).is_some() {
                            item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_UNBIND), app);
                            item_window.set_visibility(tree, Visibility::Collapsed);
                        }
                        item_index += 1;
                        item_window = item_window.next(tree);
                        if item_window == first_item_window { break; }
                    };
                }
            }
        }));
        let data = window.data_mut::<VirtItemsPresenter>(tree);
        if let Some(old_update_timer) = data.update_timer.replace(update_timer) {
            old_update_timer.drop_timer(tree);
        }
    }
}

#[derive(Clone, Default)]
pub struct VirtItemsPresenterWidget;

impl Widget for VirtItemsPresenterWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(VirtItemsPresenter {
            vertical: true,
            item_template: None,
            update_timer: None,
            error: false,
            items: Vec::new(),
            templates_changed: false,
            max_items_count: 1,
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        VirtItemsPresenter::clone(tree, source, dest, clone_window);
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
        let vertical = window.data::<VirtItemsPresenter>(tree).vertical;
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
        if vertical {
            size.y = available_height.unwrap_or(1);
        } else {
            size.x = available_width.unwrap_or(1);
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
        let data = window.data_mut::<VirtItemsPresenter>(tree);
        let vertical = data.vertical;
        let max_items_count = if vertical {
            size.y = final_inner_bounds.h();
            usize::from(size.y as u16)
        } else {
            size.x = final_inner_bounds.w();
            usize::from(size.x as u16)
        };
        let max_items_count = max(1, max_items_count);
        if data.max_items_count != max_items_count {
            data.max_items_count = max_items_count;
            VirtItemsPresenter::on_templates_changed(tree, window);
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
