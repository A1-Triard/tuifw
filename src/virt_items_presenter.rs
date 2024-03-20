use crate::{widget, StaticText, StackPanel};
use crate::virt_scroll_viewer::*;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::cmp::{max, min};
use core::mem::{replace, size_of};
use core::ops::Range;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Rect, Vector, Error, Fg, Bg, Thickness, Key};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Timer, Data};
use tuifw_window::Visibility;

pub const CMD_VIRT_ITEMS_PRESENTER_BIND: u16 = 150;
pub const CMD_VIRT_ITEMS_PRESENTER_UNBIND: u16 = 151;

widget! {
    #[widget(VirtItemsPresenterWidget, init=init)]
    pub struct VirtItemsPresenter {
        #[property(ref, arrange, on_changed=on_items_changed)]
        items: Vec<Box<dyn Data>>,
        #[property(copy, measure, on_changed=on_vertical_changed)]
        vertical: bool,
        #[property(copy, on_changed=on_templates_changed)]
        item_template: Option<Window>,
        #[property(copy, on_changed=on_offset_changed)]
        offset: i16,
        viewport: i16,
        item_size: i16,
        update_timer: Option<Timer>,
        templates_changed: bool,
        error: bool,
        #[property(copy)]
        tab_navigation: bool,
        #[property(copy)]
        up_down_navigation: bool,
        #[property(copy)]
        left_right_navigation: bool,
        #[property(copy, on_changed=on_focus_first_item_changed)]
        focus_first_item_primary: bool,
        #[property(copy, on_changed=on_focus_first_item_changed)]
        focus_first_item_secondary: bool,
        visible_range: Range<usize>,
        focus_first_item_primary_once: bool,
        focus_first_item_secondary_once: bool,
    }
}

