use std::fmt::Debug;
use tuifw_screen_base::{Vector, Point, Rect};
use tuifw_window::{RenderPort};
use dep_obj::dep::{DepObjProps, DepTypeBuilder, DepProp, DepTypeToken, DepObj};
use dep_obj::reactive::{Context, ContextExt, Reactive};
use once_cell::sync::{self};
use crate::view::base::*;

pub struct LineDecoratorType {
    token: DepTypeToken<LineDecorator>,
    orient: DepProp<LineDecorator, Reactive<View, Orient>>,
    length: DepProp<LineDecorator, Reactive<View, i16>>,
    near: DepProp<LineDecorator, Reactive<View, Option<Text>>>,
    stroke: DepProp<LineDecorator, Reactive<View, Option<Text>>>,
    far: DepProp<LineDecorator, Reactive<View, Option<Text>>>,
}

impl LineDecoratorType {
    pub fn token(&self) -> &DepTypeToken<LineDecorator> { &self.token }
    pub fn orient(&self) -> DepProp<LineDecorator, Reactive<View, Orient>> { self.orient }
    pub fn length(&self) -> DepProp<LineDecorator, Reactive<View, i16>> { self.length }
    pub fn near(&self) -> DepProp<LineDecorator, Reactive<View, Option<Text>>> { self.near }
    pub fn stroke(&self) -> DepProp<LineDecorator, Reactive<View, Option<Text>>> { self.stroke }
    pub fn far(&self) -> DepProp<LineDecorator, Reactive<View, Option<Text>>> { self.far }
}

pub static LINE_DECORATOR_TYPE: sync::Lazy<LineDecoratorType> = sync::Lazy::new(|| {
    let mut builder = DepTypeBuilder::new().expect("LineDecoratorType builder locked");
    let orient = builder.prop(|| Reactive::new(Orient::Hor));
    let length = builder.prop(|| Reactive::new(3));
    let near = builder.prop(|| Reactive::new(None));
    let stroke = builder.prop(|| Reactive::new(None));
    let far = builder.prop(|| Reactive::new(None));
    let token = builder.build();
    LineDecoratorType {
        token,
        orient, length, near, stroke, far,
    }
});

macro_attr! {
    #[derive(DepObjRaw!)]
    #[derive(Debug)]
    pub struct LineDecorator {
        dep_props: DepObjProps<Self>,
    }
}

impl LineDecorator {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        tree: &mut ViewTree,
        parent: View,
    ) -> View {
        let view = View::new(tree, parent, |view| {
            let decorator = LineDecorator {
                dep_props: DepObjProps::new(LINE_DECORATOR_TYPE.token())
            };
            (Some(Box::new(decorator) as _), None, view)
        });
        view.decorator_on_changed(tree, LINE_DECORATOR_TYPE.orient(), Self::invalidate_measure);
        view.decorator_on_changed(tree, LINE_DECORATOR_TYPE.length(), Self::invalidate_measure);
        view.decorator_on_changed(tree, LINE_DECORATOR_TYPE.near(), Self::invalidate_near);
        view.decorator_on_changed(tree, LINE_DECORATOR_TYPE.stroke(), Self::invalidate_stroke);
        view.decorator_on_changed(tree, LINE_DECORATOR_TYPE.far(), Self::invalidate_far);
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
        let &orient = view.decorator_get(tree, LINE_DECORATOR_TYPE.orient());
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
    fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
    fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
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
        let &orient = view.decorator_get(tree, LINE_DECORATOR_TYPE.orient());
        let &length = view.decorator_get(tree, LINE_DECORATOR_TYPE.length());
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
        let &orient = view.decorator_get(tree, LINE_DECORATOR_TYPE.orient());
        let &length = view.decorator_get(tree, LINE_DECORATOR_TYPE.length());
        if orient == Orient::Vert {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: 1, y: length } }
        } else {
            Rect { tl: Point { x: 0, y: 0 }, size: Vector {  x: length, y: 1 } }
        }
    }

    fn render(&self, view: View, tree: &ViewTree, port: &mut RenderPort) {
        let &orient = view.decorator_get(tree, LINE_DECORATOR_TYPE.orient());
        let &length = view.decorator_get(tree, LINE_DECORATOR_TYPE.length());
        let near = view.decorator_get(tree, LINE_DECORATOR_TYPE.near());
        let stroke = view.decorator_get(tree, LINE_DECORATOR_TYPE.stroke());
        let far = view.decorator_get(tree, LINE_DECORATOR_TYPE.far());
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
