use crate::base::*;
use crate::view::View;
use crate::view::panels::{CanvasLayout, CanvasPanel};
use dep_obj::{dep_type, ItemChange, Change, Glob};
use dep_obj::binding::{BindingExt2, b_yield, b_continue, AnyBindingBase};
use dyn_context::state::State;

dep_type! {
    #[derive(Debug)]
    pub struct DeskTop in Widget {
        windows [Widget],
    }
}

struct DeskTopBehavior;

impl WidgetBehavior for DeskTopBehavior {
    fn init_bindings(&self, widget: Widget, state: &mut dyn State) {
        let windows = BindingExt2::new(state, None, |
            state,
            view_cache: Glob<AnyBindingBase, Option<View>>,
            view_change: Option<Change<Option<View>>>,
            window: Option<ItemChange<Widget>>
        | {
            if let Some(view_change) = view_change {
                *view_cache.get_mut(state) = view_change.new;
                view_change.new.map(|view| CanvasPanel::new(state, view));
                b_yield(())
            } else if let Some(window) = window {
                let view = *view_cache.get(state);
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
                unreachable!()
            }
        });
        windows.set_source_2(state, &mut DeskTop::WINDOWS.item_initial_final_source_with_update(windows, widget.obj()));
        windows.set_source_1(state, &mut WidgetBase::VIEW.change_initial_source(widget.base()));
    }

    fn drop_bindings(&self, _widget: Widget, _state: &mut dyn State) { }
}

impl DeskTop {
    const BEHAVIOR: DeskTopBehavior = DeskTopBehavior;

    pub fn new(state: &mut dyn State) -> Widget {
        Widget::new(state, DeskTop::new_priv())
    }
}

impl WidgetObj for DeskTop {
    fn behavior(&self) -> &'static dyn WidgetBehavior { &Self::BEHAVIOR }
}
