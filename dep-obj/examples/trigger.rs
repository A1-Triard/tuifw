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
    use dep_obj::dep::{DepObj, DepProp};
    use dep_obj::reactive::Reactive;
    use components_arena::{Id, Arena, ComponentClassMutex};
    use downcast::Any;
    use std::fmt::Debug;

    pub trait ChipLegs: DepObj + Any + Debug + Send + Sync {
    }

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

    static CHIP_NODE: ComponentClassMutex<ChipNodeComponent> = ComponentClassMutex::new();

    #[derive(Derivative)]
    #[derivative(Debug(bound=""), Copy(bound=""), Clone(bound=""), Eq(bound=""), PartialEq(bound=""))]
    #[derivative(Hash(bound=""), Ord(bound=""), PartialOrd(bound=""))]
    pub struct Chip<Tag>(Id<ChipNode<Tag>>);

    impl<Tag> Chip<Tag> {
        pub fn new<T>(
            circuit: &mut Circuit<Tag>,
            legs_tag: impl FnOnce(Chip<Tag>) -> (Box<dyn ChipLegs>, Tag, T)
        ) -> T {
            circuit.arena.insert(|chip| {
                let (legs, tag, result) = legs_tag(Chip(chip));
                (ChipNode { chip: Chip(chip), legs, tag }, result)
            })
        }

        pub fn drop(self, circuit: &mut Circuit<Tag>) {
            circuit.arena.remove(self.0);
        }

        pub fn get<Legs: ChipLegs, T, Outer>(
            self,
            circuit: &Circuit<Tag>,
            prop: DepProp<Legs, Reactive<T, CircuitContext<Tag, Outer>>>
        ) -> &T {
            let legs = circuit.arena[self.0].legs.downcast_ref::<Legs>().expect("invalid cast");
            prop.get(legs.dep_props()).get()
        }

        pub fn set<Legs: ChipLegs, T, Outer>(
            self,
            circuit: &mut Circuit<Tag>,
            prop: DepProp<Legs, Reactive<T, CircuitContext<Tag, Outer>>>,
            value: T,
            context: &mut Outer
        ) -> T {
            let legs = circuit.arena[self.0].legs.downcast_mut::<Legs>().expect("invalid cast");
            let (old, on_changed) = prop.get_mut(legs.dep_props_mut()).set(value);
            CircuitContext::call(circuit, self, context, |context| on_changed.raise(context, &old));
            old
        }

        pub fn set_dist<Legs: ChipLegs, T: Eq, Outer>(
            self,
            circuit: &mut Circuit<Tag>,
            prop: DepProp<Legs, Reactive<T, CircuitContext<Tag, Outer>>>,
            value: T,
            context: &mut Outer
        ) -> T {
            let legs = circuit.arena[self.0].legs.downcast_mut::<Legs>().expect("invalid cast");
            let (old, on_changed) = prop.get_mut(legs.dep_props_mut()).set_dist(value);
            CircuitContext::call(circuit, self, context, |context| on_changed.raise(context, &old));
            old
        }

        pub fn on_changed<Legs: ChipLegs, T, Outer>(
            self,
            circuit: &mut Circuit<Tag>,
            prop: DepProp<Legs, Reactive<T, CircuitContext<Tag, Outer>>>,
            on_changed: fn(context: &mut CircuitContext<Tag, Outer>, old: &T),
        ) {
            let legs = circuit.arena[self.0].legs.downcast_mut::<Legs>().expect("invalid cast");
            prop.get_mut(legs.dep_props_mut()).on_changed(on_changed);
        }
    }

    #[derive(Debug)]
    pub struct Circuit<Tag> {
        arena: Arena<ChipNode<Tag>>,
    }

    impl<Tag> Circuit<Tag> {
        pub fn new() -> Self {
            Circuit {
                arena: Arena::new(&mut CHIP_NODE.lock().unwrap())
            }
        }
    }

    context! {
        mod<Tag, Outer> circuit_context {
            circuit (circuit_mut): mut Circuit<Tag>,
            chip: const Chip<Tag>,
            outer (outer_mut): mut Outer,
        }
    }

    pub use circuit_context::Context as CircuitContext;
}

mod and_chip {
    use crate::circuit::*;
    use dep_obj::dep::{DepObj, DepObjProps, DepProp, DepTypeBuilder, DepTypeToken};
    use dep_obj::reactive::Reactive;

    macro_attr! {
        #[derive(DepObjRaw!)]
        #[derive(Debug)]
        pub struct AndLegs {
            dep_props: DepObjProps<Self>,
        }
    }

    impl AndLegs {
        pub fn new<Tag, Outer>(circuit: &mut Circuit<Tag>, type_: &AndLegsType<Tag>) -> Chip<Tag> {
            let mut legs = AndLegs {
                dep_props: DepObjProps::new(type_.token())
            };
            type_.in_1().get_mut(&mut legs.dep_props).on_changed(Self::update::<Tag, Outer>);
            type_.in_2().get_mut(&mut legs.dep_props).on_changed(Self::update::<Tag, Outer>);
            Chip::new(circuit, |chip| (Box::new(legs) as _, chip))
        }

