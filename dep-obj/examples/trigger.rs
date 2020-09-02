#![deny(warnings)]
#![allow(dead_code)]

#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate components_arena;
#[macro_use]
extern crate dep_obj;
#[macro_use]
extern crate downcast;
#[macro_use]
extern crate derivative;

mod circuit {
    use dep_obj::{DepObj, DepProp};
    use dep_obj::{Context, ContextExt};
    use components_arena::{ComponentId, Id, Arena, ComponentClassToken};
    use downcast::Any;
    use std::fmt::Debug;
    use std::num::NonZeroUsize;

    pub trait ChipLegs: Any + Debug + Send + Sync { }

    downcast!(dyn ChipLegs);

    macro_attr! {
        #[derive(Component!(class=ChipNodeComponent))]
        #[derive(Debug)]
        struct ChipNode<Tag> {
            chip: Chip<Tag>,
            legs: Box<dyn ChipLegs>,
            tag: Tag,
        }
    }

    #[derive(Derivative)]
    #[derivative(Debug(bound=""), Copy(bound=""), Clone(bound=""), Eq(bound=""), PartialEq(bound=""))]
    #[derivative(Hash(bound=""), Ord(bound=""), PartialOrd(bound=""))]
    pub struct Chip<Tag>(Id<ChipNode<Tag>>);

    impl<Tag: 'static> Chip<Tag> {
        pub fn new<T>(
            circuit: &mut Circuit<Tag>,
            legs_tag: impl FnOnce(Chip<Tag>) -> (Box<dyn ChipLegs>, Tag, T)
        ) -> T {
            circuit.arena.insert(|chip| {
                let (legs, tag, result) = legs_tag(Chip(chip));
                (ChipNode { chip: Chip(chip), legs, tag }, result)
            })
        }

        pub fn tag(self, circuit: &Circuit<Tag>) -> &Tag {
            &circuit.arena[self.0].tag
        }

        pub fn tag_mut(self, circuit: &mut Circuit<Tag>) -> &mut Tag {
            &mut circuit.arena[self.0].tag
        }

        pub fn drop(self, circuit: &mut Circuit<Tag>) {
            circuit.arena.remove(self.0);
        }

        pub fn get<Legs: ChipLegs + DepObj<Id=Chip<Tag>>, T>(
            self,
            circuit: &Circuit<Tag>,
            prop: DepProp<Legs, T>,
        ) -> &T {
            let legs = circuit.arena[self.0].legs.downcast_ref::<Legs>().expect("invalid cast");
            prop.get(legs.dep_props())
        }

        pub fn set_uncond<Legs: ChipLegs + DepObj<Id=Chip<Tag>>, T>(
            self,
            context: &mut dyn Context,
            prop: DepProp<Legs, T>,
            value: T,
        ) -> T {
            let circuit = context.get_mut::<Circuit<Tag>>().expect("Circuit required");
            let legs = circuit.arena[self.0].legs.downcast_mut::<Legs>().expect("invalid cast");
            let (old, on_changed) = prop.set_uncond(legs.dep_props_mut(), value);
            on_changed.raise(self, context, &old);
            old
        }

        pub fn set_distinct<Legs: ChipLegs + DepObj<Id=Chip<Tag>>, T: Eq>(
            self,
            context: &mut dyn Context,
            prop: DepProp<Legs, T>,
            value: T,
        ) -> T {
            let circuit = context.get_mut::<Circuit<Tag>>().expect("Circuit required");
            let legs = circuit.arena[self.0].legs.downcast_mut::<Legs>().expect("invalid cast");
            let (old, on_changed) = prop.set_distinct(legs.dep_props_mut(), value);
            on_changed.raise(self, context, &old);
            old
        }

