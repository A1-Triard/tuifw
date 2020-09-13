use std::cmp::{min};
use std::fmt::Debug;
use tuifw_screen_base::{Vector, Rect, Side, Orient, Thickness};
use components_arena::ComponentId;
use dep_obj::{dep_obj, DepTypeToken};
use dyn_context::{Context, ContextExt};
use once_cell::sync::{self};
use crate::view::base::*;

pub trait ViewBuilderDockPanelExt {
    fn dock_panel(
        &mut self,
        f: impl for<'a, 'b, 'c> FnOnce(&'a mut DockPanelBuilder<'b, 'c>) -> &'a mut DockPanelBuilder<'b, 'c>
    ) -> &mut Self;
}

impl<'a> ViewBuilderDockPanelExt for ViewBuilder<'a> {
    fn dock_panel(
        &mut self,
        f: impl for<'b, 'c, 'd> FnOnce(&'b mut DockPanelBuilder<'c, 'd>) -> &'b mut DockPanelBuilder<'c, 'd>
    ) -> &mut Self {
        let mut builder = DockPanelBuilder::new_priv(self);
        f(&mut builder);
        self
    }
}

impl<'a, 'b> DockPanelBuilder<'a, 'b> {
    pub fn child<Tag: ComponentId>(
        &mut self,
        tag: Tag,
        layout: impl FnOnce(&mut DockLayoutBuilder) -> &mut DockLayoutBuilder,
        f: impl for<'c, 'd> FnOnce(&'c mut ViewBuilder<'d>) -> &'c mut ViewBuilder<'d>
    ) -> &mut Self {
        let view = self.core_priv().view();
        let tree: &mut ViewTree = self.core_priv_mut().context().get_mut();
        let child = View::new(tree, view, |child| (tag, child));
        DockLayout::new(tree, child);
        let mut builder = DockLayoutBuilder::new_priv();
        layout(&mut builder);
        builder.build_priv(self.core_priv_mut().context(), child, dock_layout_type());
        child.build(self.core_priv_mut().context(), f);
        self
    }
}

dep_obj! {
    #[derive(Debug)]
    pub struct DockLayout become layout in View {
        dock: Option<Side> = None,
    }
}

static DOCK_LAYOUT_TOKEN: sync::Lazy<DepTypeToken<DockLayoutType>> = sync::Lazy::new(||
    DockLayoutType::new_priv().expect("DockLayoutType builder locked")
);

pub fn dock_layout_type() -> &'static DockLayoutType { DOCK_LAYOUT_TOKEN.ty() }

impl DockLayout {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_layout(tree, DockLayout::new_priv(&DOCK_LAYOUT_TOKEN));
        view.layout_on_changed(tree, dock_layout_type().dock(), Self::invalidate_parent_measure);
    }

    fn invalidate_parent_measure<T>(context: &mut dyn Context, view: View, _old: &T) {
        let tree: &mut ViewTree = context.get_mut();
        view.parent(tree).map(|parent| parent.invalidate_measure(tree));
    }
}

impl Layout for DockLayout { }

dep_obj! {
    #[derive(Debug)]
    pub struct DockPanel become panel in View where BuilderCore<'a, 'b> = &'a mut ViewBuilder<'b>{
        base: Side = Side::Top,
    }
}

static DOCK_PANEL_TOKEN: sync::Lazy<DepTypeToken<DockPanelType>> = sync::Lazy::new(||
    DockPanelType::new_priv().expect("DockPanelType builder locked")
);

pub fn dock_panel_type() -> &'static DockPanelType { DOCK_PANEL_TOKEN.ty() }

impl DockPanel {
    const BEHAVIOR: DockPanelBehavior = DockPanelBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_panel(tree, DockPanel::new_priv(&DOCK_PANEL_TOKEN));
    }
}

impl Panel for DockPanel {
    fn behavior(&self) -> &'static dyn PanelBehavior { &Self::BEHAVIOR }
}

struct DockPanelBehavior;

