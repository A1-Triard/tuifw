use crate::base::*;
use crate::view::View;
use crate::view::panels::{CanvasLayout, CanvasPanel};
use crate::window::{Window, WindowBuilder};
use dep_obj::{DepObjBaseBuilder, dep_type_with_builder, ItemChange, Change};
use dep_obj::binding::{Binding1, BindingExt2, b_continue, b_yield, b_immediate};
use dyn_context::state::{State, StateExt};

dep_type_with_builder! {
    #[derive(Debug)]
    pub struct DeskTop become obj in Widget {
        windows [Widget],
    }

    type BaseBuilder<'a> = WidgetBuilder<'a>;
}

impl<'a> DeskTopBuilder<'a> {
    pub fn window(
        mut self,
        storage: Option<&mut Option<Widget>>,
        f: impl for<'b> FnOnce(WindowBuilder<'b>) -> WindowBuilder<'b>
    ) -> Self {
        let desk_top = self.base_priv_ref().id();
        let window = Window::build(self.base_priv_mut().state_mut(), f);
        storage.map(|x| x.replace(window));
        b_immediate(DeskTop::WINDOWS.push(self.base_priv_mut().state_mut(), desk_top.obj(), window));
        self
    }
}

struct DeskTopBehavior;

impl WidgetBehavior for DeskTopBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State) {
        let init_new_view = Binding1::new(state, (), |(), change: Option<Change<Option<View>>>|
            change.and_then(|change| change.new)
        );
        init_new_view.set_target_fn(state, (), |state, (), view: View| CanvasPanel::new(state, view));
        widget.obj::<DeskTop>().add_binding(state, init_new_view);
        init_new_view.set_source_1(state, &mut WidgetBase::VIEW.change_initial_source(widget.base()));

        let windows = BindingExt2::new(state, (), |
            state,
            _,
            view: Option<View>,
            window: Option<ItemChange<Widget>>
        | {
            if let Some(window) = window {
                if window.is_remove() || window.is_update_remove() && view.is_none() {
                    window.item.unload(state)
                } else if let Some(view) = view {
                    if let Some(prev) = window.as_insert_or_update_insert_prev() {
                        let prev_view = prev.map(|prev| {
                            let tree: &WidgetTree = state.get();
                            prev.view(tree).unwrap()
                        });
                        window.item.load(state, view, prev_view, |state, view| CanvasLayout::new(state, view))
                    } else if let Some(prev) = window.as_move_insert_prev() {
                        let tree: &WidgetTree = state.get();
                        let view = window.item.view(tree).unwrap();
                        let prev_view = prev.map(|prev| prev.view(tree).unwrap());
                        view.move_z(state, prev_view);
                        b_continue()
                    } else {
                        b_continue()
                    }
                } else {
                    b_continue()
                }
            } else {
                b_yield(())
            }
        });
        windows.set_source_2(state, &mut DeskTop::WINDOWS.item_initial_final_source_with_update(windows, widget.obj()));
        windows.set_source_1(state, &mut WidgetBase::VIEW.value_source(widget.base()));
    }

    fn drop_bindings(&self, _widget: Widget, _state: &mut dyn State) { }
}

impl DeskTop {
    const BEHAVIOR: DeskTopBehavior = DeskTopBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, DeskTop::new_priv())
    }

    pub fn build<'a>(
        state: &'a mut dyn State,
        f: impl FnOnce(DeskTopBuilder<'a>) -> DeskTopBuilder<'a>
    ) -> Widget {
        let desk_top = DeskTop::new(state);
        f(DeskTopBuilder::new_priv(WidgetBuilder { widget: desk_top, state }));
        desk_top
    }
}

impl WidgetObj for DeskTop {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