        pub fn on_changed<Legs: ChipLegs + DepObj<Id=Chip<Tag>>, T>(
            self,
            circuit: &mut Circuit<Tag>,
            prop: DepProp<Legs, T>,
            on_changed: fn(owner: Chip<Tag>, context: &mut dyn Context, old: &T),
        ) {
            let legs = circuit.arena[self.0].legs.downcast_mut::<Legs>().expect("invalid cast");
            prop.on_changed(legs.dep_props_mut(), on_changed);
        }
    }

    impl<Tag> ComponentId for Chip<Tag> {
        fn from_raw_parts(raw_parts: (usize, NonZeroUsize)) -> Self {
            Chip(Id::from_raw_parts(raw_parts))
        }

        fn into_raw_parts(self) -> (usize, NonZeroUsize) {
            self.0.into_raw_parts()
        }
    }

    #[derive(Debug)]
    pub struct Circuit<Tag> {
        arena: Arena<ChipNode<Tag>>,
    }

    impl<Tag> Circuit<Tag> {
        pub fn new(token: &mut CircuitToken) -> Self {
            Circuit {
                arena: Arena::new(&mut token.0)
            }
        }
    }

    pub struct CircuitToken(ComponentClassToken<ChipNodeComponent>);

    impl CircuitToken {
        pub fn new() -> Option<Self> {
            ComponentClassToken::new().map(CircuitToken)
        }
    }
}

mod or_chip {
    use crate::circuit::*;
    use dep_obj::{DepObj, DepObjProps, DepPropRaw, DepProp, DepTypeBuilder, DepTypeToken};
    use dep_obj::{Context, ContextExt};

    macro_attr! {
        #[derive(DepType!)]
        pub struct OrLegsType {
            in_1: DepPropRaw<OrLegsType, bool>,
            in_2: DepPropRaw<OrLegsType, bool>,
            out: DepPropRaw<OrLegsType, bool>,
        }
    }

    impl OrLegsType {
        pub fn in_1<Tag>(&self) -> DepProp<OrLegs<Tag>, bool> { self.in_1.owned_by() }
        pub fn in_2<Tag>(&self) -> DepProp<OrLegs<Tag>, bool> { self.in_2.owned_by() }
        pub fn out<Tag>(&self) -> DepProp<OrLegs<Tag>, bool> { self.out.owned_by() }

        pub fn new() -> Option<DepTypeToken<Self>> {
            DepTypeBuilder::new().map(|mut builder| {
                let in_1 = builder.prop(|| false);
                let in_2 = builder.prop(|| false);
                let out = builder.prop(|| false);
                builder.build(OrLegsType { in_1, in_2, out })
            })
        }
    }

    #[derive(Derivative)]
    #[derivative(Debug(bound=""))]
    pub struct OrLegs<Tag> {
        dep_props: DepObjProps<OrLegsType, Chip<Tag>>,
    }

    impl<Tag: Send + Sync + 'static> OrLegs<Tag> {
        pub fn new<T>(
            circuit: &mut Circuit<Tag>,
            token: &DepTypeToken<OrLegsType>,
            tag: impl FnOnce(Chip<Tag>) -> (Tag, T)
        ) -> T {
            let legs = Self {
                dep_props: DepObjProps::new(token)
            };
            let (chip, result) = Chip::new(circuit, |chip| {
                let (tag, result) = tag(chip);
                (Box::new(legs) as _, tag, (chip, result))
            });
            chip.on_changed(circuit, token.type_().in_1(), Self::update);
            chip.on_changed(circuit, token.type_().in_2(), Self::update);
            result
        }

        fn update(chip: Chip<Tag>, context: &mut dyn Context, _old: &bool) {
            let token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
            let circuit = context.get::<Circuit<Tag>>().expect("Cicuit required");
            let in_1 = *chip.get(circuit, token.type_().in_1());
            let in_2 = *chip.get(circuit, token.type_().in_2());
            let out = token.type_().out();
            chip.set_distinct(context, out, in_1 | in_2);
        }
    }

    impl<Tag> DepObj for OrLegs<Tag> {
        type Type = OrLegsType;
        type Id = Chip<Tag>;
        fn dep_props(&self) -> &DepObjProps<Self::Type, Self::Id> { &self.dep_props }
        fn dep_props_mut(&mut self) -> &mut DepObjProps<Self::Type, Self::Id> { &mut self.dep_props }
    }

    impl<Tag: Send + Sync + 'static> ChipLegs for OrLegs<Tag> { }
}

