use crate::view::base::*;
use components_arena::ComponentId;
use dep_obj::{dep_type_with_builder, DepObjBuilderCore};
use dyn_context::{Context, ContextExt};
use either::{Either, Left, Right};
use std::cmp::{min, max};
use std::fmt::Debug;
use std::hint::unreachable_unchecked;
use tuifw_screen_base::{Vector, Rect, Side, Orient, Thickness};

pub trait ViewBuilderDockPanelExt {
    fn dock_panel(
        self,
        f: impl for<'a> FnOnce(DockPanelBuilder<'a>) -> DockPanelBuilder<'a>
    ) -> Self;
}

impl<'a> ViewBuilderDockPanelExt for ViewBuilder<'a> {
    fn dock_panel(
        mut self,
        f: impl for<'b> FnOnce(DockPanelBuilder<'b>) -> DockPanelBuilder<'b>
    ) -> Self {
        let view = self.id();
        let tree: &mut ViewTree = self.context_mut().get_mut();
        DockPanel::new(tree, view);
        f(DockPanelBuilder::new_priv(self)).core_priv()
    }
}

impl<'a> DockPanelBuilder<'a> {
    pub fn child<Tag: ComponentId>(
        mut self,
        storage: Option<&mut Option<View>>,
        tag: Tag,
        layout: impl for<'b> FnOnce(DockLayoutBuilder<'b>) -> DockLayoutBuilder<'b>,
        f: impl for<'b> FnOnce(ViewBuilder<'b>) -> ViewBuilder<'b>
    ) -> Self {
        let view = self.core_priv_ref().id();
        let tree: &mut ViewTree = self.core_priv_mut().context_mut().get_mut();
        let child = View::new(tree, view, |child| (tag, child));
        storage.map(|x| x.replace(child));
        DockLayout::new(tree, child);
        child.build(self.core_priv_mut().context_mut(), |child_builder| {
            let child_builder = layout(DockLayoutBuilder::new_priv(child_builder)).core_priv();
            f(child_builder)
        });
        self
    }
}

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct DockLayout become layout in View {
        dock: Either<f32, Side> = Either::Left(1.),
    }

    type BuilderCore<'a> = ViewBuilder<'a>;
}

impl DockLayout {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_layout(tree, DockLayout::new_priv());
        view.layout(tree).on_changed(DockLayout::DOCK, Self::invalidate_parent_measure);
    }

    fn invalidate_parent_measure<T>(context: &mut dyn Context, view: View, _old: &T) {
        let tree: &mut ViewTree = context.get_mut();
        view.parent(tree).map(|parent| parent.invalidate_measure(tree));
    }
}

impl Layout for DockLayout { }

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct DockPanel become panel in View {
        base: Side = Side::Top,
    }

    type BuilderCore<'a> = ViewBuilder<'a>;
}

