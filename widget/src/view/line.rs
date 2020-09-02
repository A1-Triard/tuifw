use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use dep_obj::{DepPropRaw, DepObjProps, DepTypeBuilder, DepProp, DepTypeToken, DepObj};
use dep_obj::{Context, ContextExt};
use once_cell::sync::{self};
use crate::view::base::*;

macro_attr! {
    #[derive(DepType!)]
    pub struct LineDecoratorType {
        orient: DepPropRaw<Self, Orient>,
        length: DepPropRaw<Self, i16>,
        near: DepPropRaw<Self, Option<Text>>,
        stroke: DepPropRaw<Self, Option<Text>>,
        far: DepPropRaw<Self, Option<Text>>,
    }
}

impl LineDecoratorType {
    pub fn orient(&self) -> DepProp<LineDecorator, Orient> { self.orient.owned_by() }
    pub fn length(&self) -> DepProp<LineDecorator, i16> { self.length.owned_by() }
    pub fn near(&self) -> DepProp<LineDecorator, Option<Text>> { self.near.owned_by() }
    pub fn stroke(&self) -> DepProp<LineDecorator, Option<Text>> { self.stroke.owned_by() }
    pub fn far(&self) -> DepProp<LineDecorator, Option<Text>> { self.far.owned_by() }
}

pub static LINE_DECORATOR_TOKEN: sync::Lazy<DepTypeToken<LineDecoratorType>> = sync::Lazy::new(|| {
    let mut builder = DepTypeBuilder::new().expect("LineDecoratorType builder locked");
    let orient = builder.prop(|| Orient::Hor);
    let length = builder.prop(|| 3);
    let near = builder.prop(|| None);
    let stroke = builder.prop(|| None);
    let far = builder.prop(|| None);
    builder.build(LineDecoratorType {
        orient, length, near, stroke, far,
    })
});

pub fn line_decorator_type() -> &'static LineDecoratorType { LINE_DECORATOR_TOKEN.type_() }

#[derive(Debug)]
pub struct LineDecorator {
    dep_props: DepObjProps<LineDecoratorType, View>,
}

impl LineDecorator {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        parent: View,
    ) -> View {
        let view = View::new(tree, parent, |view| {
            let decorator = LineDecorator {
                dep_props: DepObjProps::new(&LINE_DECORATOR_TOKEN)
            };
            (Some(Box::new(decorator) as _), None, view)
        });
        view.decorator_on_changed(tree, line_decorator_type().orient(), Self::invalidate_measure);
        view.decorator_on_changed(tree, line_decorator_type().length(), Self::invalidate_measure);
        view.decorator_on_changed(tree, line_decorator_type().near(), Self::invalidate_near);
        view.decorator_on_changed(tree, line_decorator_type().stroke(), Self::invalidate_stroke);
        view.decorator_on_changed(tree, line_decorator_type().far(), Self::invalidate_far);
        view
    }

    fn invalidate_measure<T>(view: View, context: &mut dyn Context, _old: &T) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        view.invalidate_measure(tree);
    }

    fn invalidate_near(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let invalidated = Rect { tl: Point { x: 0, y: 0 }, size: Vector { x: 1, y: 1 } };
        view.invalidate_rect(tree, invalidated).unwrap();
    }

    fn invalidate_stroke(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        view.invalidate_render(tree).unwrap();
    }

    fn invalidate_far(view: View, context: &mut dyn Context, _old: &Option<Text>) {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let &orient = view.decorator_get(tree, line_decorator_type().orient());
        let size = view.render_bounds(tree).size;
        let invalidated = if orient == Orient::Vert {
            Rect { tl: Point { x: 0, y: size.y.overflowing_sub(1).0 }, size: Vector { x: 1, y: 1 } }
        } else {
            Rect { tl: Point { y: 0, x: size.x.overflowing_sub(1).0 }, size: Vector { x: 1, y: 1 } }
        };
        view.invalidate_rect(tree, invalidated).unwrap();
    }
}

impl DepObj for LineDecorator {
    type Type = LineDecoratorType;
    type Id = View;
    fn dep_props(&self) -> &DepObjProps<Self::Type, Self::Id> { &self.dep_props }
    fn dep_props_mut(&mut self) -> &mut DepObjProps<Self::Type, Self::Id> { &mut self.dep_props }
}

impl Decorator for LineDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior {
        static BEHAVIOR: LineDecoratorBehavior = LineDecoratorBehavior;
        &BEHAVIOR
    }
}

struct LineDecoratorBehavior;

impl DecoratorBehavior for LineDecoratorBehavior {
    fn children_measure_size(
        &self,
        _view: View,
        _tree: &mut ViewTree,
        _measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        (Some(0), Some(0))
    }

    fn desired_size(&self, view: View, tree: &mut ViewTree, _children_desired_size: Vector) -> Vector {
        let &orient = view.decorator_get(tree, line_decorator_type().orient());
        let &length = view.decorator_get(tree, line_decorator_type().length());
        if orient == Orient::Vert {
            Vector {  x: 1, y: length }
        } else {
            Vector {  x: length, y: 1 }
        }
    }

    fn children_arrange_bounds(&self, _view: View, _tree: &mut ViewTree, _arrange_size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0}, size: Vector::null() }
    }

    fn render_bounds(&self, view: View, tree: &mut ViewTree, _children_render_bounds: Rect) -> Rect {
        let &orient = view.decorator_get(tree, line_decorator_type().orient());
        let &length = view.decorator_get(tree, line_decorator_type().length());
        if orient == Orient::Vert {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: 1, y: length } }
        } else {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: length, y: 1 } }
        }
    }

    fn render(&self, view: View, tree: &ViewTree, port: &mut RenderPort) {
        let &orient = view.decorator_get(tree, line_decorator_type().orient());
        let &length = view.decorator_get(tree, line_decorator_type().length());
        let near = view.decorator_get(tree, line_decorator_type().near());
        let stroke = view.decorator_get(tree, line_decorator_type().stroke());
        let far = view.decorator_get(tree, line_decorator_type().far());
        if let Some(stroke) = stroke {
            for i in 0 .. length as u16 {
                if orient == Orient::Vert {
                    port.out(Point { x: 0, y: i as i16 }, stroke.fg, stroke.bg, stroke.attr, &stroke.value);
                } else {
                    port.out(Point { y: 0, x: i as i16 }, stroke.fg, stroke.bg, stroke.attr, &stroke.value);
                }
            }
        }
        if let Some(near) = near.as_ref() {
            port.out(Point { x: 0, y: 0 }, near.fg, near.bg, near.attr, &near.value);
        }
        if let Some(far) = far.as_ref() {
            if orient == Orient::Vert {
                port.out(Point { y: length.overflowing_sub(1).0, x: 0 }, far.fg, far.bg, far.attr, &far.value);
            } else {
                port.out(Point { x: length.overflowing_sub(1).0, y: 0 }, far.fg, far.bg, far.attr, &far.value);
            }
        }
    }
}
