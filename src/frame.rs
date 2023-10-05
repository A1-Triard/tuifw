use crate::{prop_string_render, prop_value_render, widget};
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Rect, Vector, Thickness, text_width, HAlign, VAlign};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree};

pub struct Frame {
    double: bool,
    text: String,
    text_align: HAlign,
}

impl<State: ?Sized> WidgetData<State> for Frame { }

impl Frame {
    pub fn new() -> Self {
        Frame { double: false, text: String::new(), text_align: HAlign::Left }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(20));
            palette.set(11, Left(21));
            palette.set(12, Left(22));
            palette.set(13, Left(23));
            palette.set(14, Left(24));
            palette.set(15, Left(25));
            palette.set(16, Left(26));
            palette.set(17, Left(27));
            palette.set(18, Left(28));
            palette.set(19, Left(29));
        });
    }

    widget!(FrameWidget; init_palette);
    prop_value_render!(double: bool);
    prop_string_render!(text);
    prop_value_render!(text_align: HAlign);
}

impl Default for Frame {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct FrameWidget;

impl<State: ?Sized> Widget<State> for FrameWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let color = window.color(tree, 0);
        let bounds = window.inner_bounds(tree);
        let data = window.data::<Frame>(tree);
        rp.fill_bg(color.1);
        rp.h_line(bounds.tl, bounds.w(), data.double, color.0, color.1);
        rp.h_line(bounds.bl_inner(), bounds.w(), data.double, color.0, color.1);
        rp.v_line(bounds.tl, bounds.h(), data.double, color.0, color.1);
        rp.v_line(bounds.tr_inner(), bounds.h(), data.double, color.0, color.1);
        rp.tl_edge(bounds.tl, data.double, color.0, color.1);
        rp.tr_edge(bounds.tr_inner(), data.double, color.0, color.1);
        rp.br_edge(bounds.br_inner(), data.double, color.0, color.1);
        rp.bl_edge(bounds.bl_inner(), data.double, color.0, color.1);
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
                rp.out(text_bounds.tl.offset(Vector { x: -1, y: 0 }), color.0, color.1, " ");
                rp.out(text_bounds.tl, color.0, color.1, &data.text);
                rp.out(text_bounds.tr(), color.0, color.1, " ");
            } else {
                rp.out(text_area_bounds.tl.offset(Vector { x: -1, y: 0 }), color.0, color.1, " ");
                rp.out(text_area_bounds.tl, color.0, color.1, &data.text);
                rp.out(text_area_bounds.tr(), color.0, color.1, "►");
                rp.tr_edge(bounds.tr_inner(), data.double, color.0, color.1);
            }
        }
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        let available_size = Vector { x: available_width.unwrap_or(0), y: available_height.unwrap_or(0) };
        let children_size = Thickness::all(1).shrink_rect_size(available_size);
        let children_width = if available_width.is_none() { None } else { Some(children_size.x) };
        let children_height = if available_height.is_none() { None } else { Some(children_size.y) };
        let mut size = Vector::null();
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.measure(tree, children_width, children_height, state);
                size = size.max(child.desired_size(tree));
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        Thickness::all(1).expand_rect_size(size)
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        let children_bounds = Thickness::all(1).shrink_rect(final_inner_bounds);
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.arrange(tree, children_bounds, state);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        final_inner_bounds.size
    }

    fn update(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        _event: Event,
        _event_source: Window<State>,
        _state: &mut State,
    ) -> bool {
        false
    }
}
