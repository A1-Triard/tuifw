#![deny(warnings)]
#![allow(dead_code)]

#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]

mod circuit {
    use components_arena::{Component, NewtypeComponentId, Id, Arena};
    use dep_obj::{dep_obj, DepType};
    use downcast_rs::{Downcast, impl_downcast};
    use dyn_context::state::{State, SelfState, StateExt};
    use macro_attr_2018::macro_attr;
    use std::fmt::Debug;

    pub trait ChipLegs: Downcast + DepType<Id=Chip> { }

    impl_downcast!(ChipLegs);

    macro_attr! {
        #[derive(Debug, Component!)]
        struct ChipNode {
            legs: Box<dyn ChipLegs>,
        }
    }

    macro_attr! {
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, NewtypeComponentId!)]
        pub struct Chip(Id<ChipNode>);
    }

    impl Chip {
        pub fn new<T>(
            state: &mut dyn State,
            legs: impl FnOnce(Chip) -> (Box<dyn ChipLegs>, T)
        ) -> T {
            let circuit: &mut Circuit = state.get_mut();
            circuit.arena.insert(|chip| {
                let (legs,  result) = legs(Chip(chip));
                (ChipNode { legs }, result)
            })
        }

        pub fn drop_chip(self, state: &mut dyn State) {
            self.drop_bindings_priv(state);
            let circuit: &mut Circuit = state.get_mut();
            circuit.arena.remove(self.0);
        }

        dep_obj! {
            pub fn legs(self as this, circuit: Circuit) -> (trait ChipLegs) {
                if mut {
                    circuit.arena[this.0].legs.as_mut()
                } else {
                    circuit.arena[this.0].legs.as_ref()
                }
            }
        }
    }

    #[derive(Debug)]
    pub struct Circuit {
        arena: Arena<ChipNode>,
    }

    impl SelfState for Circuit { }

    impl Circuit {
        pub fn new() -> Self { Circuit { arena: Arena::new() } }
    }
}

mod or_chip {
    use crate::circuit::*;
    use dep_obj::{dep_type};
    use dep_obj::binding::Binding2;
    use dyn_context::state::State;

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
            let chip = Chip::new(state, |chip| (Box::new(Self::new_priv(chip)) as _, chip));
            let binding = Binding2::new(state, (), |(), (_, in_1), (_, in_2)| Some(in_1 | in_2));
            binding.set_source_1(state, &mut OrLegs::IN_1.source(chip.legs()));
            binding.set_source_2(state, &mut OrLegs::IN_2.source(chip.legs()));
            OrLegs::OUT.bind_distinct(state, chip.legs(), binding);
            chip
        }
    }

    impl ChipLegs for OrLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use dep_obj::dep_type;
    use dep_obj::binding::Binding1;
    use dyn_context::state::State;

    dep_type! {
        #[derive(Debug)]
        pub struct NotLegs in Chip {
            in_: bool = false,
            out: bool = true,
        }
    }

    impl NotLegs {
        pub fn new(state: &mut dyn State) -> Chip {
            let chip = Chip::new(state, |chip| (Box::new(Self::new_priv(chip)) as _, chip));
            let binding = Binding1::new(state, (), |(), (_, in_1): (bool, bool)| Some(!in_1));
            binding.set_source_1(state, &mut NotLegs::IN_.source(chip.legs()));
            NotLegs::OUT.bind_distinct(state, chip.legs(), binding);
            chip
        }
    }

    impl ChipLegs for NotLegs { }
}

use circuit::*;
use dep_obj::binding::{Binding1, Bindings};
use dyn_context::state::{State, StateRefMut};
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
    bindings: Bindings,
    circuit: Circuit,
    chips: TriggerChips,
}

impl State for TriggerState {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<Bindings>() {
            Some(&self.bindings)
        } else if ty == TypeId::of::<Circuit>() {
            Some(&self.circuit)
        } else if ty == TypeId::of::<TriggerChips>() {
            Some(&self.chips)
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<Bindings>() {
            Some(&mut self.bindings)
        } else if ty == TypeId::of::<Circuit>() {
            Some(&mut self.circuit)
        } else {
            None
        }
    }
}

fn main() {
    let mut circuit = Circuit::new();
    let mut bindings = Bindings::new();
    let chips = (&mut circuit).merge_mut_and_then(|state| {
        let not_1 = NotLegs::new(state);
        let not_2 = NotLegs::new(state);
        let or_1 = OrLegs::new(state);
        let or_2 = OrLegs::new(state);
        TriggerChips { or_1, or_2, not_1, not_2 }
    }, &mut bindings);
    let state = &mut TriggerState {
        circuit,
        bindings,
        chips: chips.clone(),
    };

    let not_1_out_to_or_2_in = Binding1::new(state, (), |(), (_, value)| Some(value));
    not_1_out_to_or_2_in.set_source_1(state, &mut NotLegs::OUT.source(chips.not_1.legs()));
    OrLegs::IN_2.bind_uncond(state, chips.or_2.legs(), not_1_out_to_or_2_in);
    let not_2_out_to_or_1_in = Binding1::new(state, (), |(), (_, value)| Some(value));
    not_2_out_to_or_1_in.set_source_1(state, &mut NotLegs::OUT.source(chips.not_2.legs()));
    OrLegs::IN_2.bind_uncond(state, chips.or_1.legs(), not_2_out_to_or_1_in);
    let or_1_out_to_not_1_in = Binding1::new(state, (), |(), (_, value)| Some(value));
    or_1_out_to_not_1_in.set_source_1(state, &mut OrLegs::OUT.source(chips.or_1.legs()));
    NotLegs::IN_.bind_uncond(state, chips.not_1.legs(), or_1_out_to_not_1_in);
    let or_2_out_to_not_2_in = Binding1::new(state, (), |(), (_, value)| Some(value));
    or_2_out_to_not_2_in.set_source_1(state, &mut OrLegs::OUT.source(chips.or_2.legs()));
    NotLegs::IN_.bind_uncond(state, chips.not_2.legs(), or_2_out_to_not_2_in);

    let print_out = Binding1::new(state, (), |(), x| Some(x));
    print_out.set_source_1(state, &mut NotLegs::OUT.source(chips.not_2.legs()));
    print_out.set_target_fn(state, (), |_, (), (old, new)| {
        let old = if old { "1" } else { "0" };
        let new = if new { "1" } else { "0" };
        println!("{} -> {}", old, new);
    });
    OrLegs::IN_1.set_distinct(state, chips.or_1.legs(), true);
    OrLegs::IN_1.set_distinct(state, chips.or_1.legs(), false);
    OrLegs::IN_1.set_distinct(state, chips.or_2.legs(), true);
    OrLegs::IN_1.set_distinct(state, chips.or_2.legs(), false);
    OrLegs::IN_1.set_distinct(state, chips.or_1.legs(), true);
    OrLegs::IN_1.set_distinct(state, chips.or_1.legs(), false);
    OrLegs::IN_1.set_distinct(state, chips.or_2.legs(), true);
    OrLegs::IN_1.set_distinct(state, chips.or_2.legs(), false);

    print_out.drop_binding(state);
    chips.or_1.drop_chip(state);
    chips.or_2.drop_chip(state);
    chips.not_2.drop_chip(state);
    chips.not_1.drop_chip(state);
}