mod not_chip {
    use crate::circuit::*;
    use dep_obj::{DepObj, DepObjProps, DepPropRaw, DepProp, DepTypeBuilder, DepTypeToken};
    use dep_obj::{Context, ContextExt};

    macro_attr! {
        #[derive(DepType!)]
        pub struct NotLegsType {
            in_: DepPropRaw<NotLegsType, bool>,
            out: DepPropRaw<NotLegsType, bool>,
        }
    }

    impl NotLegsType {
        pub fn in_<Tag>(&self) -> DepProp<NotLegs<Tag>, bool> { self.in_.owned_by() }
        pub fn out<Tag>(&self) -> DepProp<NotLegs<Tag>, bool> { self.out.owned_by() }

        pub fn new() -> Option<DepTypeToken<Self>> {
            DepTypeBuilder::new().map(|mut builder| {
                let in_ = builder.prop(|| false);
                let out = builder.prop(|| true);
                builder.build(NotLegsType { in_, out })
            })
        }
    }

    #[derive(Derivative)]
    #[derivative(Debug(bound=""))]
    pub struct NotLegs<Tag> {
        dep_props: DepObjProps<NotLegsType, Chip<Tag>>,
    }

    impl<Tag: Send + Sync + 'static> NotLegs<Tag> {
        pub fn new<T>(
            circuit: &mut Circuit<Tag>,
            token: &DepTypeToken<NotLegsType>,
            tag: impl FnOnce(Chip<Tag>) -> (Tag, T)
        ) -> T {
            let legs = Self {
                dep_props: DepObjProps::new(token),
            };
            let (chip, result) = Chip::new(circuit, |chip| {
                let (tag, result) = tag(chip);
                (Box::new(legs) as _, tag, (chip, result))
            });
            chip.on_changed(circuit, token.type_().in_(), Self::update);
            result
        }

        fn update(chip: Chip<Tag>, context: &mut dyn Context, _old: &bool) {
            let token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
            let circuit = context.get::<Circuit<Tag>>().expect("Cicuit required");
            let in_ = *chip.get(circuit, token.type_().in_());
            let out = token.type_().out();
            chip.set_distinct(context, out, !in_);
        }
    }

    impl<Tag> DepObj for NotLegs<Tag> {
        type Type = NotLegsType;
        type Id = Chip<Tag>;
        fn dep_props(&self) -> &DepObjProps<Self::Type, Self::Id> { &self.dep_props }
        fn dep_props_mut(&mut self) -> &mut DepObjProps<Self::Type, Self::Id> { &mut self.dep_props }
    }

    impl<Tag: Send + Sync + 'static> ChipLegs for NotLegs<Tag> { }
}

use std::any::{Any, TypeId};
use std::num::NonZeroUsize;
use components_arena::{ComponentId};
use dep_obj::{Context, ContextExt, DepTypeToken};
use circuit::*;
use or_chip::*;
use not_chip::*;

context! {
    mod trigger_context {
        circuit (circuit_mut): mut Circuit<(usize, NonZeroUsize)>,
        or_legs_type_token: ref DepTypeToken<OrLegsType>,
        not_legs_type_token: ref DepTypeToken<NotLegsType>,
    }
}

use trigger_context::Context as TriggerContext;

