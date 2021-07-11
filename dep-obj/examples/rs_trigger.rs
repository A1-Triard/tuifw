#![deny(warnings)]
#![allow(dead_code)]

#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]

mod circuit {
    use components_arena::{Component, ComponentId, Id, Arena, ComponentClassToken};
    use dep_obj::dep_obj;
    use downcast_rs::{Downcast, impl_downcast};
    use dyn_context::State;
    use educe::Educe;
    use macro_attr_2018::macro_attr;
    use std::fmt::Debug;

    pub trait ChipLegs: Downcast + Debug + Send + Sync { }

    impl_downcast!(ChipLegs);

    macro_attr! {
        #[derive(Component!)]
        #[derive(Debug)]
        struct ChipNode {
            chip: Chip,
            legs: Box<dyn ChipLegs>,
        }
    }

    macro_attr! {
        #[derive(ComponentId!)]
        #[derive(Educe)]
        #[educe(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub struct Chip(Id<ChipNode>);
    }

    impl Chip {
        pub fn new<T>(
            circuit: &mut Circuit,
            legs: impl FnOnce(Chip) -> (Box<dyn ChipLegs>, T)
        ) -> T {
            circuit.arena.insert(|chip| {
                let (legs,  result) = legs(Chip(chip));
                (ChipNode { chip: Chip(chip), legs }, result)
            })
        }

        pub fn drop(self, circuit: &mut Circuit) {
            circuit.arena.remove(self.0);
        }

        dep_obj! {
            pub fn legs(self as this, circuit: Circuit) -> dyn ChipLegs {
                if mut { &mut circuit.arena[this.0].legs } else { &circuit.arena[this.0].legs }
            }
        }
    }

    macro_attr! {
        #[derive(Debug, State!)]
        pub struct Circuit {
            arena: Arena<ChipNode>,
        }
    }

    impl Circuit {
        pub fn new(token: &mut CircuitToken) -> Self {
            Circuit {
                arena: Arena::new(&mut token.0)
            }
        }
    }

    pub struct CircuitToken(ComponentClassToken<ChipNode>);

    impl CircuitToken {
        pub fn new() -> Option<Self> {
            ComponentClassToken::new().map(CircuitToken)
        }
    }
}

mod or_chip {
    use crate::circuit::*;
    use components_arena::{ComponentId, RawId};
    use dep_obj::{dep_type};
    use dep_obj::flow::{Just};
    use dyn_context::{State, StateExt};

    dep_type! {
        #[derive(Debug)]
        pub struct OrLegs in Chip {
            in_1: bool = false,
            in_2: bool = false,
            out: bool = false,
        }
    }

    impl OrLegs {
        pub fn new(state: &mut dyn State) -> Chip {
            let legs = Self::new_priv();
            let circuit: &mut Circuit = state.get_mut();
            let chip = Chip::new(circuit, |chip| (Box::new(legs) as _, chip));
            let in_1 = chip.legs(state).prop(OrLegs::IN_1).values();
            let in_2 = chip.legs(state).prop(OrLegs::IN_2).values();
            in_1.zip(in_2, state).handle(state, chip, Self::update);
            chip
        }

        fn update(state: &mut dyn State, chip: RawId, Just((in_1, in_2)): Just<(bool, bool)>) {
            let chip = Chip::from_raw(chip);
            chip.legs(state).prop(OrLegs::OUT).set_distinct(in_1 | in_2);
        }
    }

    impl ChipLegs for OrLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use components_arena::{ComponentId, RawId};
    use dep_obj::dep_type;
    use dep_obj::flow::Just;
    use dyn_context::{State, StateExt};

    dep_type! {
        #[derive(Debug)]
        pub struct NotLegs in Chip {
            in_: bool = false,
            out: bool = true,
        }
    }

    impl NotLegs {
        pub fn new(state: &mut dyn State) -> Chip {
            let circuit: &mut Circuit = state.get_mut();
            let legs = Self::new_priv();
            let chip = Chip::new(circuit, |chip| (Box::new(legs) as _, chip));
            chip.legs(state).prop(NotLegs::IN_).values().handle(state, chip, Self::update);
            chip
        }

