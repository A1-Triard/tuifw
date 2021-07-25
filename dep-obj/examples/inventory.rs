#![deny(warnings)]
#![allow(dead_code)]

#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]

use components_arena::{Arena, Component, NewtypeComponentId, Id, ComponentClassMutex};
use dep_obj::{dep_type, dep_obj};
use macro_attr_2018::macro_attr;
use dep_obj::flow::{Flows, FlowsMutex, Just};
use dyn_context::{State, StateExt};
use std::any::{TypeId, Any};
use std::borrow::Cow;
use std::fmt::Write;

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
        name: Cow<'static, str> = Cow::Borrowed(""),
        equipped: bool = false,
    }
}

impl Item {
    fn new(state: &mut dyn State) -> Item {
        let game: &mut Game = state.get_mut();
        game.items.insert(|id| (ItemComponent { props: ItemProps::new_priv() }, Item(id)))
    }

    dep_obj! {
        fn props(self as this, game: Game) -> &mut ItemProps {
            &mut game.items[this.0].props
        }
    }
}

macro_attr! {
    #[derive(Debug, Component!)]
    struct NpcComponent {
        props: NpcProps,
    }
}

static NPC_COMPONENT: ComponentClassMutex<NpcComponent> = ComponentClassMutex::new();

macro_attr! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
    struct Npc(Id<NpcComponent>);
}

dep_type! {
    #[derive(Debug)]
    struct NpcProps in Npc {
        equipped_items [Item],
    }
}

impl Npc {
    fn new(state: &mut dyn State) -> Npc {
        let game: &mut Game = state.get_mut();
        let npc = game.npcs.insert(|id| (NpcComponent { props: NpcProps::new_priv() }, Npc(id)));
        npc.props(state).vec(NpcProps::EQUIPPED_ITEMS)
            .removed_inserted_items().handle(state, npc, |state, _npc, Just((removed, inserted))| {
            for item in removed {
                item.props(state).prop(ItemProps::EQUIPPED).set_distinct(false);
            }
            for item in inserted {
                item.props(state).prop(ItemProps::EQUIPPED).set_distinct(true);
            }
        });
        npc
    }

    dep_obj! {
        fn props(self as this, game: Game) -> &mut NpcProps {
            &mut game.npcs[this.0].props
        }
    }
}

#[derive(Debug)]
struct Game {
    items: Arena<ItemComponent>,
    npcs: Arena<NpcComponent>,
    flows: Flows,
    log: String,
}

impl State for Game {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<Game>() {
            Some(self)
        } else if ty == TypeId::of::<Flows>() {
            Some(&self.flows)
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<Game>() {
            Some(self)
        } else if ty == TypeId::of::<Flows>() {
            Some(&mut self.flows)
        } else {
            None
        }
    }
}

impl Game {
    fn new() -> Game {
        Game {
            items: Arena::new(&mut ITEM_COMPONENT.lock().unwrap()),
            npcs: Arena::new(&mut NPC_COMPONENT.lock().unwrap()),
            flows: Flows::new(&mut FLOWS.lock().unwrap()),
            log: String::new(),
        }
    }
}

fn main() {
    let game = &mut Game::new();
    let npc = Npc::new(game);
    let sword = Item::new(game);
    sword.props(game).prop(ItemProps::NAME).set_uncond(Cow::Borrowed("Sword"));
    let shield = Item::new(game);
    shield.props(game).prop(ItemProps::NAME).set_uncond(Cow::Borrowed("Shield"));
    for item in [sword, shield] {
        item.props(game).prop(ItemProps::EQUIPPED).changes()
            .zip(item.props(game).prop(ItemProps::NAME).values(), game)
            .handle(game, item, |state, _item, Just(((old_equipped, equipped), name))| {
                if old_equipped == equipped { return; }
                let game: &mut Game = state.get_mut();
                writeln!(&mut game.log, "{} {}.", name, if equipped { "equipped" } else { "unequipped" }).unwrap();
        });
    }
    npc.props(game).vec(NpcProps::EQUIPPED_ITEMS).push(sword);
    npc.props(game).vec(NpcProps::EQUIPPED_ITEMS).push(shield);
    npc.props(game).vec(NpcProps::EQUIPPED_ITEMS).remove(0);
    assert_eq!(game.log, "\
        Sword equipped.\n\
        Shield equipped.\n\
        Sword unequipped.\n\
    ");
}