impl VirtItemsPresenter {
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
        window.data_mut::<VirtItemsPresenter>(tree).error = true;
        let error_text = Self::error_text(tree, window);
        StaticText::set_text(tree, error_text, error.to_string());
        error_text.set_visibility(tree, Visibility::Visible);
    }

    fn on_items_changed(tree: &mut WindowTree, window: Window) {
        Self::on_extent_changed(tree, window);
        Self::on_templates_changed(tree, window);
    }

    fn on_vertical_changed(tree: &mut WindowTree, window: Window) {
        let viewport = window.inner_bounds(tree).size;
        let data = window.data::<VirtItemsPresenter>(tree);
        let vertical = data.vertical;
        let offset = data.offset;
        let viewport = if vertical { viewport.y } else { viewport.x };
        let extent = (data.items.len() as u16 as i16).wrapping_mul(data.item_size);
        if let Some(parent) = window.parent(tree) {
            if let Some(sv) = parent.widget_extension::<dyn VirtScrollViewerWidgetExtension>(tree) {
                sv.set_extent(tree, parent, vertical, extent);
                sv.set_offset(tree, parent, vertical, offset);
                sv.set_viewport(tree, parent, vertical, viewport);
            }
        }
        Self::on_templates_changed(tree, window);
    }

    fn on_extent_changed(tree: &mut WindowTree, window: Window) {
        let data = window.data::<VirtItemsPresenter>(tree);
        let vertical = data.vertical;
        let extent = (data.items.len() as u16 as i16).wrapping_mul(data.item_size);
        if let Some(parent) = window.parent(tree) {
            if let Some(sv) = parent.widget_extension::<dyn VirtScrollViewerWidgetExtension>(tree) {
                sv.set_extent(tree, parent, vertical, extent);
            }
        }
    }

    fn on_offset_changed(tree: &mut WindowTree, window: Window) {
        let data = window.data::<VirtItemsPresenter>(tree);
        let vertical = data.vertical;
        let offset = data.offset;
        if let Some(parent) = window.parent(tree) {
            if let Some(sv) = parent.widget_extension::<dyn VirtScrollViewerWidgetExtension>(tree) {
                sv.set_offset(tree, parent, vertical, offset);
            }
        }
        Self::update(tree, window);
    }

    fn on_focus_first_item_changed(tree: &mut WindowTree, window: Window) {
        VirtItemsPresenter::set_offset(tree, window, 0);
        Self::on_templates_changed(tree, window);
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
            let old_items_range = data.visible_range.clone();
            if data.templates_changed {
                let mut focus_item_primary =
                    data.focus_first_item_primary | replace(&mut data.focus_first_item_primary_once, false)
                ;
                let mut focus_item_secondary =
                    data.focus_first_item_secondary | replace(&mut data.focus_first_item_secondary_once, false)
                ;
                data.templates_changed = false;
                if let Some(panel) = Self::panel(tree, window) {
                    if let Some(first_item_window) = panel.first_child(tree) {
                        let mut item_window = first_item_window;
                        loop {
                            item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_UNBIND), app);
                            item_window.set_source_index(tree, None);
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
                    for item_index in items_range.clone() {
                        let item_window = match item_template.new_instance(tree, Some(panel), prev) {
                            Ok(item_window) => item_window,
                            Err(error) => return Self::show_error(tree, window, error),
                        };
                        item_window.set_source_index(tree, Some(item_index));
                        item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND), app);
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
            } else if let Some(panel) = Self::panel(tree, window) {
                let data = window.data::<VirtItemsPresenter>(tree);
                let item_template = data.item_template.unwrap();
                panel.set_margin(tree, panel_margin);
                let drop_head_range = old_items_range.start .. min(items_range.start, old_items_range.end);
                let drop_tail_range = max(items_range.end, old_items_range.start) .. old_items_range.end;
                let new_head_range = items_range.start .. min(old_items_range.start, items_range.end);
                let new_tail_range = max(old_items_range.end, items_range.start) .. items_range.end;
                let drop_head = drop_head_range.end.saturating_sub(drop_head_range.start);
                let drop_tail = drop_tail_range.end.saturating_sub(drop_tail_range.start);
                if let Some(first_item_window) = panel.first_child(tree) {
                    let mut item_window = first_item_window;
                    for _ in 0 .. drop_head {
                        let next = item_window.next(tree);
                        item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_UNBIND), app);
                        item_window.set_source_index(tree, None);
                        item_window.drop_window(tree, app);
                        item_window = next;
                    }
                    for _ in 0 .. (old_items_range.end - old_items_range.start) - drop_head - drop_tail {
                        item_window = item_window.next(tree);
                    }
                    for _ in 0 .. drop_tail {
                        let next = item_window.next(tree);
                        item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_UNBIND), app);
                        item_window.set_source_index(tree, None);
                        item_window.drop_window(tree, app);
                        item_window = next;
                    }
                }
                let mut prev = None;
                if let Some(first_item_window) = panel.first_child(tree) {
                    let mut item_window = first_item_window;
                    for _ in 0 .. (old_items_range.end - old_items_range.start) - drop_head - drop_tail {
                        prev = Some(item_window);
                        item_window = item_window.next(tree);
                    }
                    debug_assert_eq!(item_window, first_item_window);
                }
                for item_index in new_tail_range {
                    let item_window = match item_template.new_instance(tree, Some(panel), prev) {
                        Ok(item_window) => item_window,
                        Err(error) => return Self::show_error(tree, window, error),
                    };
                    item_window.set_source_index(tree, Some(item_index));
                    item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND), app);
                    prev = Some(item_window);
                }
                for item_index in new_head_range.rev() {
                    let item_window = match item_template.new_instance(tree, Some(panel), None) {
                        Ok(item_window) => item_window,
                        Err(error) => return Self::show_error(tree, window, error),
                    };
                    item_window.set_source_index(tree, Some(item_index));
                    item_window.raise(tree, Event::Cmd(CMD_VIRT_ITEMS_PRESENTER_BIND), app);
                }
            }
            let data = window.data_mut::<VirtItemsPresenter>(tree);
            data.visible_range = items_range;
        }));
        let data = window.data_mut::<VirtItemsPresenter>(tree);
        if let Some(old_update_timer) = data.update_timer.replace(update_timer) {
            old_update_timer.drop_timer(tree);
        }
    }
}

#[derive(Clone, Default)]
pub struct VirtItemsPresenterWidget;

impl_supports_interfaces!(VirtItemsPresenterWidget: VirtItemsPresenterWidgetExtension);

