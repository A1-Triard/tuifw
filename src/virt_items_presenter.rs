use crate::{widget, StaticText, StackPanel};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::min;
use core::mem::size_of;
use either::{Left, Right};
use tuifw_screen_base::{Rect, Vector, Error, Fg, Bg, Thickness};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Timer, Data};
use tuifw_window::Visibility;

pub const CMD_VIRT_ITEMS_PRESENTER_BIND: u16 = 150;
pub const CMD_VIRT_ITEMS_PRESENTER_UNBIND: u16 = 151;

widget! {
    #[widget(VirtItemsPresenterWidget, init=init)]
    pub struct VirtItemsPresenter {
        #[property(ref, arrange, on_changed=update)]
        items: Vec<Box<dyn Data>>,
        #[property(copy, measure, on_changed=on_templates_changed)]
        vertical: bool,
        #[property(copy, on_changed=on_templates_changed)]
        item_template: Option<Window>,
        #[property(copy, on_changed=update)]
        offset: i16,
        viewport: i16,
        item_size: i16,
        update_timer: Option<Timer>,
        templates_changed: bool,
        error: bool,
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
            let skip_items = (data.offset as u16 / data.item_size as u16).saturating_sub(1);
            let panel_margin = data.offset as u16 - skip_items * data.item_size as u16;
            let take_items =
                (data.viewport as u16 as u32 + panel_margin as u32).div_ceil(data.item_size as u16 as u32)
                + 1
            ;
            let panel_margin = Thickness::new(0, -i32::from(panel_margin), 0, 0);
            let items_range =
                min(data.items.len(), usize::from(skip_items))
                ..
                min(
                    data.items.len(),
                    if size_of::<usize>() >= size_of::<u32>() {
                        usize::try_from(take_items + u32::from(skip_items)).unwrap()
                    } else {
                        usize::try_from(min(
                            u32::try_from(usize::MAX).unwrap(),
                            take_items + u32::from(skip_items)
                        )).unwrap()
                    }
                )
            ;
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
                    let panel = match StackPanel::new(tree, Some(window), None) {
                        Ok(panel) => panel,
                        Err(error) => return Self::show_error(tree, window, error),
                    };
                    StackPanel::set_vertical(tree, panel, vertical);
                    panel.set_margin(tree, panel_margin);
                    let mut prev = None;
                    for item_index in items_range {
                        let item = window.data::<VirtItemsPresenter>(tree).items[item_index].clone();
                        let item_window = match item_template.new_instance(tree, Some(panel), prev) {
                            Ok(item_window) => item_window,
                            Err(error) => return Self::show_error(tree, window, error),
                        };
                        item_window.set_source(tree, Some(item));
                        item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND), app);
                        prev = Some(item_window);
                    }
                }
            } else if let Some(panel) = Self::panel(tree, window) {
                panel.set_margin(tree, panel_margin);
                let new_tail_or_drop_tail = if let Some(first_item_window) = panel.first_child(tree) {
                    let mut item_window = first_item_window;
                    let mut item_index = items_range.start;
                    loop {
                        if item_index == items_range.end { break Left((item_window, first_item_window)); }
                        let item = window.data::<VirtItemsPresenter>(tree).items[item_index].clone();
                        item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_UNBIND), app);
                        item_window.set_source(tree, Some(item));
                        item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND), app);
                        item_index += 1;
                        let prev = item_window;
                        item_window = item_window.next(tree);
                        if item_window == first_item_window { break Right((Some(prev), item_index)); }
                    }
                } else {
                    Right((None, items_range.start))
                };
                match new_tail_or_drop_tail {
                    Right((mut prev, mut item_index)) => {
                        let item_template = window.data::<VirtItemsPresenter>(tree).item_template.unwrap();
                        loop {
                            if item_index == items_range.end { break; }
                            let item = window.data::<VirtItemsPresenter>(tree).items[item_index].clone();
                            let item_window = match item_template.new_instance(tree, Some(panel), prev) {
                                Ok(item_window) => item_window,
                                Err(error) => return Self::show_error(tree, window, error),
                            };
                            item_window.set_source(tree, Some(item));
                            item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND), app);
                            prev = Some(item_window);
                            item_index += 1;
                        }
                    },
                    Left((mut item_window, first_item_window)) => {
                        loop {
                            item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_UNBIND), app);
                            item_window.set_source(tree, None);
                            item_window = item_window.next(tree);
                            if item_window == first_item_window { break; }
                        }
                    }
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
            offset: 0,
            viewport: 0,
            item_size: 1,
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
        let item_size = VirtItemsPresenter::panel(tree, window)
            .and_then(|x| x.first_child(tree))
            .map_or(Vector { x: 1, y: 1 }, |item| item.render_bounds(tree).size)
        ;
        let data = window.data::<VirtItemsPresenter>(tree);
        let vertical = data.vertical;
        if vertical {
            size.y = final_inner_bounds.h();
            let data = window.data_mut::<VirtItemsPresenter>(tree);
            if data.viewport != size.y {
                data.viewport = size.y;
                VirtItemsPresenter::update(tree, window);
            }
            let data = window.data_mut::<VirtItemsPresenter>(tree);
            if data.item_size != item_size.y {
                data.item_size = item_size.y;
                VirtItemsPresenter::update(tree, window);
            }
        } else {
            size.x = final_inner_bounds.w();
            let data = window.data_mut::<VirtItemsPresenter>(tree);
            if data.viewport != size.x {
                data.viewport = size.x;
                VirtItemsPresenter::update(tree, window);
            }
            let data = window.data_mut::<VirtItemsPresenter>(tree);
            if data.item_size != item_size.x {
                data.item_size = item_size.x;
                VirtItemsPresenter::update(tree, window);
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
