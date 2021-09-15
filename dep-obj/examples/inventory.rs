#![deny(warnings)]
#![allow(dead_code)]

#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]

use components_arena::{Arena, Component, NewtypeComponentId, Id};
use dep_obj::{Change, DepObjBaseBuilder, DepObjId, dep_obj, dep_type, dep_type_with_builder, InsertRemove};
use macro_attr_2018::macro_attr;
use dep_obj::binding::{Binding1, Binding2, BindingExt2, Bindings};
use dyn_context::state::{State, StateExt};
use std::any::{TypeId, Any};
use std::borrow::Cow;
use std::fmt::Write;

macro_attr! {
    #[derive(Debug, Component!)]
    struct ItemData {
        props: ItemProps,
    }
}

macro_attr! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
    struct Item(Id<ItemData>);
}

impl DepObjId for Item { }

dep_type_with_builder! {
    #[derive(Debug)]
    struct ItemProps become props in Item {
        name: Cow<'static, str> = Cow::Borrowed(""),
        equipped: bool = false,
        enhancement: i8 = 0,
    }

    type BaseBuilder<'a> = ItemBuilder<'a>;
}

struct ItemBuilder<'a> {
    item: Item,
    state: &'a mut dyn State,
}

impl<'a> DepObjBaseBuilder<Item> for ItemBuilder<'a> {
    fn id(&self) -> Item { self.item }
    fn state(&self) -> &dyn State { self.state }
    fn state_mut(&mut self) -> &mut dyn State { self.state }
}

impl<'a> ItemBuilder<'a> {
    fn props(
        self,
        f: impl for<'b> FnOnce(ItemPropsBuilder<'b>) -> ItemPropsBuilder<'b>
    ) -> Self {
        f(ItemPropsBuilder::new_priv(self)).base_priv()
    }
}

impl Item {
    fn new(state: &mut dyn State) -> Item {
        let game: &mut Game = state.get_mut();
        game.items.insert(|id| (ItemData { props: ItemProps::new_priv() }, Item(id)))
    }

    fn drop_item(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let game: &mut Game = state.get_mut();
        game.items.remove(self.0);
    }

    fn build<'a>(
        self,
        state: &'a mut dyn State,
        f: impl FnOnce(ItemBuilder<'a>) -> ItemBuilder<'a>
    ) {
        f(ItemBuilder { item: self, state });
    }

    dep_obj! {
        fn props(self as this, game: Game) -> (ItemProps) {
            if mut {
                &mut game.items[this.0].props
            } else {
                &game.items[this.0].props
            }
        }
    }
}

macro_attr! {
    #[derive(Debug, Component!)]
    struct NpcComponent {
        props: NpcProps,
    }
}

macro_attr! {
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, NewtypeComponentId!)]
    struct Npc(Id<NpcComponent>);
}

impl DepObjId for Npc { }

dep_type! {
    #[derive(Debug)]
    struct NpcProps in Npc {
        equipped_items [Item],
        items_enhancement: i8 = 0,
    }
}

impl Npc {
    fn new(state: &mut dyn State) -> Npc {
        let game: &mut Game = state.get_mut();
        let npc = game.npcs.insert(|id| (NpcComponent { props: NpcProps::new_priv() }, Npc(id)));

        let equipped = Binding1::new(state, (), |(), item: Option<InsertRemove<Item>>| item);
        equipped.set_target_fn(state, (), |state, (), item|
            ItemProps::EQUIPPED.set(state, item.item.props(), !item.remove)
        );
        equipped.set_source_1(state, &mut NpcProps::EQUIPPED_ITEMS.item_source(npc.props()));
        npc.props().add_binding(state, equipped);

        let enhancement = BindingExt2::new(state, (), |state, (), enhancement, item: Option<InsertRemove<Item>>| {
            if let Some(item) = item {
                if item.remove {
                    ItemProps::ENHANCEMENT.unset(state, item.item.props());
                } else {
                    ItemProps::ENHANCEMENT.set(state, item.item.props(), enhancement);
                }
                None
            } else {
                Some(())
            }
        });
        enhancement.set_source_2(state, &mut NpcProps::EQUIPPED_ITEMS.item_source_with_refresh(enhancement, npc.props()));
        enhancement.set_source_1(state, &mut NpcProps::ITEMS_ENHANCEMENT.value_source(npc.props()));

        npc
    }

