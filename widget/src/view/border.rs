use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use crate::property::Property;
use crate::view::base::*;

#[derive(Debug)]
struct BorderRender;

#[derive(Debug)]
pub struct BorderView {
    view: View,
    tl: Property<Self, Option<Text>, ViewContext>,
    tr: Property<Self, Option<Text>, ViewContext>,
    bl: Property<Self, Option<Text>, ViewContext>,
    br: Property<Self, Option<Text>, ViewContext>,
    l: Property<Self, Option<Text>, ViewContext>,
    t: Property<Self, Option<Text>, ViewContext>,
    r: Property<Self, Option<Text>, ViewContext>,
    b: Property<Self, Option<Text>, ViewContext>
}

impl BorderView {
    pub fn new(
        tree: &mut ViewTree,
        parent: View,
    ) -> View {
        View::new(tree, parent, Some(Box::new(BorderRender) as _), |view| {
            let mut obj = BorderView {
                view,
                tl: Property::new(None),
                tr: Property::new(None),
                bl: Property::new(None),
                br: Property::new(None),
                l: Property::new(None),
                t: Property::new(None),
                r: Property::new(None),
                b: Property::new(None),
            };
            obj.on_tl_changed(Self::invalidate_tl);
            obj.on_tr_changed(Self::invalidate_tr);
            obj.on_bl_changed(Self::invalidate_bl);
            obj.on_br_changed(Self::invalidate_br);
            obj.on_l_changed(Self::invalidate_l);
            obj.on_t_changed(Self::invalidate_t);
            obj.on_r_changed(Self::invalidate_r);
            obj.on_b_changed(Self::invalidate_b);
            (Box::new(obj) as _, view)
        })
    }

    property!(Option<Text>, tl, set_tl, on_tl_changed, ViewContext);
    property!(Option<Text>, tr, set_tr, on_tr_changed, ViewContext);
    property!(Option<Text>, bl, set_bl, on_bl_changed, ViewContext);
    property!(Option<Text>, br, set_br, on_br_changed, ViewContext);
    property!(Option<Text>, l, set_l, on_l_changed, ViewContext);
    property!(Option<Text>, t, set_t, on_t_changed, ViewContext);
    property!(Option<Text>, r, set_r, on_r_changed, ViewContext);
    property!(Option<Text>, b, set_b, on_b_changed, ViewContext);

    fn invalidate_tl(&mut self, context: &mut ViewContext, _old: &Option<Text>) {
        let tree = context.get_1();
        self.view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_tr(&mut self, context: &mut ViewContext, _old: &Option<Text>) {
        let tree = context.get_1();
        let size = self.view.size(tree).unwrap();
        self.view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_bl(&mut self, context: &mut ViewContext, _old: &Option<Text>) {
        let tree = context.get_1();
        let size = self.view.size(tree).unwrap();
        self.view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_br(&mut self, context: &mut ViewContext, _old: &Option<Text>) {
        let tree = context.get_1();
        let size = self.view.size(tree).unwrap();
        self.view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: 1, y: 1 }
        }).unwrap();
    }

    fn invalidate_l(&mut self, context: &mut ViewContext, _old: &Option<Text>) {
        let tree = context.get_1();
        let size = self.view.size(tree).unwrap();
        self.view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 1, y: size.y }
        }).unwrap();
    }

    fn invalidate_t(&mut self, context: &mut ViewContext, _old: &Option<Text>) {
        let tree = context.get_1();
        let size = self.view.size(tree).unwrap();
        self.view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: size.x, y: 1 }
        }).unwrap();
    }

    fn invalidate_r(&mut self, context: &mut ViewContext, _old: &Option<Text>) {
        let tree = context.get_1();
        let size = self.view.size(tree).unwrap();
        self.view.invalidate_rect(tree, Rect {
            tl: Point { x: size.x.overflowing_sub(1).0, y: 0 },
            size: Vector { x: 1, y: size.y }
        }).unwrap();
    }

    fn invalidate_b(&mut self, context: &mut ViewContext, _old: &Option<Text>) {
        let tree = context.get_1();
        let size = self.view.size(tree).unwrap();
        self.view.invalidate_rect(tree, Rect {
            tl: Point { x: 0, y: size.y.overflowing_sub(1).0 },
            size: Vector { x: size.x, y: 1 }
        }).unwrap();
    }
}

impl ViewObj for BorderView {
    fn client_bounds(&self, _tree: &ViewTree, size: Vector) -> Rect {
        let tl = Point {
            x: if self.l().is_some() || self.tl().is_some() || self.bl().is_some() { 1 } else { 0 },
            y: if self.t().is_some() || self.tl().is_some() || self.tr().is_some() { 1 } else { 0 },
        };
        let br = Vector {
            x: if self.r().is_some() || self.tr().is_some() || self.br().is_some() { -1 } else { 0 },
            y: if self.t().is_some() || self.tl().is_some() || self.tr().is_some() { -1 } else { 0 },
        };
        Rect { tl, size: size + br }.intersect(Rect { tl: Point { x: 0, y: 0 }, size })
    }
}

impl Render for BorderRender {
    fn render(&self, tree: &ViewTree, view: View, port: &mut RenderPort) {
        let size = view.size(tree).unwrap();
        let obj = view.obj(tree).downcast_ref::<BorderView>().unwrap();
        let l = obj.l().as_ref().or_else(|| if obj.tl().is_some() || obj.bl().is_some() { Some(&Text::SPACE) } else { None });
        let t = obj.t().as_ref().or_else(|| if obj.tl().is_some() || obj.tr().is_some() { Some(&Text::SPACE) } else { None });
        let r = obj.r().as_ref().or_else(|| if obj.tr().is_some() || obj.br().is_some() { Some(&Text::SPACE) } else { None });
        let b = obj.b().as_ref().or_else(|| if obj.bl().is_some() || obj.br().is_some() { Some(&Text::SPACE) } else { None });
        if let Some(l) = l {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: 0, y: y as i16 }, l.fg, l.bg, l.attr, &l.value);
            }
        }
        if let Some(r) = r {
            for y in 0 .. size.y as u16 {
                port.out(Point { x: size.x.overflowing_sub(1).0, y: y as i16 }, r.fg, r.bg, r.attr, &r.value);
            }
        }
        if let Some(t) = t {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: 0 }, t.fg, t.bg, t.attr, &t.value);
            }
        }
        if let Some(b) = b {
            for x in 0 .. size.x as u16 {
                port.out(Point { x: x as i16, y: size.y.overflowing_sub(1).0 }, b.fg, b.bg, b.attr, &b.value);
            }
        }
        if let Some(tl) = obj.tl() {
            port.out(Point { x: 0, y: 0 }, tl.fg, tl.bg, tl.attr, &tl.value);
        }
        if let Some(tr) = obj.tr() {
            port.out(Point { x: size.x.overflowing_sub(1).0, y: 0 }, tr.fg, tr.bg, tr.attr, &tr.value);
        }
        if let Some(bl) = obj.bl() {
            port.out(Point { x: 0, y: size.y.overflowing_sub(1).0 }, bl.fg, bl.bg, bl.attr, &bl.value);
        }
        if let Some(br) = obj.br() {
            let p = Point { x: size.x.overflowing_sub(1).0, y: size.y.overflowing_sub(1).0 };
            port.out(p, br.fg, br.bg, br.attr, &br.value);
        }
    }
}