        fn update(state: &mut dyn State, chip: RawId, Just(in_): Just<bool>) {
            let chip = Chip::from_raw(chip);
            chip.legs(state).prop(NotLegs::OUT).set_distinct(!in_);
        }
    }

    impl ChipLegs for NotLegs { }
}

use circuit::*;
use components_arena::{ComponentId, RawId};
use dep_obj::flow::{Flows, FlowsToken, Just};
use dyn_context::{State, StateRefMut};
use not_chip::*;
use or_chip::*;
use std::any::{Any, TypeId};

#[derive(Debug, Clone)]
struct TriggerChips {
    pub or_1: Chip,
    pub or_2: Chip,
    pub not_1: Chip,
    pub not_2: Chip,
}

#[derive(Debug)]
struct TriggerState {
    flows: Flows,
    circuit: Circuit,
    chips: TriggerChips,
}

impl State for TriggerState {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<Flows>() {
            Some(&self.flows)
        } else if ty == TypeId::of::<Circuit>() {
            Some(&self.circuit)
        } else if ty == TypeId::of::<TriggerChips>() {
            Some(&self.chips)
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<Flows>() {
            Some(&mut self.flows)
        } else if ty == TypeId::of::<Circuit>() {
            Some(&mut self.circuit)
        } else {
            None
        }
    }
}

fn main() {
    let mut circuit_token = CircuitToken::new().unwrap();
    let mut circuit = Circuit::new(&mut circuit_token);
    let mut flows_token = FlowsToken::new().unwrap();
    let mut flows = Flows::new(&mut flows_token);
    let chips = (&mut circuit).merge_mut_and_then(|state| {
        let not_1 = NotLegs::new(state);
        let not_2 = NotLegs::new(state);
        let or_1 = OrLegs::new(state);
        let or_2 = OrLegs::new(state);
        TriggerChips { or_1, or_2, not_1, not_2 }
    }, &mut flows);
    let state = &mut TriggerState {
        circuit,
        flows,
        chips: chips.clone(),
    };
    let not_out_to_or_in = |state: &mut dyn State, or: RawId, Just(out): Just<bool>| {
        let or = Chip::from_raw(or);
        or.legs(state).prop(OrLegs::IN_2).set_uncond(out);
    };
    let or_out_to_not_out = |state: &mut dyn State, not: RawId, Just(out): Just<bool>| {
        let not = Chip::from_raw(not);
        not.legs(state).prop(NotLegs::IN_).set_uncond(out);
    };
    chips.not_1.legs(state).prop(NotLegs::OUT).values().handle(state, chips.or_2, not_out_to_or_in);
    chips.not_2.legs(state).prop(NotLegs::OUT).values().handle(state, chips.or_1, not_out_to_or_in);
    chips.or_1.legs(state).prop(OrLegs::OUT).values().handle(state, chips.not_1, or_out_to_not_out);
    chips.or_2.legs(state).prop(OrLegs::OUT).values().handle(state, chips.not_2, or_out_to_not_out);
    chips.not_1.legs(state).prop(NotLegs::OUT).changes().handle(state, (), |_, _, Just((old, new))| {
        let old = if old { "1" } else { "0" };
        let new = if new { "1" } else { "0" };
        println!("{} -> {}", old, new);
    });
    chips.or_1.legs(state).prop(OrLegs::IN_1).set_distinct(true);
    chips.or_1.legs(state).prop(OrLegs::IN_1).set_distinct(false);
    chips.or_2.legs(state).prop(OrLegs::IN_1).set_distinct(true);
    chips.or_2.legs(state).prop(OrLegs::IN_1).set_distinct(false);
    chips.or_1.legs(state).prop(OrLegs::IN_1).set_distinct(true);
    chips.or_1.legs(state).prop(OrLegs::IN_1).set_distinct(false);
    chips.or_2.legs(state).prop(OrLegs::IN_1).set_distinct(true);
    chips.or_2.legs(state).prop(OrLegs::IN_1).set_distinct(false);
}