impl DockPanel {
    const BEHAVIOR: DockPanelBehavior = DockPanelBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        view: View,
    ) {
        view.set_panel(tree, DockPanel::new_priv());
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
        children_measure_size: (Option<i16>, Option<i16>)
    ) -> Vector {
        let mut children_size = Vector::null();
        if let Some(last_child) = view.last_child(tree) {
            let mut size = children_measure_size;
            let mut last_undocked_child = None;
            let mut factor_sum = 0.;
            let mut orient = None;
            let mut breadth = 0;
            let mut child = last_child;
            loop {
                child = child.next(tree);
                let dock = match child.layout_ref(tree).get(DockLayout::DOCK) {
                    &Right(dock) => dock,
                    &Left(factor) => {
                        factor_sum += factor;
                        last_undocked_child = Some(child);
                        continue;
                    }
                };
                let w = if dock == Side::Left || dock == Side::Right { None } else { size.0 };
                let h = if dock == Side::Top || dock == Side::Bottom { None } else { size.1 };
                child.measure(tree, (w, h));
                let child_size = child.desired_size(tree);
                let (child_orient, child_breadth) = match dock {
                    Side::Left | Side::Right => {
                        size.0.as_mut().map(|w| *w = (*w as u16).saturating_sub(child_size.x as u16) as i16);
                        children_size.x = (children_size.x as u16).saturating_add(child_size.x as u16) as i16;
                        (Orient::Hor, child_size.y as u16)
                    },
                    Side::Top | Side::Bottom => {
                        size.1.as_mut().map(|h| *h = (*h as u16).saturating_sub(child_size.y as u16) as i16);
                        children_size.y = (children_size.y as u16).saturating_add(child_size.y as u16) as i16;
                        (Orient::Vert, child_size.x as u16)
                    }
                };
                if Some(child_orient) == orient {
                    breadth = max(breadth, child_breadth);
                } else {
                    if let Some(orient) = orient {
                        match orient {
                            Orient::Hor => children_size.y = (children_size.y as u16).saturating_add(breadth) as i16,
                            Orient::Vert => children_size.x = (children_size.x as u16).saturating_add(breadth) as i16,
                        }
                    }
                    orient = Some(child_orient);
                    breadth = child_breadth;
                }
                if child == last_child { break; }
            }
            match orient.unwrap_or_else(|| unsafe { unreachable_unchecked() }) {
                Orient::Hor => children_size.y = (children_size.y as u16).saturating_add(breadth) as i16,
                Orient::Vert => children_size.x = (children_size.x as u16).saturating_add(breadth) as i16,
            }
            if let Some(last_undocked_child) = last_undocked_child {
                let orient = match view.panel_ref(tree).get(DockPanel::BASE) {
                    Side::Left | Side::Right => Orient::Hor,
                    Side::Top | Side::Bottom => Orient::Vert
                };
                let mut breadth = 0u16;
                let mut length = 0u16;
                let mut size = (size.0.map(|w| (w, w)), size.1.map(|h| (h, h)));
                let mut child = last_child;
                loop {
                    child = child.next(tree);
                    let factor = match child.layout_ref(tree).get(DockLayout::DOCK) {
                        &Right(_) => continue,
                        &Left(factor) => factor
                    };
                    let child_size = match orient {
                        Orient::Hor => {
                            let child_w = size.0.as_mut().map(|&mut (w, ref mut last_w)| if child == last_undocked_child {
                                *last_w
                            } else {
                                let child_w = frac(factor, factor_sum, w);
                                *last_w = (*last_w as u16).saturating_sub(child_w as u16) as i16;
                                child_w
                            });
                            (child_w, size.1.map(|x| x.0))
                        },
                        Orient::Vert => {
                            let child_h = size.1.as_mut().map(|&mut (h, ref mut last_h)| if child == last_undocked_child {
                                *last_h
                            } else {
                                let child_h = frac(factor, factor_sum, h);
                                *last_h = (*last_h as u16).saturating_sub(child_h as u16) as i16;
                                child_h
                            });
                            (size.0.map(|x| x.0), child_h)
                        },
                    };
                    child.measure(tree, child_size);
                    let child_size = child.desired_size(tree);
                    match orient {
                        Orient::Hor => {
                            length = length.saturating_add(child_size.x as u16);
                            breadth = max(breadth, child_size.y as u16);
                        },
                        Orient::Vert => {
                            length = length.saturating_add(child_size.y as u16);
                            breadth = max(breadth, child_size.x as u16);
                        }
                    }
                    if child == last_undocked_child { break; }
                }
                match orient {
                    Orient::Hor => {
                        children_size.x = (children_size.x as u16).saturating_add(length) as i16;
                        children_size.y = (children_size.y as u16).saturating_add(breadth) as i16;
                    },
                    Orient::Vert => {
                        children_size.y = (children_size.y as u16).saturating_add(length) as i16;
                        children_size.x = (children_size.x as u16).saturating_add(breadth) as i16;
                    },
                }
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
        let mut children_rect = Rect { tl: children_arrange_bounds.tl, size: Vector::null() };
        if let Some(last_child) = view.last_child(tree) {
            let mut last_undocked_child = None;
            let mut bounds = children_arrange_bounds;
            let mut factor_sum = 0.;
            let mut child = last_child;
            loop {
                child = child.next(tree);
                let dock = match child.layout_ref(tree).get(DockLayout::DOCK) {
                    &Right(dock) => dock,
                    &Left(factor) => {
                        factor_sum += factor;
                        last_undocked_child = Some(child);
                        continue;
                    }
                };
                let child_size = child.desired_size(tree);
                let child_size = match dock {
                    Side::Left => Vector { x: child_size.x, y: bounds.h() },
                    Side::Right => Vector { x: child_size.x, y: bounds.h() },
                    Side::Top => Vector { y: child_size.y, x: bounds.w() },
                    Side::Bottom => Vector { y: child_size.y, x: bounds.w() },
                };
                let child_tl = match dock {
                    Side::Left | Side::Top => bounds.tl,
                    Side::Right => bounds.tr().offset(-Vector { x: child_size.x, y: 0 }),
                    Side::Bottom => bounds.bl().offset(-Vector { y: child_size.y, x: 0 }),
                };
                let child_rect = Rect { tl: child_tl, size: child_size };
                child.arrange(tree, child_rect);
                children_rect = children_rect.union_intersect(child_rect, children_arrange_bounds);
                let d = match dock {
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
            if let Some(last_undocked_child) = last_undocked_child {
                children_rect = children_arrange_bounds;
                let base_bounds = bounds;
                let &dock = view.panel_ref(tree).get(DockPanel::BASE);
                let mut child = last_child;
                loop {
                    child = child.next(tree);
                    let factor = match child.layout_ref(tree).get(DockLayout::DOCK) {
                        &Right(_) => continue,
                        &Left(factor) => factor
                    };
                    let child_size = if child == last_undocked_child {
                        bounds.size
                    } else {
                        match dock {
                            Side::Left | Side::Right =>
                                Vector { x: frac(factor, factor_sum, base_bounds.w()), y: bounds.h() },
                            Side::Top | Side::Bottom =>
                                Vector { y: frac(factor, factor_sum, base_bounds.h()), x: bounds.w() },
                        }
                    };
                    let child_tl = match dock {
                        Side::Left | Side::Top => bounds.tl,
                        Side::Right => bounds.tr().offset(-Vector { x: child_size.x, y: 0 }),
                        Side::Bottom => bounds.bl().offset(-Vector { y: child_size.y, x: 0 }),
                    };
                    let child_rect = Rect { tl: child_tl, size: child_size };
                    child.arrange(tree, child_rect);
                    let d = match dock {
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
                    if child == last_undocked_child { break; }
                }
            }
        }
        children_rect
    }
}

fn frac(numerator: f32, denominator: f32, scale: i16) -> i16 {
    let frac = numerator * (scale as u16 as f32) / denominator;
    if !frac.is_finite() || frac < 0. || frac > scale as u16 as f32 { return 0; }
    frac.round() as u16 as i16
}