impl VirtItemsPresenterWidgetExtension for VirtItemsPresenterWidget { }

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
            tab_navigation: false,
            up_down_navigation: false,
            left_right_navigation: false,
            focus_first_item_primary: false,
            focus_first_item_secondary: false,
            visible_range: 0 .. 0,
            focus_first_item_primary_once: false,
            focus_first_item_secondary_once: false,
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
        let data = window.data_mut::<VirtItemsPresenter>(tree);
        let vertical = data.vertical;
        if vertical {
            size.y = final_inner_bounds.h();
            if data.viewport != size.y {
                data.viewport = size.y;
                if let Some(parent) = window.parent(tree) {
                    if let Some(sv) = parent.widget_extension::<dyn VirtScrollViewerWidgetExtension>(tree) {
                        sv.set_viewport(tree, parent, vertical, size.y);
                    }
                }
                VirtItemsPresenter::update(tree, window);
            }
            let data = window.data_mut::<VirtItemsPresenter>(tree);
            if data.item_size != item_size.y {
                data.item_size = item_size.y;
                VirtItemsPresenter::update(tree, window);
                VirtItemsPresenter::on_extent_changed(tree, window);
            }
        } else {
            size.x = final_inner_bounds.w();
            if data.viewport != size.x {
                data.viewport = size.x;
                if let Some(parent) = window.parent(tree) {
                    if let Some(sv) = parent.widget_extension::<dyn VirtScrollViewerWidgetExtension>(tree) {
                        sv.set_viewport(tree, parent, vertical, size.x);
                    }
                }
                VirtItemsPresenter::update(tree, window);
            }
            let data = window.data_mut::<VirtItemsPresenter>(tree);
            if data.item_size != item_size.x {
                data.item_size = item_size.x;
                VirtItemsPresenter::update(tree, window);
                VirtItemsPresenter::on_extent_changed(tree, window);
            }
        }
        size
    }

    fn bring_into_view(
        &self,
        tree: &mut WindowTree,
        window: Window,
        rect: Rect,
    ) -> bool {
        let bounds = window.inner_bounds(tree);
        let data = window.data_mut::<VirtItemsPresenter>(tree);
        let offset = data.offset;
        if data.vertical {
            if rect.v_range().intersect(bounds.v_range()).is_empty() {
                let from_top = rect.t().wrapping_sub(bounds.t()).checked_abs().map_or(i16::MIN, |x| -x);
                let from_bottom = rect.b().wrapping_sub(bounds.b()).checked_abs().map_or(i16::MIN, |x| -x);
                if from_top >= from_bottom {
                    VirtItemsPresenter::set_offset(tree, window, offset.wrapping_add(from_top));
                } else {
                    VirtItemsPresenter::set_offset(tree, window, offset.wrapping_sub(from_bottom));
                }
            }
        } else {
            if rect.h_range().intersect(bounds.h_range()).is_empty() {
                let from_left = rect.l().wrapping_sub(bounds.l()).checked_abs().map_or(i16::MIN, |x| -x);
                let from_right = rect.r().wrapping_sub(bounds.r()).checked_abs().map_or(i16::MIN, |x| -x);
                if from_left >= from_right {
                    VirtItemsPresenter::set_offset(tree, window, offset.wrapping_add(from_left));
                } else {
                    VirtItemsPresenter::set_offset(tree, window, offset.wrapping_sub(from_right));
                }
            }
        }
        true
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
                let data = window.data::<VirtItemsPresenter>(tree);
                if data.tab_navigation {
                    if event_source.parent(tree).and_then(|x| x.parent(tree)) == Some(window) {
                        let focus = event_source.next(tree);
                        if event_source.is_secondary_focused(tree) {
                            if focus == event_source.parent(tree).unwrap().first_child(tree).unwrap() {
                                let data = window.data_mut::<VirtItemsPresenter>(tree);
                                data.focus_first_item_secondary_once = true;
                                VirtItemsPresenter::set_offset(tree, window, 0);
                                VirtItemsPresenter::on_templates_changed(tree, window);
                            } else {
                                focus.set_focused_secondary(tree, true);
                            }
                            true
                        } else if event_source.is_primary_focused(tree) {
                            if focus == event_source.parent(tree).unwrap().first_child(tree).unwrap() {
                                let data = window.data_mut::<VirtItemsPresenter>(tree);
                                data.focus_first_item_primary_once = true;
                                VirtItemsPresenter::set_offset(tree, window, 0);
                                VirtItemsPresenter::on_templates_changed(tree, window);
                            } else {
                                focus.set_focused_primary(tree, true);
                            }
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
                let data = window.data::<VirtItemsPresenter>(tree);
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
                let data = window.data::<VirtItemsPresenter>(tree);
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
                let data = window.data::<VirtItemsPresenter>(tree);
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
                let data = window.data::<VirtItemsPresenter>(tree);
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
