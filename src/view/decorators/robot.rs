use crate::view::base::*;
use alloc::boxed::Box;
use core::cmp::max;
use dep_obj::{Builder, DepObjId, dep_type, ext_builder};
use dep_obj::binding::{Binding, Binding1};
use dyn_context::{State, StateExt};
use core::fmt::Debug;
use tuifw_screen_base::{Attr, Color, Point, Rect, Vector};
use tuifw_window::RenderPort;

ext_builder!(<'a> Builder<'a, View> as BuilderViewRobotDecoratorExt[View] {
    fn robot_decorator(state: &mut dyn State, view: View) -> (RobotDecorator) {
        RobotDecorator::new(state, view);
    }
});

dep_type! {
    #[derive(Debug)]
    pub struct RobotDecorator = View[DecoratorKey] {
        width: u8 = 1,
        height: u8 = 1,
        robot_x: u8 = 0,
        robot_y: u8 = 0,
    }
}

impl RobotDecorator {
    const BEHAVIOR: RobotDecoratorBehavior = RobotDecoratorBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        state: &mut dyn State,
        view: View,
    ) {
        view.set_decorator(state, RobotDecorator::new_priv());
    }
}

impl Decorator for RobotDecorator {
    fn behavior(&self) -> &'static dyn DecoratorBehavior { &Self::BEHAVIOR }
}

#[derive(Debug)]
struct RobotDecoratorBindings {
    bg: Binding<Option<Color>>,
    width: Binding<u8>,
    height: Binding<u8>,
}

impl DecoratorBindings for RobotDecoratorBindings { }

#[derive(Debug)]
struct RobotDecoratorBehavior;

impl DecoratorBehavior for RobotDecoratorBehavior {
    fn ty(&self) -> &'static str { "Robot" }

    fn children_measure_size(
        &self,
        _view: View,
        _state: &mut dyn State,
        measure_size: (Option<i16>, Option<i16>)
    ) -> (Option<i16>, Option<i16>) {
        measure_size
    }

    fn desired_size(&self, view: View, state: &mut dyn State, _children_desired_size: Vector) -> Vector {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<RobotDecoratorBindings>().unwrap();
        let width = bindings.width.get_value(state).unwrap_or(1);
        let height = bindings.height.get_value(state).unwrap_or(1);
        Vector { x: 4 * width as i16 + 1, y: 2 * height as i16 + 1 }
    }

    fn children_arrange_bounds(&self, _view: View, _state: &mut dyn State, arrange_size: Vector) -> Rect {
        Rect { tl: Point { x: 0, y: 0 }, size: arrange_size }
    }

    fn render_bounds(&self, view: View, state: &mut dyn State, _children_render_bounds: Rect) -> Rect {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<RobotDecoratorBindings>().unwrap();
        let width = bindings.width.get_value(state).unwrap_or(1);
        let height = bindings.height.get_value(state).unwrap_or(1);
        Rect {
            tl: Point { x: 0, y: 0 },
            size: Vector { x: 4 * width as i16 + 1, y: 2 * height as i16 + 1 }
        }
    }

    fn render(&self, view: View, state: &dyn State, port: &mut RenderPort) {
        let tree: &ViewTree = state.get();
        let bindings = view.decorator_bindings(tree).downcast_ref::<RobotDecoratorBindings>().unwrap();
        let width = bindings.width.get_value(state).unwrap_or(1);
        let height = bindings.height.get_value(state).unwrap_or(1);
        let bg = bindings.bg.get_value(state).unwrap_or_default();
        port.fill(|port, p| port.out(p, Color::White, bg, Attr::empty(), " "));
        port.out(Point { x: 0, y: 0 }, Color::Yellow, bg, Attr::INTENSITY, "╔");
        port.out(Point { x: 4 * width as i16, y: 0 }, Color::Yellow, bg, Attr::INTENSITY, "╗");
        port.out(Point { x: 0, y: 2 * height as i16 }, Color::Yellow, bg, Attr::INTENSITY, "╚");
        port.out(Point { x: 4 * width as i16, y: 2 * height as i16 }, Color::Yellow, bg, Attr::INTENSITY, "╝");
        for x in 1 .. 4 * width as i16 {
            port.out(Point { x, y: 0 }, Color::Yellow, bg, Attr::INTENSITY, "═");
            port.out(Point { x, y: 2 * height as i16 }, Color::Yellow, bg, Attr::INTENSITY, "═");
        }
        for y in 1 .. 2 * height as i16 {
            port.out(Point { x: 0, y }, Color::Yellow, bg, Attr::INTENSITY, "║");
            port.out(Point { x: 4 * width as i16, y }, Color::Yellow, bg, Attr::INTENSITY, "║");
        }
        for x in 0 .. width {
            for y in 0 .. height {
                port.out(
                    Point { x: 2 + 4 * x as i16, y: 1 + 2 * y as i16 },
                    Color::Yellow, bg, Attr::INTENSITY,
                    "·"
                );
            }
        }
    }

    fn init_bindings(&self, view: View, state: &mut dyn State) -> Box<dyn DecoratorBindings> {
        let width = Binding1::new(state, (), |(), w| Some(max(1, w)));
        view.add_binding::<RobotDecorator, _>(state, width);
        let height = Binding1::new(state, (), |(), h| Some(max(1, h)));
        view.add_binding::<RobotDecorator, _>(state, height);
        let bg = Binding1::new(state, (), |(), bg| Some(bg));
        view.add_binding::<RobotDecorator, _>(state, bg);
        width.set_target_fn(state, view, |state, view, _| view.invalidate_measure(state));
        height.set_target_fn(state, view, |state, view, _| view.invalidate_measure(state));
        bg.set_target_fn(state, view, |state, view, _| view.invalidate_render(state));
        width.set_source_1(state, &mut RobotDecorator::WIDTH.value_source(view));
        height.set_source_1(state, &mut RobotDecorator::HEIGHT.value_source(view));
        bg.set_source_1(state, &mut ViewBase::BG.value_source(view));
        Box::new(RobotDecoratorBindings {
            width: width.into(),
            height: height.into(),
            bg: bg.into(),
        })
    }

    fn drop_bindings(&self, _view: View, _state: &mut dyn State, _bindings: Box<dyn DecoratorBindings>) { }
}
