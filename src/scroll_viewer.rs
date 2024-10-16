use crate::widget;
use crate::virt_scroll_viewer::*;
use alloc::boxed::Box;
use alloc::string::String;
use core::cmp::max;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Point, Rect, Vector, Thickness, text_width, HAlign, VAlign, Error};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Color};
use tuifw_window::{COLOR_FRAME};

widget! {
    #[widget(ScrollViewerWidget, init=init_palette)]
    pub struct ScrollViewer {
        #[property(str, render)]
        text: String,
        #[property(copy, render)]
        text_align: HAlign,
        #[property(copy, measure)]
        h_scroll: bool,
        #[property(copy, measure)]
        v_scroll: bool,
        h_extent: u32,
        #[property(copy, arrange)]
        h_offset: u32,
        h_viewport: u32,
        v_extent: u32,
        #[property(copy, arrange)]
        v_offset: u32,
        v_viewport: u32,
        has_virtual_child: bool,
    }
}

impl ScrollViewer {
    fn init_palette(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        window.palette_mut(tree, |palette| {
            palette.set(0, Color::Palette(COLOR_FRAME));
        });
        Ok(())
    }

    pub fn h_extent(tree: &WindowTree, window: Window) -> u32 {
        let data = window.data::<ScrollViewer>(tree);
        data.h_extent
    }

    pub fn h_viewport(tree: &WindowTree, window: Window) -> u32 {
        let data = window.data::<ScrollViewer>(tree);
        data.h_viewport
    }

    pub fn v_extent(tree: &WindowTree, window: Window) -> u32 {
        let data = window.data::<ScrollViewer>(tree);
        data.v_extent
    }

    pub fn v_viewport(tree: &WindowTree, window: Window) -> u32 {
        let data = window.data::<ScrollViewer>(tree);
        data.v_viewport
    }
}

#[derive(Clone, Default)]
struct ScrollViewerWidget;

impl_supports_interfaces!(ScrollViewerWidget: VirtScrollViewerWidgetExtension);

impl VirtScrollViewerWidgetExtension for ScrollViewerWidget {
    fn set_offset(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: u32) {
        let data = window.data_mut::<ScrollViewer>(tree);
        if vertical {
            data.v_offset = value;
        } else {
            data.h_offset = value;
        }
        window.invalidate_render(tree);
    }

    fn set_viewport(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: u32) {
        let data = window.data_mut::<ScrollViewer>(tree);
        if vertical {
            data.v_viewport = value;
        } else {
            data.h_viewport = value;
        }
        window.invalidate_render(tree);
    }

    fn set_extent(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: u32) {
        let data = window.data_mut::<ScrollViewer>(tree);
        if vertical {
            data.v_extent = value;
        } else {
            data.h_extent = value;
        }
        window.invalidate_render(tree);
    }
}

