use crate::view::base::*;
use components_arena::ComponentId;
use dep_obj::{DepObjBaseBuilder, Style, dep_type, dep_type_with_builder};
use dep_obj::binding::{Binding, Binding1};
use dyn_context::state::{State, StateExt};
use std::fmt::Debug;
use tuifw_screen_base::{Point, Rect, Vector};

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
        CanvasPanel::new(self.state_mut(), view);
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
        let child = View::new(self.0.state_mut(), view, |child| (tag, child));
        storage.map(|x| x.replace(child));
        CanvasLayout::new(self.0.state_mut(), child);
        child.build(self.0.state_mut(), |child_builder| {
            let child_builder = layout(CanvasLayoutBuilder::new_priv(child_builder)).base_priv();
            f(child_builder)
        });
        self
    }
}

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct CanvasLayout become layout in View {
        tl: Point = Point { x: 0, y: 0 },
    }

    type BaseBuilder<'a> = ViewBuilder<'a>;
}

impl CanvasLayout {
    const BEHAVIOR: CanvasLayoutBehavior = CanvasLayoutBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        state: &mut dyn State,
        view: View,
    ) {
        view.set_layout(state, CanvasLayout::new_priv());
    }
}

impl Layout for CanvasLayout {
    fn behavior(&self) -> &'static dyn LayoutBehavior { &Self::BEHAVIOR }
}

struct CanvasLayoutBehavior;

impl LayoutBehavior for CanvasLayoutBehavior {
    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn LayoutBindings> {
        let tl = Binding1::new(state, (), |(), (_, tl)| Some(tl));
        tl.set_source_1(state, &mut CanvasLayout::TL.source(view.layout()));
        tl.set_target_fn(state, view, |state, view, _| {
            let tree: &ViewTree = state.get();
            view.parent(tree).map(|parent| parent.invalidate_arrange(state)).expect("invalidate_arrange failed");
        });
        Box::new(CanvasLayoutBindings {
            tl: tl.into()
        })
    }

    fn drop_bindings(&self, _view: View, state: &mut dyn State, bindings: Box<dyn LayoutBindings>) {
        let bindings = bindings.downcast::<CanvasLayoutBindings>().unwrap();
        bindings.tl.drop_binding(state);
    }
}

#[derive(Debug)]
struct CanvasLayoutBindings {
    tl: Binding<Point>,
}

impl LayoutBindings for CanvasLayoutBindings { }

#[derive(Debug, Clone)]
struct CanvasPanelTemplate {
    layout: Style<CanvasLayout>,
}

impl PanelTemplate for CanvasPanelTemplate {
    fn apply_panel(&self, state: &mut dyn State, view: View) {
        CanvasPanel::new(state, view);
    }

    fn apply_layout(&self, state: &mut dyn State, view: View) {
        CanvasLayout::new(state, view);
        view.layout().apply_style(state, Some(self.layout.clone()));
    }
}

dep_type! {
    #[derive(Debug)]
    pub struct CanvasPanel in View { }
}

impl CanvasPanel {
    const BEHAVIOR: CanvasPanelBehavior = CanvasPanelBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        state: &mut dyn State,
        view: View,
    ) {
        view.set_panel(state, CanvasPanel::new_priv());
    }

    pub fn template(layout: Style<CanvasLayout>) -> Box<dyn PanelTemplate> {
        Box::new(CanvasPanelTemplate { layout })
    }
}

impl Panel for CanvasPanel {
    fn behavior(&self) -> &'static dyn PanelBehavior { &Self::BEHAVIOR }
}

#[derive(Debug)]
struct CanvasPanelBindings;

impl PanelBindings for CanvasPanelBindings { }

struct CanvasPanelBehavior;

impl PanelBehavior for CanvasPanelBehavior {
    fn children_desired_size(
        &self,
        view: View,
        state: &mut dyn State,
        children_measure_size: (Option<i16>, Option<i16>)
    ) -> Vector {
        let tree: &ViewTree = state.get();
        if let Some(last_child) = view.last_child(tree) {
            let mut child = last_child;
            loop {
                let tree: &ViewTree = state.get();
                child = child.next(tree);
                child.measure(state, (None, None));
                if child == last_child { break; }
            }
        }
        Vector { x: children_measure_size.0.unwrap_or(0), y: children_measure_size.1.unwrap_or(0) }
    }

    fn children_render_bounds(
        &self,
        view: View,
        state: &mut dyn State,
        children_arrange_bounds: Rect
    ) -> Rect {
        let tree: &ViewTree = state.get();
        if let Some(last_child) = view.last_child(tree) {
            let mut child = last_child;
            loop {
                let tree: &ViewTree = state.get();
                child = child.next(tree);
                let child_offset = child.layout_bindings(tree).downcast_ref::<CanvasLayoutBindings>().unwrap().tl
                    .get_value(state).map_or_else(Vector::null, |x| x.offset_from(Point { x: 0, y: 0 }));
                let tree: &ViewTree = state.get();
                let child_size = child.desired_size(tree);
                child.arrange(state, Rect {
                    tl: children_arrange_bounds.tl.offset(child_offset),
                    size: child_size
                });
                if child == last_child { break; }
            }
        }
        children_arrange_bounds
    }

    fn init_bindings(&self, _view: View, _state: &mut dyn State) -> Box<dyn PanelBindings> {
        Box::new(CanvasPanelBindings)
    }

    fn drop_bindings(&self, _view: View, _state: &mut dyn State, _bindings: Box<dyn PanelBindings>) { }
}
