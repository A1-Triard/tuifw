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
    use dep_obj::reactive::{Reactive, Context, ContextExt};
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

        pub fn drop(self, circuit: &mut Circuit<Tag>) {
            circuit.arena.remove(self.0);
        }

        pub fn get<Legs: ChipLegs, T>(
            self,
            circuit: &Circuit<Tag>,
            prop: DepProp<Legs, Reactive<Chip<Tag>, T>>,
        ) -> &T {
            let legs = circuit.arena[self.0].legs.downcast_ref::<Legs>().expect("invalid cast");
            prop.get(legs.dep_props()).get()
        }

        pub fn set<Legs: ChipLegs, T>(
            self,
            context: &mut dyn Context,
            prop: DepProp<Legs, Reactive<Chip<Tag>, T>>,
            value: T,
        ) -> T {
            let circuit = context.get_mut::<Circuit<Tag>>().expect("Circuit required");
            let legs = circuit.arena[self.0].legs.downcast_mut::<Legs>().expect("invalid cast");
            let (old, on_changed) = prop.get_mut(legs.dep_props_mut()).set(value);
            on_changed.raise(self, context, &old);
            old
        }

        pub fn set_dist<Legs: ChipLegs, T: Eq>(
            self,
            context: &mut dyn Context,
            prop: DepProp<Legs, Reactive<Chip<Tag>, T>>,
            value: T,
        ) -> T {
            let circuit = context.get_mut::<Circuit<Tag>>().expect("Circuit required");
            let legs = circuit.arena[self.0].legs.downcast_mut::<Legs>().expect("invalid cast");
            let (old, on_changed) = prop.get_mut(legs.dep_props_mut()).set_dist(value);
            on_changed.raise(self, context, &old);
            old
        }

        pub fn on_changed<Legs: ChipLegs, T>(
            self,
            circuit: &mut Circuit<Tag>,
            prop: DepProp<Legs, Reactive<Chip<Tag>, T>>,
            on_changed: fn(owner: Chip<Tag>, context: &mut dyn Context, old: &T),
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
}

mod or_chip {
    use crate::circuit::*;
    use dep_obj::dep::{DepObj, DepObjProps, DepProp, DepTypeBuilder, DepTypeToken};
    use dep_obj::reactive::{Reactive, Context, ContextExt};

    macro_attr! {
        #[derive(DepObjRaw!)]
        #[derive(Debug)]
        pub struct OrLegs {
            dep_props: DepObjProps<Self>,
        }
    }

    impl OrLegs {
        pub fn new<Tag: 'static>(circuit: &mut Circuit<Tag>, tag: Tag, type_: &OrLegsType<Tag>) -> Chip<Tag> {
            let mut legs = OrLegs {
                dep_props: DepObjProps::new(type_.token())
            };
            type_.in_1().get_mut(&mut legs.dep_props).on_changed(Self::update::<Tag>);
            type_.in_2().get_mut(&mut legs.dep_props).on_changed(Self::update::<Tag>);
            Chip::new(circuit, |chip| (Box::new(legs) as _, tag, chip))
        }

