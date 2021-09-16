use crate::base::*;
use crate::view::panels::{CanvasLayout};
use dep_obj::{dep_type, ItemChange};
use dep_obj::binding::{BindingExt2, b_yield, b_continue};
use dyn_context::state::State;
use crate::view::View;

dep_type! {
    #[derive(Debug)]
    pub struct DeskTop in Widget {
        windows [Widget],
    }
}

struct DeskTopBehavior;

impl WidgetBehavior for DeskTopBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State) {
        let windows = BindingExt2::new(state, (), |state, (), view: Option<View>, window: Option<ItemChange<Widget>>| {
            if let Some(window) = window {
                if window.is_remove() || window.is_update() && view.is_none() {
                    window.item.unload(state)
                } else if let Some(view) = view {
                    if window.is_insert_or_after_update() {
                        window.item.load(state, view, |state, view| CanvasLayout::new(state, view))
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
}

impl WidgetObj for DeskTop {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
