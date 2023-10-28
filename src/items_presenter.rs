use crate::{widget, StaticText, StackPanel};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::mem::replace;
use either::Right;
use tuifw_screen_base::{Rect, Vector, Error, Fg, Bg};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Timer, Data};
use tuifw_window::Visibility;

pub const CMD_ITEMS_PRESENTER_BIND: u16 = 140;
pub const CMD_ITEMS_PRESENTER_UNBIND: u16 = 141;

enum Update {
    Refresh,
    Clear,
    Push(Box<dyn Data>),
    Insert(usize, Box<dyn Data>),
}

widget! {
    #[widget(ItemsPresenterWidget, init=init)]
    pub struct ItemsPresenter {
        #[property(copy, on_changed=refresh)]
        panel_template: Option<Window>,
        #[property(copy, on_changed=refresh)]
        item_template: Option<Window>,
        update_timer: Option<Timer>,
        items: Vec<Window>,
        update_queue: VecDeque<Update>,
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
        if let Some(panel_template) = ItemsPresenter::panel_template(tree, window) {
            panel_template.new_instance(tree, Some(window), None)?;
        } else {
            StackPanel::new(tree, Some(window), None)?;
        }
        Ok(())
    }

    pub fn clear(tree: &mut WindowTree, window: Window) {
        let data = window.data_mut::<ItemsPresenter>(tree);
        data.update_queue.push_back(Update::Clear);
        Self::update(tree, window);
    }

    pub fn push(tree: &mut WindowTree, window: Window, item: Box<dyn Data>) {
        let data = window.data_mut::<ItemsPresenter>(tree);
        data.update_queue.push_back(Update::Push(item));
        Self::update(tree, window);
    }

    pub fn insert(tree: &mut WindowTree, window: Window, index: usize, item: Box<dyn Data>) {
        let data = window.data_mut::<ItemsPresenter>(tree);
        data.update_queue.push_back(Update::Insert(index, item));
        Self::update(tree, window);
    }

    fn refresh(tree: &mut WindowTree, window: Window) {
        let data = window.data_mut::<ItemsPresenter>(tree);
        data.update_queue.push_back(Update::Refresh);
        Self::update(tree, window);
    }

    fn error_text(tree: &WindowTree, window: Window) -> Window {
        let first_child = window.first_child(tree).unwrap();
        let mut child = first_child;
        loop {
            let next = child.next(tree);
            if next == first_child { break child; }
            child = next;
        }
    }

    fn panel(tree: &WindowTree, window: Window) -> Window {
        window.first_child(tree).unwrap()
    }

    fn old_panel(tree: &WindowTree, window: Window) -> Window {
        let first_child = window.first_child(tree).unwrap();
        first_child.next(tree)
    }

    fn show_error(tree: &mut WindowTree, window: Window, error: Error) {
        window.data_mut::<ItemsPresenter>(tree).error = true;
        let error_text = Self::error_text(tree, window);
        StaticText::set_text(tree, error_text, error.to_string());
        error_text.set_visibility(tree, Visibility::Visible);
    }

    fn update(tree: &mut WindowTree, window: Window) {
        let update_timer = Timer::new(tree, 0, Box::new(move |tree, app| {
            let data = window.data_mut::<ItemsPresenter>(tree);
            data.update_timer = None;
            while let Some(update) = window.data_mut::<ItemsPresenter>(tree).update_queue.pop_front() {
                let data = window.data_mut::<ItemsPresenter>(tree);
                if data.error {
                    data.update_queue.clear();
                    return;
                }
                Self::do_update(tree, window, update, app);
            }
        }));
        let data = window.data_mut::<ItemsPresenter>(tree);
        if let Some(old_update_timer) = data.update_timer.replace(update_timer) {
            old_update_timer.drop_timer(tree);
        }
    }

    fn do_update(tree: &mut WindowTree, window: Window, update: Update, app: &mut dyn App) {
        match update {
            Update::Refresh => {
                let items_count = window.data::<ItemsPresenter>(tree).items.len();
                for i in 0 .. items_count {
                    let item_window = window.data::<ItemsPresenter>(tree).items[i];
                    item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_UNBIND), app);
                }
                let panel = if let Some(panel_template) = ItemsPresenter::panel_template(tree, window) {
                    match panel_template.new_instance(tree, Some(window), None) {
                        Ok(panel) => panel,
                        Err(error) => return Self::show_error(tree, window, error),
                    }
                } else {
                    match StackPanel::new(tree, Some(window), None) {
                        Ok(panel) => panel,
                        Err(error) => return Self::show_error(tree, window, error),
                    }
                };
                for i in (0 .. items_count).rev() {
                    let item_window = window.data::<ItemsPresenter>(tree).items[i];
                    let item = item_window.source_mut(tree, |source| source.take());
                    let item_window = if let Some(item_template) = ItemsPresenter::item_template(tree, window) {
                        match item_template.new_instance(tree, Some(panel), None) {
                            Ok(item_window) => item_window,
                            Err(error) => return Self::show_error(tree, window, error),
                        }
                    } else {
                        match StaticText::new(tree, Some(panel), None) {
                            Ok(item_window) => item_window,
                            Err(error) => return Self::show_error(tree, window, error),
                        }
                    };
                    item_window.set_source(tree, item);
                    item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_BIND), app);
                    window.data_mut::<ItemsPresenter>(tree).items[i] = item_window;
                }
                Self::old_panel(tree, window).drop_window(tree, app);
            },
            Update::Clear => {
                let items = replace(&mut window.data_mut::<ItemsPresenter>(tree).items, Vec::new());
                for item_window in items {
                    item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_UNBIND), app);
                    item_window.set_source(tree, None);
                    item_window.drop_window(tree, app);
                }
            },
            Update::Push(item) => {
                let panel = Self::panel(tree, window);
                let prev = window.data::<ItemsPresenter>(tree).items.last().copied();
                let item_window = if let Some(item_template) = ItemsPresenter::item_template(tree, window) {
                    match item_template.new_instance(tree, Some(panel), prev) {
                        Ok(item_window) => item_window,
                        Err(error) => return Self::show_error(tree, window, error),
                    }
                } else {
                    match StaticText::new(tree, Some(panel), prev) {
                        Ok(item_window) => item_window,
                        Err(error) => return Self::show_error(tree, window, error),
                    }
                };
                item_window.set_source(tree, Some(item));
                item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_BIND), app);
                window.data_mut::<ItemsPresenter>(tree).items.push(item_window);
            },
            Update::Insert(index, item) => {
                let panel = Self::panel(tree, window);
                let prev = index.checked_sub(1).map(|i| window.data::<ItemsPresenter>(tree).items[i]);
                let item_window = if let Some(item_template) = ItemsPresenter::item_template(tree, window) {
                    match item_template.new_instance(tree, Some(panel), prev) {
                        Ok(item_window) => item_window,
                        Err(error) => return Self::show_error(tree, window, error),
                    }
                } else {
                    match StaticText::new(tree, Some(panel), prev) {
                        Ok(item_window) => item_window,
                        Err(error) => return Self::show_error(tree, window, error),
                    }
                };
                item_window.set_source(tree, Some(item));
                item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_BIND), app);
                window.data_mut::<ItemsPresenter>(tree).items.insert(index, item_window);
            },
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
            update_queue: VecDeque::new(),
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
