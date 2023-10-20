use crate::widget2;
use alloc::boxed::Box;
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Rect, Vector, Thickness, text_width, HAlign, VAlign};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App};
use tuifw_window::{COLOR_FRAME, COLORS, COLOR_IN_FRAME};

widget2! {
    #[widget(FrameWidget, init=init_palette)]
    pub struct Frame {
        #[property(value, render)]
        double: bool,
        #[property(ref, render)]
        text: String,
        #[property(value, render)]
        text_align: HAlign,
    }
}

impl WidgetData for Frame { }

impl Frame {
    fn init_palette(tree: &mut WindowTree, window: Window) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(COLOR_FRAME));
            for c in COLORS {
                palette.set(c, Left(c + COLOR_IN_FRAME));
            }
        });
    }
}

#[derive(Clone, Default)]
pub struct FrameWidget;

impl Widget for FrameWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(Frame {
            double: false, text: String::new(), text_align: HAlign::Left
        })
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
        let data = window.data::<Frame>(tree);
        rp.fill_bg(color.1);
        rp.h_line(bounds.tl, bounds.w(), data.double, color);
        rp.h_line(bounds.bl_inner(), bounds.w(), data.double, color);
        rp.v_line(bounds.tl, bounds.h(), data.double, color);
        rp.v_line(bounds.tr_inner(), bounds.h(), data.double, color);
        rp.tl_edge(bounds.tl, data.double, color);
        rp.tr_edge(bounds.tr_inner(), data.double, color);
        rp.br_edge(bounds.br_inner(), data.double, color);
        rp.bl_edge(bounds.bl_inner(), data.double, color);
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
                rp.tr_edge(bounds.tr_inner(), data.double, color);
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
        let available_size = Vector { x: available_width.unwrap_or(0), y: available_height.unwrap_or(0) };
        let children_size = Thickness::all(1).shrink_rect_size(available_size);
        let children_width = if available_width.is_none() { None } else { Some(children_size.x) };
        let children_height = if available_height.is_none() { None } else { Some(children_size.y) };
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
        Thickness::all(1).expand_rect_size(size)
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        app: &mut dyn App,
    ) -> Vector {
        let children_bounds = Thickness::all(1).shrink_rect(final_inner_bounds);
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.arrange(tree, children_bounds, app);
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
