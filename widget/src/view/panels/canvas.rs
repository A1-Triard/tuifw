use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use dep_obj::{DepPropRaw, DepObjProps, DepTypeBuilder, DepProp, DepTypeToken, DepObj};
use dep_obj::{Context, ContextExt};
use once_cell::sync::{self};
use crate::view::base::*;

macro_attr! {
    #[derive(DepType!)]
    pub struct CanvasLayoutType {
        tl: DepPropRaw<Self, Point>,
    }
}

impl CanvasLayoutType {
    pub fn tl(&self) -> DepProp<CanvasLayout, Point> { self.tl.owned_by() }
}

pub static CANVAS_LAYOUT_TOKEN: sync::Lazy<DepTypeToken<CanvasLayoutType>> = sync::Lazy::new(|| {
    let mut builder = DepTypeBuilder::new().expect("CanvasLayoutType builder locked");
    let tl = builder.prop(|| Point { x: 0, y: 0 });
    builder.build(CanvasLayoutType {
        tl,
    })
});

pub fn canvas_layout_type() -> &'static CanvasLayoutType { CANVAS_LAYOUT_TYPE.type_() }

#[derive(Debug)]
pub struct CanvasLayout {
    dep_props: DepObjProps<CanvasLayoutType, View>,
}

impl CanvasLayout {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_layout(tree, CanvasLayout {
            dep_props: DepObjProps::new(&CANVAS_LAYOUT_TOKEN)
        });
        view.layout_on_changed(tree, canvas_layout_type().tl(), Self::invalidate_parent_arrange);
    }

    fn invalidate_parent_arrange(view: View, context: &mut dyn Context, _old: &Point) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        view.parent(tree).map(|parent| parent.invalidate_arrange(tree));
    }
}

pub struct CanvasPanel(());

impl CanvasPanel {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_panel(tree, CanvasPanel(()));
    }
}

impl Panel for CanvasPanel {
    fn behavior(&self) -> &'static dyn PanelBehavior {
        static BEHAVIOR: CanvasPanelBehavior = CanvasPanelBehavior;
        &BEHAVIOR
    }
}

struct CanvasPanelBehavior;

impl PanelBehavior for CanvasPanelBehavior {
    fn children_desired_size(
        &self,
        view: View,
        tree: &mut ViewTree,
        children_measure_size: (Option<i16>, Option<i16>)
    ) -> Vector {
        if let Some(last_child) = view.last_child(tree) {
            let mut child = last_child;
            loop {
                child = child.next(tree);
                child.measure(tree, (None, None));
                if child == last_child { break; }
            }
        }
        Vector { x: children_measure_size.0.unwrap_or(0), y: children_measure_size.1.unwrap_or(0) }
    }

    fn children_render_bounds(
        &self,
        view: View,
        tree: &mut ViewTree,
        children_arrange_bounds: Rect
    ) -> Rect {
        if let Some(last_child) = self.last_child(tree) {
            let mut child = last_child;
            loop {
                child = child.next(tree);
                let child_offset = child.layout_get(tree, canvas_layout_type().tl())
                    .offset_from(Point { x: 0, y: 0 });
                let child_size = child.desired_size(tree);
                child.arrange(tree, Rect {
                    tl: children_arrange_bounds.tl.offset(child_offset),
                    size: child_size
                });
                if child == last_child { break; }
            }
        }
        children_arrange_bounds
    }
}
