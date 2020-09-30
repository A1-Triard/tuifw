use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use components_arena::ComponentId;
use dep_obj::{dep_type, DepObjBuilderCore};
use dyn_context::{Context, ContextExt};
use crate::view::base::*;

pub trait ViewBuilderCanvasPanelExt {
    fn canvas_panel(
        self,
        f: impl for<'a> FnOnce(CanvasPanelBuilder<'a>) -> CanvasPanelBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderCanvasPanelExt for ViewBuilder<'a> {
    fn canvas_panel(
        mut self,
        f: impl for<'b> FnOnce(CanvasPanelBuilder<'b>) -> CanvasPanelBuilder<'b>
    ) -> Self {
        let view = self.id();
        let tree: &mut ViewTree = self.context_mut().get_mut();
        CanvasPanel::new(tree, view);
        f(CanvasPanelBuilder(self)).0
    }
}

pub struct CanvasPanelBuilder<'a>(ViewBuilder<'a>);

impl<'a> CanvasPanelBuilder<'a> {
    pub fn child<Tag: ComponentId>(
        mut self,
        storage: Option<&mut Option<View>>,
        tag: Tag,
        layout: impl for<'b> FnOnce(CanvasLayoutBuilder<'b>) -> CanvasLayoutBuilder<'b>,
        f: impl for<'b> FnOnce(ViewBuilder<'b>) -> ViewBuilder<'b>
    ) -> Self {
        let view = self.0.id();
        let tree: &mut ViewTree = self.0.context_mut().get_mut();
        let child = View::new(tree, view, |child| (tag, child));
        storage.map(|x| x.replace(child));
        CanvasLayout::new(tree, child);
        child.build(self.0.context_mut(), |child_builder| {
            let child_builder = layout(CanvasLayoutBuilder::new_priv(child_builder)).core_priv();
            f(child_builder)
        });
        self
    }
}

dep_type! {
    #[derive(Debug)]
    pub struct CanvasLayout become layout in View {
        tl: Point = Point { x: 0, y: 0 },
    }

    type BuilderCore<'a> = ViewBuilder<'a>;
}

impl CanvasLayout {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_layout(tree, CanvasLayout::new_priv());
        view.layout_on_changed(tree, CanvasLayout::TL, Self::invalidate_parent_arrange);
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
                let child_offset = child.layout_get(tree, CanvasLayout::TL)
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