impl Widget for ScrollViewerWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(ScrollViewer {
            text: String::new(), text_align: HAlign::Left,
            h_scroll: false,
            v_scroll: false,
            h_extent: 0,
            h_offset: 0,
            h_viewport: 0,
            v_extent: 0,
            v_offset: 0,
            v_viewport: 0,
            has_virtual_child: false,
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        ScrollViewer::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
    ) {
        let color = window.color(tree, 0);
        let bounds = window.inner_bounds(tree);
        let data = window.data::<ScrollViewer>(tree);
        rp.fill_bg(color);
        rp.h_line(bounds.tl, bounds.w(), true, color);
        rp.h_line(bounds.bl_inner(), bounds.w(), true, color);
        rp.v_line(bounds.tl, bounds.h(), true, color);
        rp.v_line(bounds.tr_inner(), bounds.h(), true, color);
        rp.tl_edge(bounds.tl, true, color);
        rp.tr_edge(bounds.tr_inner(), true, color);
        rp.br_edge(bounds.br_inner(), true, color);
        rp.bl_edge(bounds.bl_inner(), true, color);
        let indicator_area = Thickness::all(1).shrink_rect(bounds);
        if data.v_scroll {
            let v_indicator_range = (indicator_area.h() as u16).saturating_sub(1) as i16;
            let v_indicator =
                (
                    (
                        i64::from(data.v_offset) * i64::from(v_indicator_range) +
                        i64::from(data.v_extent - data.v_viewport) / 2
                    )
                    /
                    i64::from(data.v_extent - data.v_viewport)
                ) as i16
            ;
            rp.text(Point { x: bounds.r_inner(), y: indicator_area.t().wrapping_add(v_indicator) }, color, "╬");
        }
        if data.h_scroll {
            let h_indicator_range = (indicator_area.w() as u16).saturating_sub(1) as i16;
            let h_indicator =
                (
                    (
                        i64::from(data.h_offset) * i64::from(h_indicator_range) +
                        i64::from(data.h_extent - data.h_viewport) / 2
                    )
                    /
                    i64::from(data.h_extent - data.h_viewport)
                ) as i16
            ;
            rp.text(Point { x: indicator_area.l().wrapping_add(h_indicator), y: bounds.b_inner() }, color, "╩");
        }
        if !data.text.is_empty() {
            let text_area_bounds = Thickness::new(2, 0, 2, 0).shrink_rect(bounds.t_line());
            let text_width = text_width(&data.text);
            if text_width <= text_area_bounds.w() {
                let margin = Thickness::align(
                    Vector { x: text_width, y: 1 },
                    text_area_bounds.size,
                    data.text_align,
                    VAlign::Top
                );
                let text_bounds = margin.shrink_rect(text_area_bounds);
                rp.text(text_bounds.tl.offset(Vector { x: -1, y: 0 }), color, " ");
                rp.text(text_bounds.tl, color, &data.text);
                rp.text(text_bounds.tr(), color, " ");
            } else {
                rp.text(text_area_bounds.tl.offset(Vector { x: -1, y: 0 }), color, " ");
                rp.text(text_area_bounds.tl, color, &data.text);
                rp.text(text_area_bounds.tr(), color, "►");
                rp.tr_edge(bounds.tr_inner(), true, color);
            }
        }
    }

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        available_width: Option<i16>,
        available_height: Option<i16>,
        app: &mut dyn App,
    ) -> Vector {
        let data = window.data::<ScrollViewer>(tree);
        let h_scroll = data.h_scroll;
        let v_scroll = data.v_scroll;
        let available_size = Vector { x: available_width.unwrap_or(0), y: available_height.unwrap_or(0) };
        let children_size = Thickness::all(1).shrink_rect_size(available_size);
        let mut virt = false;
        let mut size = Vector::null();
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                let virtual_child = child.widget_extension::<dyn VirtItemsPresenterWidgetExtension>(tree)
                    .is_some()
                ;
                if virtual_child { virt = true; }
                let child_width = if (h_scroll && !virtual_child) || available_width.is_none() {
                    None
                } else {
                    Some(children_size.x)
                };
                let child_height = if (v_scroll && !virtual_child) || available_height.is_none() {
                    None
                } else {
                    Some(children_size.y)
                };
                child.measure(tree, child_width, child_height, app);
                size = size.max(child.desired_size(tree));
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        let data = window.data_mut::<ScrollViewer>(tree);
        data.has_virtual_child = virt;
        if !virt {
            data.h_extent = (size.x as u16).into();
            data.h_viewport = (children_size.x as u16).into();
            data.v_extent = (size.y as u16).into();
            data.v_viewport = (children_size.y as u16).into();
        }
        let size = Thickness::all(1).expand_rect_size(size);
        Vector {
            x: if h_scroll { available_size.x } else { size.x },
            y: if v_scroll { available_size.y } else { size.y },
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        app: &mut dyn App,
    ) -> Vector {
        let base_children_bounds = Thickness::all(1).shrink_rect(final_inner_bounds);
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                let virtual_child = child.widget_extension::<dyn VirtItemsPresenterWidgetExtension>(tree)
                    .is_some()
                ;
                let child_bounds = if virtual_child {
                    base_children_bounds
                } else {
                    let data = window.data::<ScrollViewer>(tree);
                    let offset = -Vector {
                        x: u16::try_from(data.h_offset).unwrap() as i16,
                        y: u16::try_from(data.v_offset).unwrap() as i16
                    };
                    let mut child_bounds = base_children_bounds.offset(offset);
                    if data.h_scroll {
                        child_bounds.size.x = u16::try_from(data.h_extent).unwrap() as i16;
                    }
                    if data.v_scroll {
                        child_bounds.size.y = u16::try_from(data.v_extent).unwrap() as i16;
                    }
                    child.set_clip(tree, Some(Rect {
                        tl: Point { x: max(0, -offset.x), y: max(0, -offset.y) },
                        size: base_children_bounds.size
                    }));
                    child_bounds
                };
                child.arrange(tree, child_bounds, app);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        final_inner_bounds.size
    }

    fn bring_into_view(
        &self,
        tree: &mut WindowTree,
        window: Window,
        rect: Rect,
    ) -> bool {
        let data = window.data::<ScrollViewer>(tree);
        if data.has_virtual_child { return false; }
        let bounds = window.inner_bounds(tree);
        let bounds = Thickness::all(1).shrink_rect(bounds);
        let data = window.data_mut::<ScrollViewer>(tree);
        if data.v_scroll {
            let offset = data.v_offset;
            if rect.v_range().intersect(bounds.v_range()).is_empty() {
                let from_top = rect.t().wrapping_sub(bounds.t()).checked_abs().map_or(i16::MIN, |x| -x);
                let from_bottom = rect.b().wrapping_sub(bounds.b()).checked_abs().map_or(i16::MIN, |x| -x);
                if from_top >= from_bottom {
                    ScrollViewer::set_v_offset(tree, window, offset.wrapping_add((from_top as u16).into()));
                } else {
                    ScrollViewer::set_v_offset(tree, window, offset.wrapping_sub((from_bottom as u16).into()));
                }
            }
        }
        let data = window.data_mut::<ScrollViewer>(tree);
        if data.h_scroll {
            let offset = data.h_offset;
            if rect.h_range().intersect(bounds.h_range()).is_empty() {
                let from_left = rect.l().wrapping_sub(bounds.l()).checked_abs().map_or(i16::MIN, |x| -x);
                let from_right = rect.r().wrapping_sub(bounds.r()).checked_abs().map_or(i16::MIN, |x| -x);
                if from_left >= from_right {
                    ScrollViewer::set_h_offset(tree, window, offset.wrapping_add((from_left as u16).into()));
                } else {
                    ScrollViewer::set_h_offset(tree, window, offset.wrapping_sub((from_right as u16).into()));
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
        _event_source: Window,
        _app: &mut dyn App,
    ) -> bool {
        if let Event::LmbDown(point) = event {
            let point = window.inner_point(point, tree); // TODO swap args
            let bounds = window.inner_bounds(tree);
            let data = window.data::<ScrollViewer>(tree);
            if point.x == bounds.r_inner() && data.v_scroll {
                let indicator_area = Thickness::all(1).shrink_rect(bounds);
                if indicator_area.v_range().contains(point.y) {
                    let new_indicator_pos = point.offset_from(indicator_area.tl).y;
                    let v_indicator_range = (indicator_area.h() as u16).saturating_sub(1) as i16;
                    let offset = if v_indicator_range == 0 {
                        0
                    } else {
                        ((
                            u64::from(new_indicator_pos as u16) * u64::from(data.v_extent - data.v_viewport) +
                            u64::from(v_indicator_range as u16) / 2
                        ) / u64::from(v_indicator_range as u16)) as u32
                    };
                    let mut has_virtual_child = false;
                    if let Some(first_child) = window.first_child(tree) {
                        let mut child = first_child;
                        loop {
                            if let Some(virtual_child) = child.widget_extension::<dyn VirtItemsPresenterWidgetExtension>(tree) {
                                virtual_child.set_offset(tree, child, true, offset);
                                has_virtual_child = true;
                            }
                            child = child.next(tree);
                            if child == first_child { break; }
                        }
                    }
                    if !has_virtual_child {
                        ScrollViewer::set_v_offset(tree, window, offset);
                    }
                    true
                } else {
                    false
                }
            } else if point.y == bounds.b_inner() && data.h_scroll {
                let indicator_area = Thickness::all(1).shrink_rect(bounds);
                if indicator_area.h_range().contains(point.x) {
                    let new_indicator_pos = point.offset_from(indicator_area.tl).x;
                    let h_indicator_range = (indicator_area.w() as u16).saturating_sub(1) as i16;
                    let offset = if h_indicator_range == 0 {
                        0
                    } else {
                        ((
                            u64::from(new_indicator_pos as u16) * u64::from(data.h_extent - data.h_viewport) +
                            u64::from(h_indicator_range as u16) / 2
                        ) / u64::from(h_indicator_range as u16)) as u32
                    };
                    let mut has_virtual_child = false;
                    if let Some(first_child) = window.first_child(tree) {
                        let mut child = first_child;
                        loop {
                            if let Some(virtual_child) = child.widget_extension::<dyn VirtItemsPresenterWidgetExtension>(tree) {
                                virtual_child.set_offset(tree, child, false, offset);
                                has_virtual_child = true;
                            }
                            child = child.next(tree);
                            if child == first_child { break; }
                        }
                    }
                    if !has_virtual_child {
                        ScrollViewer::set_h_offset(tree, window, offset);
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
    }
}
