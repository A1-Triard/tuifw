use crate::base::*;
use crate::view::View;
use crate::view::panels::{CanvasLayout, CanvasPanel};
use crate::window::Window;
use dep_obj::{Builder, Change, DepObjBuilder, DepObjId, ItemChange};
use dep_obj::{dep_type, ext_builder};
use dep_obj::binding::{Binding1, BindingExt2, Re};
use dyn_context::{State, StateExt};

dep_type! {
    #[derive(Debug)]
    pub struct DeskTop = Widget[WidgetObjKey] {
        windows [Widget],
    }
}

ext_builder!(<'a> Builder<'a, Widget> as BuilderWidgetDeskTopExt[Widget] {
    desk_top -> (DeskTop)
});

impl<T: DepObjBuilder<Id=Widget>> DeskTopBuilder<T> {
    pub fn window(
        mut self,
        storage: Option<&mut Option<Widget>>,
        build: impl for<'a> FnOnce(Builder<'a, Widget>) -> Builder<'a, Widget>
    ) -> Self {
        let desk_top = self.id();
        let window = Window::new(self.state_mut());
        window.build(self.state_mut(), build);
        storage.map(|x| x.replace(window));
        DeskTop::WINDOWS.push(self.state_mut(), desk_top, window).immediate();
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
        widget.add_binding::<DeskTop, _>(state, init_new_view);
        init_new_view.set_source_1(state, &mut WidgetBase::VIEW.change_initial_source(widget));

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
                        Re::Continue
                    } else {
                        Re::Continue
                    }
                } else {
                    Re::Continue
                }
            } else {
                Re::Yield(())
            }
        });
        windows.set_source_2(state, &mut DeskTop::WINDOWS.item_initial_final_source_with_update(windows, widget));
        windows.set_source_1(state, &mut WidgetBase::VIEW.value_source(widget));
    }

    fn drop_bindings(&self, _widget: Widget, _state: &mut dyn State) { }
}

impl DeskTop {
    const BEHAVIOR: DeskTopBehavior = DeskTopBehavior;

    #[allow(clippy::new_ret_no_self)]
    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, DeskTop::new_priv())
    }
}

impl WidgetObj for DeskTop {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
