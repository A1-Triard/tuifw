#![feature(explicit_generic_args_with_impl_trait)]

#![deny(warnings)]

use dep_obj::DepObjId;
use dep_obj::binding::{Binding1, Bindings};
use dyn_context::{State, StateExt, StateRefMut, Stop};
use tuifw::{HAlign, VAlign, Key};
use tuifw::view::{BuilderViewAlignExt, View, ViewBase, ViewTree, ViewInput};
use tuifw::view::panels::{BuilderViewDockPanelExt};
use tuifw::view::decorators::{BuilderViewRobotDecoratorExt};

fn build(state: &mut dyn State) -> View {
    let tree: &ViewTree = state.get();
    let root = tree.root();
    let mut robot = None;
    root.build(state, |view| view
        .dock_panel(|panel| panel
            .child(Some(&mut robot), (), |layout| layout, |view| view
                .align(|align| align
                    .h_align(HAlign::Center)
                    .v_align(VAlign::Center)
                )
                .robot_decorator(|decorator| decorator
                    .width(5)
                    .height(5)
                )
            )
        )
    );
    robot.unwrap()
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let bindings = &mut Bindings::new();
    let tree = &mut ViewTree::new(screen, bindings);
    tree.merge_mut_and_then(|state| {
        let robot = build(state);
        let input_binding = Binding1::new(state, (), |(), input: Option<ViewInput>| input);
        input_binding.set_target_fn(state, (), |state, (), input| {
            if input.key().1 == Key::Escape {
                input.mark_as_handled();
                ViewTree::quit(state);
            }
        });
        robot.add_binding::<ViewBase, _>(state, input_binding);
        input_binding.set_source_1(state, &mut ViewBase::INPUT.source(robot));
        robot.focus(state);
        while ViewTree::update(state, true).unwrap() { }
        ViewTree::stop(state);
    }, bindings);
}