        fn update<Tag: 'static>(chip: Chip<Tag>, context: &mut dyn Context, _old: &bool) {
            let type_ = context.get::<OrLegsType<Tag>>().expect("OrLegsType required");
            let circuit = context.get::<Circuit<Tag>>().expect("Cicuit required");
            let in_1 = *chip.get(circuit, type_.in_1());
            let in_2 = *chip.get(circuit, type_.in_2());
            let out = type_.out();
            chip.set_dist(context, out, in_1 & in_2);
        }
    }

    impl DepObj for OrLegs {
        fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
        fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
    }

    pub struct OrLegsType<Tag> {
        token: DepTypeToken<OrLegs>,
        in_1: DepProp<OrLegs, Reactive<Chip<Tag>, bool>>,
        in_2: DepProp<OrLegs, Reactive<Chip<Tag>, bool>>,
        out: DepProp<OrLegs, Reactive<Chip<Tag>, bool>>,
    }

    impl<Tag> OrLegsType<Tag> {
        pub fn token(&self) -> &DepTypeToken<OrLegs> { &self.token }
        pub fn in_1(&self) -> DepProp<OrLegs, Reactive<Chip<Tag>, bool>> { self.in_1 }
        pub fn in_2(&self) -> DepProp<OrLegs, Reactive<Chip<Tag>, bool>> { self.in_2 }
        pub fn out(&self) -> DepProp<OrLegs, Reactive<Chip<Tag>, bool>> { self.out }

        pub fn new() -> Option<Self> {
            DepTypeBuilder::new().map(|mut builder| {
                let in_1 = builder.prop::<Reactive<Chip<Tag>, bool>>(|| Reactive::new(false));
                let in_2 = builder.prop::<Reactive<Chip<Tag>, bool>>(|| Reactive::new(false));
                let out = builder.prop::<Reactive<Chip<Tag>, bool>>(|| Reactive::new(false));
                let token = builder.build();
                OrLegsType { token, in_1, in_2, out }
            })
        }
    }

    impl ChipLegs for OrLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use dep_obj::dep::{DepObj, DepObjProps, DepProp, DepTypeBuilder, DepTypeToken};
    use dep_obj::reactive::{Reactive, Context, ContextExt};

    macro_attr! {
        #[derive(DepObjRaw!)]
        #[derive(Debug)]
        pub struct NotLegs {
            dep_props: DepObjProps<Self>,
        }
    }

    impl NotLegs {
        pub fn new<Tag: 'static>(circuit: &mut Circuit<Tag>, tag: Tag, type_: &NotLegsType<Tag>) -> Chip<Tag> {
            let mut legs = NotLegs {
                dep_props: DepObjProps::new(type_.token())
            };
            type_.in_().get_mut(&mut legs.dep_props).on_changed(Self::update::<Tag>);
            Chip::new(circuit, |chip| (Box::new(legs) as _, tag, chip))
        }

        fn update<Tag: 'static>(chip: Chip<Tag>, context: &mut dyn Context, _old: &bool) {
            let type_ = context.get::<NotLegsType<Tag>>().expect("NotLegsType required");
            let circuit = context.get::<Circuit<Tag>>().expect("Cicuit required");
            let in_ = *chip.get(circuit, type_.in_());
            let out = type_.out();
            chip.set_dist(context, out, !in_);
        }
    }

    impl DepObj for NotLegs {
        fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
        fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
    }

    pub struct NotLegsType<Tag> {
        token: DepTypeToken<NotLegs>,
        in_: DepProp<NotLegs, Reactive<Chip<Tag>, bool>>,
        out: DepProp<NotLegs, Reactive<Chip<Tag>, bool>>,
    }

    impl<Tag> NotLegsType<Tag> {
        pub fn token(&self) -> &DepTypeToken<NotLegs> { &self.token }
        pub fn in_(&self) -> DepProp<NotLegs, Reactive<Chip<Tag>, bool>> { self.in_ }
        pub fn out(&self) -> DepProp<NotLegs, Reactive<Chip<Tag>, bool>> { self.out }

        pub fn new() -> Option<Self> {
            DepTypeBuilder::new().map(|mut builder| {
                let in_ = builder.prop::<Reactive<Chip<Tag>, bool>>(|| Reactive::new(false));
                let out = builder.prop::<Reactive<Chip<Tag>, bool>>(|| Reactive::new(false));
                let token = builder.build();
                NotLegsType { token, in_, out }
            })
        }
    }

    impl ChipLegs for NotLegs { }
}

use std::any::{Any, TypeId};
use dep_obj::reactive::Context;
use circuit::*;
use or_chip::*;
use not_chip::*;

context! {
    mod trigger_context {
        circuit (circuit_mut): mut Circuit<u8>,
        or_legs_type: ref OrLegsType<u8>,
        not_legs_type: ref NotLegsType<u8>,
    }
}


use trigger_context::Context as TriggerContext;

impl Context for TriggerContext {
    fn get_raw(&self, type_: TypeId) -> Option<&dyn Any> {
        if type_ == TypeId::of::<Circuit<u8>>() {
            Some(self.circuit())
        } else if type_ == TypeId::of::<OrLegsType<u8>>() {
            Some(self.or_legs_type())
        } else if type_ == TypeId::of::<NotLegsType<u8>>() {
            Some(self.not_legs_type())
        } else {
            None
        }
    }

    fn get_mut_raw(&mut self, type_: TypeId) -> Option<&mut dyn Any> {
        if type_ == TypeId::of::<Circuit<u8>>() {
            Some(self.circuit_mut())
        } else {
            None
        }
    }
}

fn main() {
    let circuit = &mut Circuit::new();
    let or_legs_type: OrLegsType<u8> =  OrLegsType::new().unwrap();
    let not_legs_type: NotLegsType<u8> = NotLegsType::new().unwrap();
    let or_1 = OrLegs::new(circuit, 1, &or_legs_type);
    let or_2 = OrLegs::new(circuit, 2, &or_legs_type);
    let not_1 = NotLegs::new(circuit, 3, &not_legs_type);
    let not_2 = NotLegs::new(circuit, 4, &not_legs_type);
    /*
    not_1.on_changed(circuit, not_legs_type.out(), |context, _old| {
        let chip = context.chip();
        let out = context.outer().not_legs_type
        let out = *chip.get(context.circuit(), type_.out());
        chip.set_dist(context.circuit_mut(), type_.in_2(), out);
    });
    */
    /*
    not_2.on_changed(circuit, type_.out(), |context, _old| {
        let chip = context.chip();
        let out = *chip.get(context.circuit(), type_.out());
        chip.set_dist(context.circuit_mut(), type_.in_1(), out);
    });
    */
}