impl Context for TriggerContext {
    fn get_raw(&self, type_: TypeId) -> Option<&dyn Any> {
        if type_ == TypeId::of::<Circuit<(usize, NonZeroUsize)>>() {
            Some(self.circuit())
        } else if type_ == TypeId::of::<DepTypeToken<OrLegsType>>() {
            Some(self.or_legs_type_token())
        } else if type_ == TypeId::of::<DepTypeToken<NotLegsType>>() {
            Some(self.not_legs_type_token())
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, type_: TypeId) -> Option<&mut dyn Any> {
        if type_ == TypeId::of::<Circuit<(usize, NonZeroUsize)>>() {
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
    let or_1 = OrLegs::new(circuit, &or_legs_token, |chip| (not_1.into_raw_parts(), chip));
    let or_2 = OrLegs::new(circuit, &or_legs_token, |chip| (not_2.into_raw_parts(), chip));
    *not_1.tag_mut(circuit) = or_2.into_raw_parts();
    *not_2.tag_mut(circuit) = or_1.into_raw_parts();
    not_1.on_changed(circuit, not_legs_token.type_().out(), |not_1, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let or_legs_token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
        let circuit = context.get::<Circuit<(usize, NonZeroUsize)>>().expect("Cicuit required");
        let or_2: Chip<(usize, NonZeroUsize)> = Chip::from_raw_parts(*not_1.tag(circuit));
        let &out = not_1.get(circuit, not_legs_token.type_().out());
        let in_2 = or_legs_token.type_().in_2();
        or_2.set_distinct(context, in_2, out);
    });
    not_2.on_changed(circuit, not_legs_token.type_().out(), |not_2, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let or_legs_token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
        let circuit = context.get::<Circuit<(usize, NonZeroUsize)>>().expect("Cicuit required");
        let or_1: Chip<(usize, NonZeroUsize)> = Chip::from_raw_parts(*not_2.tag(circuit));
        let &out = not_2.get(circuit, not_legs_token.type_().out());
        let in_2 = or_legs_token.type_().in_2();
        or_1.set_distinct(context, in_2, out);
    });
    or_1.on_changed(circuit, or_legs_token.type_().out(), |or_1, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let or_legs_token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
        let circuit = context.get::<Circuit<(usize, NonZeroUsize)>>().expect("Cicuit required");
        let not_1: Chip<(usize, NonZeroUsize)> = Chip::from_raw_parts(*or_1.tag(circuit));
        let &out = or_1.get(circuit, or_legs_token.type_().out());
        let in_ = not_legs_token.type_().in_();
        not_1.set_distinct(context, in_, out);
    });
    or_2.on_changed(circuit, or_legs_token.type_().out(), |or_2, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let or_legs_token = context.get::<DepTypeToken<OrLegsType>>().expect("OrLegsType required");
        let circuit = context.get::<Circuit<(usize, NonZeroUsize)>>().expect("Cicuit required");
        let not_2: Chip<(usize, NonZeroUsize)> = Chip::from_raw_parts(*or_2.tag(circuit));
        let &out = or_2.get(circuit, or_legs_token.type_().out());
        let in_ = not_legs_token.type_().in_();
        not_2.set_distinct(context, in_, out);
    });
    not_1.on_changed(circuit, not_legs_token.type_().out(), |not_1, context, _old| {
        let not_legs_token = context.get::<DepTypeToken<NotLegsType>>().expect("NotLegsType required");
        let circuit = context.get::<Circuit<(usize, NonZeroUsize)>>().expect("Cicuit required");
        let &out = not_1.get(circuit, not_legs_token.type_().out());
        println!("{}", if out { 1 } else { 0 });
    });
    TriggerContext::call(circuit, &or_legs_token, &not_legs_token, |context| {
        or_1.set_distinct(context, or_legs_token.type_().in_1(), true);
        or_1.set_distinct(context, or_legs_token.type_().in_1(), false);
        or_2.set_distinct(context, or_legs_token.type_().in_1(), true);
        or_2.set_distinct(context, or_legs_token.type_().in_1(), false);
        or_1.set_distinct(context, or_legs_token.type_().in_1(), true);
        or_1.set_distinct(context, or_legs_token.type_().in_1(), false);
        or_2.set_distinct(context, or_legs_token.type_().in_1(), true);
        or_2.set_distinct(context, or_legs_token.type_().in_1(), false);
    });
}
