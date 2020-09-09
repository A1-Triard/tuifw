#![deny(warnings)]
#![allow(dead_code)]

mod circuit {
    use dep_obj::dep_system;
    use components_arena::{RawId, Component, ComponentId, Id, Arena, ComponentClassToken};
    use std::fmt::Debug;
    use std::mem::replace;
    use educe::Educe;
    use macro_attr_2018::macro_attr;
    use downcast_rs::{Downcast, impl_downcast};

    pub trait ChipLegs: Downcast + Debug + Send + Sync { }

    impl_downcast!(ChipLegs);

    macro_attr! {
        #[derive(Component!)]
        #[derive(Debug)]
        struct ChipNode {
            chip: Chip,
            legs: Box<dyn ChipLegs>,
            tag: RawId,
        }
    }

    macro_attr! {
        #[derive(ComponentId!)]
        #[derive(Educe)]
        #[educe(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        pub struct Chip(Id<ChipNode>);
    }

    impl Chip {
        pub fn new<Tag: ComponentId, T>(
            circuit: &mut Circuit,
            legs_tag: impl FnOnce(Chip) -> (Box<dyn ChipLegs>, Tag, T)
        ) -> T {
            circuit.arena.insert(|chip| {
                let (legs, tag, result) = legs_tag(Chip(chip));
                (ChipNode { chip: Chip(chip), legs, tag: tag.into_raw() }, result)
            })
        }

        pub fn tag<Tag: ComponentId>(self, circuit: &Circuit) -> Tag {
            Tag::from_raw(circuit.arena[self.0].tag)
        }

        pub fn set_tag<Tag: ComponentId>(self, circuit: &mut Circuit, tag: Tag) -> Tag {
            Tag::from_raw(replace(&mut circuit.arena[self.0].tag, tag.into_raw()))
        }

        pub fn drop(self, circuit: &mut Circuit) {
            circuit.arena.remove(self.0);
        }

        dep_system! {
            pub dyn fn legs(self as this, circuit: Circuit) -> ChipLegs {
                if mut { &mut circuit.arena[this.0].legs } else { &circuit.arena[this.0].legs }
            }
        }
    }

    #[derive(Debug)]
    pub struct Circuit {
        arena: Arena<ChipNode>,
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
    use components_arena::ComponentId;
    use dep_obj::{dep_obj, DepTypeToken};
    use dyn_context::{Context, ContextExt};

    dep_obj! {
        #[derive(Debug)]
        pub struct OrLegs as Chip: OrLegsType {
            in_1: bool = false,
            in_2: bool = false,
            out: bool = false,
        }
    }

    impl OrLegsType {
        pub fn new() -> Option<DepTypeToken<Self>> { Self::new_raw() }
    }

    impl OrLegs {
        pub fn new<Tag: ComponentId, T>(
            circuit: &mut Circuit,
            token: &DepTypeToken<OrLegsType>,
            tag: impl FnOnce(Chip) -> (Tag, T)
        ) -> T {
            let legs = Self::new_raw(token);
            let (chip, result) = Chip::new(circuit, |chip| {
                let (tag, result) = tag(chip);
                (Box::new(legs) as _, tag, (chip, result))
            });
            chip.legs_on_changed(circuit, token.ty().in_1(), Self::update);
            chip.legs_on_changed(circuit, token.ty().in_2(), Self::update);
            result
        }

        fn update(chip: Chip, context: &mut dyn Context, _old: &bool) {
            let token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
            let circuit = context.get::<Circuit>().expect("Cicuit required");
            let in_1 = *chip.legs_get(circuit, token.ty().in_1());
            let in_2 = *chip.legs_get(circuit, token.ty().in_2());
            let out = token.ty().out();
            chip.legs_set_distinct(context, out, in_1 | in_2);
        }
    }

    impl ChipLegs for OrLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use components_arena::ComponentId;
    use dep_obj::{dep_obj, DepTypeToken};
    use dyn_context::{Context, ContextExt};

    dep_obj! {
        #[derive(Debug)]
        pub struct NotLegs as Chip: NotLegsType {
            in_: bool = false,
            out: bool = true,
        }
    }

    impl NotLegsType {
        pub fn new() -> Option<DepTypeToken<Self>> { Self::new_raw() }
    }

    impl NotLegs {
        pub fn new<Tag: ComponentId, T>(
            circuit: &mut Circuit,
            token: &DepTypeToken<NotLegsType>,
            tag: impl FnOnce(Chip) -> (Tag, T)
        ) -> T {
            let legs = Self::new_raw(token);
            let (chip, result) = Chip::new(circuit, |chip| {
                let (tag, result) = tag(chip);
                (Box::new(legs) as _, tag, (chip, result))
            });
            chip.legs_on_changed(circuit, token.ty().in_(), Self::update);
            result
        }

        fn update(chip: Chip, context: &mut dyn Context, _old: &bool) {
            let token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
            let circuit = context.get::<Circuit>().expect("Cicuit required");
            let in_ = *chip.legs_get(circuit, token.ty().in_());
            let out = token.ty().out();
            chip.legs_set_distinct(context, out, !in_);
        }
    }

