#![deny(warnings)]
#![allow(dead_code)]

#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]

use components_arena::{Arena, Component, NewtypeComponentId, Id, ComponentClassMutex};
use dep_obj::{dep_type, dep_obj};
use macro_attr_2018::macro_attr;
use dep_obj::flow::{Flows, FlowsMutex};

static FLOWS: FlowsMutex = FlowsMutex::new();

macro_attr! {
    #[derive(Debug, Component!)]
    struct ItemComponent {
        props: ItemProps,
    }
}

static ITEM_COMPONENT: ComponentClassMutex<ItemComponent> = ComponentClassMutex::new();

macro_attr! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
    struct Item(Id<ItemComponent>);
}

dep_type! {
    #[derive(Debug)]
    struct ItemProps in Item {
        equipped: bool = false,
    }
}

#[derive(Debug)]
struct Game {
    items: Arena<ItemComponent>,
    flows: Flows,
}

impl Game {
    fn new() -> Game {
        Game {
            items: Arena::new(&mut ITEM_COMPONENT.lock().unwrap()),
            flows: Flows::new(&mut FLOWS.lock().unwrap()),
        }
    }
}

impl Item {
    dep_obj! {
        fn props(self as this, game: Game) -> ItemProps {
            if mut { &mut game.items[this.0].props } else { &game.items[this.0].props }
        }
    }
}

fn main() {

}