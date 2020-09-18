use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use components_arena::ComponentId;
use dep_obj::{dep_obj, DepTypeToken};
use dyn_context::{Context, ContextExt};
use once_cell::sync::{self};
use crate::view::base::*;

pub trait ViewBuilderCanvasPanelExt {
    fn canvas_panel(
        &mut self,
        f: impl for<'a, 'b, 'c> FnOnce(&'a mut CanvasPanelBuilder<'b, 'c>) -> &'a mut CanvasPanelBuilder<'b, 'c>
    ) -> &mut Self;
}

impl<'a> ViewBuilderCanvasPanelExt for ViewBuilder<'a> {
    fn canvas_panel(
        &mut self,
        f: impl for<'b, 'c, 'd> FnOnce(&'b mut CanvasPanelBuilder<'c, 'd>) -> &'b mut CanvasPanelBuilder<'c, 'd>
    ) -> &mut Self {
        let view = self.view();
        let tree: &mut ViewTree = self.context().get_mut();
        CanvasPanel::new(tree, view);
        let mut builder = CanvasPanelBuilder(self);
        f(&mut builder);
        self
    }
}

pub struct CanvasPanelBuilder<'a, 'b>(&'a mut ViewBuilder<'b>);

impl<'a, 'b> CanvasPanelBuilder<'a, 'b> {
    pub fn child<Tag: ComponentId>(
        &mut self,
        storage: Option<&mut Option<View>>,
        tag: Tag,
        layout: impl for<'c, 'd, 'e> FnOnce(&'c mut CanvasLayoutBuilder<'d, 'e>) -> &'c mut CanvasLayoutBuilder<'d, 'e>,
        f: impl for<'c, 'd> FnOnce(&'c mut ViewBuilder<'d>) -> &'c mut ViewBuilder<'d>
    ) -> &mut Self {
        let view = self.0.view();
        let tree: &mut ViewTree = self.0.context().get_mut();
        let child = View::new(tree, view, |child| (tag, child));
        storage.map(|x| x.replace(child));
        CanvasLayout::new(tree, child);
        CanvasLayoutBuilder::build_priv(self.0, child, canvas_layout_type(), layout);
        child.build(self.0.context(), f);
        self
    }
}

dep_obj! {
    #[derive(Debug)]
    pub struct CanvasLayout become layout in View {
        tl: Point = Point { x: 0, y: 0 },
    }

    use<'a, 'b> &'a mut ViewBuilder<'b> as BuilderCore;
}

static CANVAS_LAYOUT_TOKEN: sync::Lazy<DepTypeToken<CanvasLayoutType>> = sync::Lazy::new(||
    CanvasLayoutType::new_priv().expect("CanvasLayoutType builder locked")
);

pub fn canvas_layout_type() -> &'static CanvasLayoutType { CANVAS_LAYOUT_TOKEN.ty() }

impl CanvasLayout {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_layout(tree, CanvasLayout::new_priv(&CANVAS_LAYOUT_TOKEN));
        view.layout_on_changed(tree, canvas_layout_type().tl(), Self::invalidate_parent_arrange);
    }

    fn invalidate_parent_arrange(context: &mut dyn Context, view: View, _old: &Point) {
        let tree: &mut ViewTree = context.get_mut();
        view.parent(tree).map(|parent| parent.invalidate_arrange(tree));
    }
}

impl Layout for CanvasLayout { }

#[derive(Debug)]
pub struct CanvasPanel(());

impl CanvasPanel {
    const BEHAVIOR: CanvasPanelBehavior = CanvasPanelBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_panel(tree, CanvasPanel(()));
    }
}

impl Panel for CanvasPanel {
    fn behavior(&self) -> &'static dyn PanelBehavior { &Self::BEHAVIOR }
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
        if let Some(last_child) = view.last_child(tree) {
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