    impl ChipLegs for NotLegs { }
}

use std::any::{Any, TypeId};
use std::num::NonZeroUsize;
use dep_obj::{DepTypeToken};
use dyn_context::{Context, ContextExt, context};
use circuit::*;
use or_chip::*;
use not_chip::*;

context! {
    mod trigger_context {
        circuit: mut Circuit,
        or_legs_token: ref DepTypeToken<OrLegsType>,
        not_legs_token: ref DepTypeToken<NotLegsType>,
    }
}

use trigger_context::Context as TriggerContext;

impl Context for TriggerContext {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if ty == TypeId::of::<Circuit>() {
            Some(self.circuit())
        } else if ty == TypeId::of::<DepTypeToken<OrLegsType>>() {
            Some(self.or_legs_token())
        } else if ty == TypeId::of::<DepTypeToken<NotLegsType>>() {
            Some(self.not_legs_token())
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if ty == TypeId::of::<Circuit>() {
            Some(self.circuit_mut())
        } else {
            None
        }
    }
}

fn main() {
    let mut circuit_token = CircuitToken::new().unwrap();
    let circuit = &mut Circuit::new(&mut circuit_token);
    let or_legs_token: DepTypeToken<OrLegsType> =  OrLegsType::new().unwrap();
    let not_legs_token: DepTypeToken<NotLegsType> = NotLegsType::new().unwrap();
    let not_1 = NotLegs::new(circuit, &not_legs_token, |chip| ((0, unsafe { NonZeroUsize::new_unchecked(1) }), chip));
    let not_2 = NotLegs::new(circuit, &not_legs_token, |chip| ((0, unsafe { NonZeroUsize::new_unchecked(1) }), chip));
    let or_1 = OrLegs::new(circuit, &or_legs_token, |chip| (not_1, chip));
    let or_2 = OrLegs::new(circuit, &or_legs_token, |chip| (not_2, chip));
    not_1.set_tag(circuit, or_2);
    not_2.set_tag(circuit, or_1);
    not_1.legs_on_changed(circuit, not_legs_token.ty().out(), |not_1, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let or_legs_token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
        let circuit = context.get::<Circuit>().expect("Cicuit required");
        let or_2: Chip = not_1.tag(circuit);
        let &out = not_1.legs_get(circuit, not_legs_token.ty().out());
        let in_2 = or_legs_token.ty().in_2();
        or_2.legs_set_uncond(context, in_2, out);
    });
    not_2.legs_on_changed(circuit, not_legs_token.ty().out(), |not_2, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let or_legs_token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
        let circuit = context.get::<Circuit>().expect("Cicuit required");
        let or_1: Chip = not_2.tag(circuit);
        let &out = not_2.legs_get(circuit, not_legs_token.ty().out());
        let in_2 = or_legs_token.ty().in_2();
        or_1.legs_set_uncond(context, in_2, out);
    });
    or_1.legs_on_changed(circuit, or_legs_token.ty().out(), |or_1, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let or_legs_token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
        let circuit = context.get::<Circuit>().expect("Cicuit required");
        let not_1: Chip = or_1.tag(circuit);
        let &out = or_1.legs_get(circuit, or_legs_token.ty().out());
        let in_ = not_legs_token.ty().in_();
        not_1.legs_set_uncond(context, in_, out);
    });
    or_2.legs_on_changed(circuit, or_legs_token.ty().out(), |or_2, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let or_legs_token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
        let circuit = context.get::<Circuit>().expect("Cicuit required");
        let not_2: Chip = or_2.tag(circuit);
        let &out = or_2.legs_get(circuit, or_legs_token.ty().out());
        let in_ = not_legs_token.ty().in_();
        not_2.legs_set_uncond(context, in_, out);
    });
    not_1.legs_on_changed(circuit, not_legs_token.ty().out(), |not_1, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let circuit = context.get::<Circuit>().expect("Cicuit required");
        let &out = not_1.legs_get(circuit, not_legs_token.ty().out());
        println!("{}", if out { "0 -> 1" } else { "1 -> 0" });
    });
    TriggerContext::call(circuit, &or_legs_token, &not_legs_token, |context| {
        or_1.legs_set_distinct(context, or_legs_token.ty().in_1(), true);
        or_1.legs_set_distinct(context, or_legs_token.ty().in_1(), false);
        or_2.legs_set_distinct(context, or_legs_token.ty().in_1(), true);
        or_2.legs_set_distinct(context, or_legs_token.ty().in_1(), false);
        or_1.legs_set_distinct(context, or_legs_token.ty().in_1(), true);
        or_1.legs_set_distinct(context, or_legs_token.ty().in_1(), false);
        or_2.legs_set_distinct(context, or_legs_token.ty().in_1(), true);
        or_2.legs_set_distinct(context, or_legs_token.ty().in_1(), false);
    });
}
