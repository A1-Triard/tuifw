use crate::{widget, StaticText};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Rect, Vector, Error, Fg, Bg, Key};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Timer, Data};
use tuifw_window::Visibility;

pub const CMD_ITEMS_PRESENTER_BIND: u16 = 140;
pub const CMD_ITEMS_PRESENTER_UNBIND: u16 = 141;

widget! {
    #[widget(ItemsPresenterWidget, init=init)]
    pub struct ItemsPresenter {
        #[property(ref, on_changed=update)]
        items: Vec<Box<dyn Data>>,
        #[property(copy, on_changed=update)]
        panel_template: Option<Window>,
        #[property(copy, on_changed=update)]
        item_template: Option<Window>,
        update_timer: Option<Timer>,
        error: bool,
        #[property(copy)]
        tab_navigation: bool,
        #[property(copy)]
        up_down_navigation: bool,
        #[property(copy)]
        left_right_navigation: bool,
        #[property(copy, on_changed=update)]
        focus_first_item_primary: bool,
        #[property(copy, on_changed=update)]
        focus_first_item_secondary: bool,
    }
}

impl ItemsPresenter {
    fn init(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        let error_text = StaticText::new(tree, Some(window), None)?;
        error_text.set_visibility(tree, Visibility::Collapsed);
        error_text.set_color(tree, 0, (Fg::BrightRed, Bg::Blue));
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

    fn update(tree: &mut WindowTree, window: Window) {
        let update_timer = Timer::new(tree, 0, Box::new(move |tree, app| {
            let data = window.data_mut::<ItemsPresenter>(tree);
            data.update_timer = None;
            if data.error { return; }
            if let Some(panel) = Self::panel(tree, window) {
                if let Some(first_item_window) = panel.first_child(tree) {
                    let mut item_window = first_item_window;
                    loop {
                        item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_UNBIND), app);
                        item_window.set_source_index(tree, None);
                        item_window = item_window.next(tree);
                        if item_window == first_item_window { break; }
                    }
                }
                panel.drop_window(tree, app);
            }
            let data = window.data::<ItemsPresenter>(tree);
            if let Some(item_template) = data.item_template {
                if let Some(panel_template) = data.panel_template {
                    let mut focus_item_primary = data.focus_first_item_primary;
                    let mut focus_item_secondary = data.focus_first_item_secondary;
                    let panel = match panel_template.new_instance(tree, Some(window), None) {
                        Ok(panel) => panel,
                        Err(error) => return Self::show_error(tree, window, error),
                    };
                    let mut prev = None;
                    for item_index in 0 .. window.data::<ItemsPresenter>(tree).items.len() {
                        let item_window = match item_template.new_instance(tree, Some(panel), prev) {
                            Ok(item_window) => item_window,
                            Err(error) => return Self::show_error(tree, window, error),
                        };
                        item_window.set_source_index(tree, Some(item_index));
                        item_window.raise(tree, Event::Cmd(CMD_ITEMS_PRESENTER_BIND), app);
                        if focus_item_primary {
                            focus_item_primary = false;
                            item_window.set_focused_primary(tree, true);
                        }
                        if focus_item_secondary {
                            focus_item_secondary = false;
                            item_window.set_focused_secondary(tree, true);
                        }
                        prev = Some(item_window);
                    }
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
struct ItemsPresenterWidget;

impl_supports_interfaces!(ItemsPresenterWidget);

impl Widget for ItemsPresenterWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(ItemsPresenter {
            panel_template: None,
            item_template: None,
            update_timer: None,
            error: false,
            items: Vec::new(),
            tab_navigation: false,
            up_down_navigation: false,
            left_right_navigation: false,
            focus_first_item_primary: false,
            focus_first_item_secondary: false,
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
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        event_source: Window,
        _app: &mut dyn App,
    ) -> bool {
        match event {
            Event::Key(Key::Tab) => {
                let data = window.data::<ItemsPresenter>(tree);
                if data.tab_navigation {
                    if event_source.parent(tree).and_then(|x| x.parent(tree)) == Some(window) {
                        let focus = event_source.next(tree);
                        if event_source.is_secondary_focused(tree) {
                            focus.set_focused_secondary(tree, true);
                            true
                        } else if event_source.is_primary_focused(tree) {
                            focus.set_focused_primary(tree, true);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            Event::Key(Key::Down) => {
                let data = window.data::<ItemsPresenter>(tree);
                if data.up_down_navigation {
                    if event_source.parent(tree).and_then(|x| x.parent(tree)) == Some(window) {
                        let focus = event_source.next(tree);
                        if focus == event_source.parent(tree).unwrap().first_child(tree).unwrap() {
                            false
                        } else {
                            if event_source.is_secondary_focused(tree) {
                                focus.set_focused_secondary(tree, true);
                                true
                            } else if event_source.is_primary_focused(tree) {
                                focus.set_focused_primary(tree, true);
                                true
                            } else {
                                false
                            }
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            Event::Key(Key::Up) => {
                let data = window.data::<ItemsPresenter>(tree);
                if data.up_down_navigation {
                    if event_source.parent(tree).and_then(|x| x.parent(tree)) == Some(window) {
                        let focus = {
                            let mut item = event_source.parent(tree).unwrap().first_child(tree).unwrap();
                            loop {
                                let next = item.next(tree);
                                if next == event_source { break item; }
                                item = next;
                            }
                        };
                        if focus.next(tree) == event_source.parent(tree).unwrap().first_child(tree).unwrap() {
                            false
                        } else {
                            if event_source.is_secondary_focused(tree) {
                                focus.set_focused_secondary(tree, true);
                                true
                            } else if event_source.is_primary_focused(tree) {
                                focus.set_focused_primary(tree, true);
                                true
                            } else {
                                false
                            }
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            Event::Key(Key::Right) => {
                let data = window.data::<ItemsPresenter>(tree);
                if data.left_right_navigation {
                    if event_source.parent(tree).and_then(|x| x.parent(tree)) == Some(window) {
                        let focus = event_source.next(tree);
                        if focus == event_source.parent(tree).unwrap().first_child(tree).unwrap() {
                            false
                        } else {
                            if event_source.is_secondary_focused(tree) {
                                focus.set_focused_secondary(tree, true);
                                true
                            } else if event_source.is_primary_focused(tree) {
                                focus.set_focused_primary(tree, true);
                                true
                            } else {
                                false
                            }
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            Event::Key(Key::Left) => {
                let data = window.data::<ItemsPresenter>(tree);
                if data.left_right_navigation {
                    if event_source.parent(tree).and_then(|x| x.parent(tree)) == Some(window) {
                        let focus = {
                            let mut item = event_source.parent(tree).unwrap().first_child(tree).unwrap();
                            loop {
                                let next = item.next(tree);
                                if next == event_source { break item; }
                                item = next;
                            }
                        };
                        if focus.next(tree) == event_source.parent(tree).unwrap().first_child(tree).unwrap() {
                            false
                        } else {
                            if event_source.is_secondary_focused(tree) {
                                focus.set_focused_secondary(tree, true);
                                true
                            } else if event_source.is_primary_focused(tree) {
                                focus.set_focused_primary(tree, true);
                                true
                            } else {
                                false
                            }
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            _ => false,
        }
    }
}