        fn update<Tag, Outer>(context: CircuitContext<Tag, Outer>, _old: bool) {
            let chip = context.chip();
            let in_1 = *chip.get(context.circuit(), type_.in_1());
            let in_2 = *chip.get(context.circuit(), type_.in_2());
            chip.set_dist(context.circuit_mut(), type_.out(), in_1 & in_2);
        }
    }

    impl DepObj for AndLegs {
        fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
        fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
    }

    pub struct AndLegsType<Tag> {
        token: DepTypeToken<AndLegs>,
        in_1: DepProp<AndLegs, Reactive<bool, CircuitContext<Tag, Outer>>>,
        in_2: DepProp<AndLegs, Reactive<bool, CircuitContext<Tag, Outer>>>,
        out: DepProp<AndLegs, Reactive<bool, CircuitContext<Tag, Outer>>>,
    }

    impl<Tag> AndLegsType<Tag> {
        pub fn token(&self) -> &DepTypeToken<AndLegs> { &self.token }
        pub fn in_1(&self) -> DepProp<AndLegs, Reactive<bool, CircuitContext<Tag, Outer>>> { self.in_1 }
        pub fn in_2(&self) -> DepProp<AndLegs, Reactive<bool, CircuitContext<Tag, Outer>>> { self.in_2 }
        pub fn out(&self) -> DepProp<AndLegs, Reactive<bool, CircuitContext<Tag, Outer>>> { self.out }

        pub fn new() -> Option<Self> {
            DepTypeBuilder::new().map(|mut builder| {
                let in_1 = builder.prop::<Reactive<bool, CircuitContext<Tag, Outer>>>(|| Reactive::new(false));
                let in_2 = builder.prop::<Reactive<bool, CircuitContext<Tag, Outer>>>(|| Reactive::new(false));
                let out = builder.prop::<Reactive<bool, CircuitContext<Tag, Outer>>>(|| Reactive::new(false));
                let token = builder.build();
                AndLegsType { token, in_1, in_2, out }
            })
        }
    }

    impl ChipLegs for AndLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use dep_obj::dep::{DepObj, DepObjProps, DepProp, DepTypeBuilder, DepTypeToken};
    use dep_obj::reactive::Reactive;

    macro_attr! {
        #[derive(DepObjRaw!)]
        #[derive(Debug)]
        pub struct NotLegs {
            dep_props: DepObjProps<Self>,
        }
    }

    impl NotLegs {
        pub fn new<Tag>(circuit: &mut Circuit<Tag>, type_: &NotLegsType<Tag>) -> Chip<Tag> {
            let mut legs = NotLegs {
                dep_props: DepObjProps::new(type_.token())
            };
            type_.in_().get_mut(&mut legs.dep_props).on_changed(|context, _old| {
                let chip = context.chip();
                let in_ = *chip.get(context.circuit(), type_.in_());
                chip.set_dist(context.circuit_mut(), type_.out(), !in_);
            });
            Chip::new(circuit, |chip| (Box::new(legs) as _, chip))
        }
    }

    impl DepObj for NotLegs {
        fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
        fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
    }

    pub struct NotLegsType<Tag> {
        token: DepTypeToken<NotLegs>,
        in_: DepProp<NotLegs, Reactive<bool, CircuitContext<Tag, Outer>>>,
        out: DepProp<NotLegs, Reactive<bool, CircuitContext<Tag, Outer>>>,
    }

    impl<Tag> NotLegsType<Tag> {
        pub fn token(&self) -> &DepTypeToken<NotLegs> { &self.token }
        pub fn in_(&self) -> DepProp<NotLegs, Reactive<bool, CircuitContext<Tag, Outer>>> { self.in_ }
        pub fn out(&self) -> DepProp<NotLegs, Reactive<bool, CircuitContext<Tag, Outer>>> { self.out }

        pub fn new() -> Option<Self> {
            DepTypeBuilder::new().map(|mut builder| {
                let in_ = builder.prop::<Reactive<bool, CircuitContext<Tag, Outer>>>(|| Reactive::new(false));
                let out = builder.prop::<Reactive<bool, CircuitContext<Tag, Outer>>>(|| Reactive::new(false));
                let token = builder.build();
                NotLegsType { token, in_, out }
            })
        }
    }

    impl ChipLegs for NotLegs { }

}

use circuit::*;
use and_chip::*;
use not_chip::*;

fn main() {
    let circuit = &mut Circuit::new();
    let and_legs_type = AndLegsType::new().unwrap();
    let not_legs_type = NotLegsType::new().unwrap();
    let and_1 = AndLegs::new(circuit, &and_legs_type);
    let and_2 = AndLegs::new(circuit, &and_legs_type);
    let not_1 = NotLegs::new(circuit, &not_legs_type);
    let not_2 = NotLegs::new(circuit, &not_legs_type);
    /*
    not_1.on_changed(circuit, type_.out(), |context, _old| {
        let chip = context.chip();
        let out = *chip.get(context.circuit(), type_.out());
        chip.set_dist(context.circuit_mut(), type_.in_2(), out);
    });
    not_2.on_changed(circuit, type_.out(), |context, _old| {
        let chip = context.chip();
        let out = *chip.get(context.circuit(), type_.out());
        chip.set_dist(context.circuit_mut(), type_.in_1(), out);
    });
    */
}
