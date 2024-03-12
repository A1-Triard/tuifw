use crate::widget;
use alloc::boxed::Box;
use alloc::string::String;
use core::cmp::max;
use dynamic_cast::impl_supports_interfaces;
use either::Left;
use tuifw_screen_base::{Point, Rect, Vector, Thickness, text_width, HAlign, VAlign, Error};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App};
use tuifw_window::{COLOR_FRAME, COLORS, COLOR_IN_FRAME};

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
        h_extent: i16,
        #[property(copy, arrange)]
        h_offset: i16,
        h_viewport: i16,
        v_extent: i16,
        #[property(copy, arrange)]
        v_offset: i16,
        v_viewport: i16,
    }
}

impl ScrollViewer {
    fn init_palette(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(COLOR_FRAME));
            for c in COLORS {
                palette.set(c, Left(c + COLOR_IN_FRAME));
            }
        });
        Ok(())
    }

    pub fn h_extent(tree: &WindowTree, window: Window) -> i16 {
        let data = window.data::<ScrollViewer>(tree);
        data.h_extent
    }

    pub fn h_viewport(tree: &WindowTree, window: Window) -> i16 {
        let data = window.data::<ScrollViewer>(tree);
        data.h_viewport
    }

    pub fn v_extent(tree: &WindowTree, window: Window) -> i16 {
        let data = window.data::<ScrollViewer>(tree);
        data.v_extent
    }

    pub fn v_viewport(tree: &WindowTree, window: Window) -> i16 {
        let data = window.data::<ScrollViewer>(tree);
        data.v_viewport
    }
}

#[derive(Clone, Default)]
pub struct ScrollViewerWidget;

impl_supports_interfaces!(ScrollViewerWidget);

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
        rp.fill_bg(color.1);
        rp.h_line(bounds.tl, bounds.w(), true, color);
        rp.h_line(bounds.bl_inner(), bounds.w(), true, color);
        rp.v_line(bounds.tl, bounds.h(), true, color);
        rp.v_line(bounds.tr_inner(), bounds.h(), true, color);
        rp.tl_edge(bounds.tl, true, color);
        rp.tr_edge(bounds.tr_inner(), true, color);
        rp.br_edge(bounds.br_inner(), true, color);
        rp.bl_edge(bounds.bl_inner(), true, color);
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
        let children_width = if h_scroll || available_width.is_none() { None } else { Some(children_size.x) };
        let children_height = if v_scroll || available_height.is_none() { None } else { Some(children_size.y) };
        let mut size = Vector::null();
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.measure(tree, children_width, children_height, app);
                size = size.max(child.desired_size(tree));
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        let data = window.data_mut::<ScrollViewer>(tree);
        data.h_extent = size.x;
        data.h_viewport = children_size.x;
        data.v_extent = size.y;
        data.v_viewport = children_size.y;
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
        let data = window.data::<ScrollViewer>(tree);
        let offset = -Vector { x: data.h_offset, y: data.v_offset };
        let mut children_bounds = base_children_bounds.offset(offset);
        if data.h_scroll {
            children_bounds.size.x = data.h_extent;
        }
        if data.v_scroll {
            children_bounds.size.y = data.v_extent;
        }
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.arrange(tree, children_bounds, app);
                child.set_clip(tree, Some(Rect {
                    tl: Point { x: max(0, -offset.x), y: max(0, -offset.y) },
                    size: base_children_bounds.size
                }));
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