    fn drop_npc(self, state: &mut dyn State) {
        self.drop_bindings_priv(state);
        let game: &mut Game = state.get_mut();
        game.npcs.remove(self.0);
    }

    dep_obj! {
        fn props(self as this, game: Game) -> (NpcProps) {
            if mut {
                &mut game.npcs[this.0].props
            } else {
                &game.npcs[this.0].props
            }
        }
    }
}

#[derive(Debug)]
struct Game {
    items: Arena<ItemData>,
    npcs: Arena<NpcComponent>,
    bindings: Bindings,
    log: String,
}

impl State for Game {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<Game>() {
            Some(self)
        } else if ty == TypeId::of::<Bindings>() {
            Some(&self.bindings)
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<Game>() {
            Some(self)
        } else if ty == TypeId::of::<Bindings>() {
            Some(&mut self.bindings)
        } else {
            None
        }
    }
}

impl Game {
    fn new() -> Game {
        Game {
            items: Arena::new(),
            npcs: Arena::new(),
            bindings: Bindings::new(),
            log: String::new(),
        }
    }
}

fn main() {
    let game = &mut Game::new();
    let npc = Npc::new(game);
    NpcProps::ITEMS_ENHANCEMENT.set(game, npc.props(), 5);
    let sword = Item::new(game);
    sword.build(game, |sword| sword
        .props(|props| props
            .name(Cow::Borrowed("Sword"))
        )
    );
    let shield = Item::new(game);
    ItemProps::NAME.set(game, shield.props(), Cow::Borrowed("Shield"));
    for item in [sword, shield] {
        let equipped = Binding2::new(game, (), |(), name, equipped: Option<Change<bool>>|
            equipped.map(|equipped| (name, equipped.new))
        );
        equipped.set_target_fn(game, (), |game, (), (name, equipped)| {
            let game: &mut Game = game.get_mut();
            writeln!(&mut game.log, "{} {}.", name, if equipped { "equipped" } else { "unequipped" }).unwrap();
        });
        equipped.set_source_1(game, &mut ItemProps::NAME.value_source(item.props()));
        equipped.set_source_2(game, &mut ItemProps::EQUIPPED.change_source(item.props()));
        item.props().add_binding(game, equipped);
        let enhancement = Binding2::new(game, (), |(), name, enhancement: Option<Change<i8>>|
            enhancement.map(|enhancement| (name, enhancement))
        );
        enhancement.set_target_fn(game, (), |game, (), (name, enhancement)| {
            let game: &mut Game = game.get_mut();
            writeln!(&mut game.log, "{} enhancement changed: {} -> {}.", name, enhancement.old, enhancement.new).unwrap();
        });
        enhancement.set_source_1(game, &mut ItemProps::NAME.value_source(item.props()));
        enhancement.set_source_2(game, &mut ItemProps::ENHANCEMENT.change_source(item.props()));
        item.props().add_binding(game, enhancement);
    }
    NpcProps::EQUIPPED_ITEMS.push(game, npc.props(), sword);
    NpcProps::EQUIPPED_ITEMS.push(game, npc.props(), shield);
    NpcProps::ITEMS_ENHANCEMENT.set(game, npc.props(), 4);
    NpcProps::EQUIPPED_ITEMS.remove(game, npc.props(), 0);
    NpcProps::ITEMS_ENHANCEMENT.set(game, npc.props(), 5);
    assert_eq!(game.log, "\
        Sword equipped.\n\
        Sword enhancement changed: 0 -> 5.\n\
        Shield equipped.\n\
        Shield enhancement changed: 0 -> 5.\n\
        Sword enhancement changed: 5 -> 0.\n\
        Shield enhancement changed: 5 -> 0.\n\
        Sword enhancement changed: 0 -> 4.\n\
        Shield enhancement changed: 0 -> 4.\n\
        Sword unequipped.\n\
        Sword enhancement changed: 4 -> 0.\n\
        Shield enhancement changed: 4 -> 0.\n\
        Shield enhancement changed: 0 -> 5.\n\
    ");
    npc.drop_npc(game);
    sword.drop_item(game);
    shield.drop_item(game);
    print!("{}", game.log);
}