impl PanelBehavior for DockPanelBehavior {
    fn children_desired_size(
        &self,
        view: View,
        tree: &mut ViewTree,
        mut size: (Option<i16>, Option<i16>)
    ) -> Vector {
        let mut children_size = Vector::null();
        if let Some(last_child) = view.last_child(tree) {
            let mut child = last_child;
            loop {
                child = child.next(tree);
                let &dock = child.layout_get(tree, dock_layout_type().dock());
                let w = if dock == Some(Side::Left) || dock == Some(Side::Right) { None } else { size.0 };
                let h = if dock == Some(Side::Top) || dock == Some(Side::Bottom) { None } else { size.1 };
                child.measure(tree, (w, h));
                let child_size = child.desired_size(tree);
                let orient = match dock.unwrap_or_else(|| *view.panel_get(tree, dock_panel_type().base())) {
                    Side::Left | Side::Right => Orient::Hor,
                    Side::Top | Side::Bottom => Orient::Vert,
                };
                if orient == Orient::Hor {
                    size.0.as_mut().map(|w| *w = (*w as u16).saturating_sub(child_size.x as u16) as i16);
                    children_size.x = (children_size.x as u16).saturating_add(child_size.x as u16) as i16;
                } else {
                    size.1.as_mut().map(|h| *h = (*h as u16).saturating_sub(child_size.y as u16) as i16);
                    children_size.y = (children_size.y as u16).saturating_add(child_size.y as u16) as i16;
                }
                if child == last_child { break; }
            }
        }
        children_size
    }

    fn children_render_bounds(
        &self,
        view: View,
        tree: &mut ViewTree,
        children_arrange_bounds: Rect
    ) -> Rect {
        let mut bounds = children_arrange_bounds;
        let mut children_rect = Rect { tl: bounds.tl, size: Vector::null() };
        if let Some(last_child) = view.last_child(tree) {
            let mut child = last_child;
            loop {
                child = child.next(tree);
                let child_size = child.desired_size(tree);
                let dock = child.layout_get(tree, dock_layout_type().dock());
                let child_size = match dock {
                    Some(Side::Left) => Vector { x: child_size.x, y: bounds.h() },
                    Some(Side::Right) => Vector { x: child_size.x, y: bounds.h() },
                    Some(Side::Top) => Vector { y: child_size.y, x: bounds.w() },
                    Some(Side::Bottom) => Vector { y: child_size.y, x: bounds.w() },
                    None => bounds.size,
                };
                let base = dock.unwrap_or_else(|| *view.panel_get(tree, dock_panel_type().base()));
                let child_tl = match base {
                    Side::Left | Side::Top => bounds.tl,
                    Side::Right => bounds.tr().offset(-Vector { x: child_size.x, y: 0 }),
                    Side::Bottom => bounds.bl().offset(-Vector { y: child_size.y, x: 0 }),
                };
                let child_rect = Rect { tl: child_tl, size: child_size };
                child.arrange(tree, child_rect);
                children_rect = children_rect.union_intersect(child_rect, children_arrange_bounds);
                let d = match base {
                    Side::Left => unsafe { Thickness::new_unchecked(
                        min(child_rect.w() as u16, bounds.w() as u16) as u32 as i32, 0, 0, 0
                    ) },
                    Side::Right => unsafe { Thickness::new_unchecked(
                        0, 0, min(child_rect.w() as u16, bounds.w() as u16) as u32 as i32, 0
                    ) },
                    Side::Top => unsafe { Thickness::new_unchecked(
                        0, min(child_rect.h() as u16, bounds.h() as u16) as u32 as i32, 0, 0
                    ) },
                    Side::Bottom => unsafe { Thickness::new_unchecked(
                        0, 0, 0, min(child_rect.h() as u16, bounds.h() as u16) as u32 as i32
                    ) },
                };
                bounds = d.shrink_rect(bounds);
                if child == last_child { break; }
            }
        }
        children_rect
    }
}
