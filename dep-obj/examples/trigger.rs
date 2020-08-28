#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate components_arena;
#[macro_use]
extern crate dep_obj;
#[macro_use]
extern crate downcast;

mod circuit {
    use dep_obj::dep::{DepObj, DepObjProps, DepProp};
    use dep_obj::reactive::Reactive;
    use components_arena::{Id, Arena, ComponentClassMutex};
    use downcast::Any;
    use std::fmt::Debug;

    pub trait ChipLegs: DepObj + Any + Debug + Send + Sync {
    }

    downcast!(dyn ChipLegs);

    macro_attr! {
        #[derive(Component!)]
        #[derive(Debug)]
        struct ChipNode {
            chip: Chip,
            legs: Box<dyn ChipLegs>,
        }
    }

    static CHIP_NODE: ComponentClassMutex<ChipNode> = ComponentClassMutex::new();

    #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct Chip(Id<ChipNode>);

    impl Chip {
        pub fn new<T>(circuit: &mut Circuit, legs: impl FnOnce(Chip) -> (Box<dyn ChipLegs>, T)) -> T {
            circuit.arena.insert(|chip| {
                let (legs, result) = legs(Chip(chip));
                (ChipNode { chip: Chip(chip), legs }, result)
            })
        }

        pub fn drop(self, circuit: &mut Circuit) {
            circuit.arena.remove(self.0);
        }

        pub fn get<Legs: ChipLegs, T>(self, circuit: &Circuit, prop: DepProp<Legs, Reactive<T, CircuitContext>>) -> &T {
            let legs = circuit.arena[self.0].legs.downcast_ref::<Legs>().expect("invalid cast");
            prop.get(legs.dep_props()).get()
        }
    }

    #[derive(Debug)]
    pub struct Circuit {
        arena: Arena<ChipNode>,
    }

    impl Circuit {
        pub fn new() -> Circuit {
            Circuit {
                arena: Arena::new(&mut CHIP_NODE.lock().unwrap())
            }
        }
    }

    context! {
        mod circuit_context {
            circuit: mut Circuit,
            chip: const Chip,
        }
    }

    pub use circuit_context::Context as CircuitContext;
}

mod and_chip {
    use crate::circuit::*;
    use dep_obj::dep::{DepObj, DepObjProps, DepProp, DepTypeBuilder, DepTypeToken};
    use dep_obj::reactive::Reactive;
    use once_cell::sync::{self};

    macro_attr! {
        #[derive(DepObjRaw!)]
        #[derive(Debug)]
        pub struct AndLegs {
            dep_props: DepObjProps<Self>,
        }
    }

    impl DepObj for AndLegs {
        fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
        fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
    }

    pub struct AndLegsType {
        token: DepTypeToken<AndLegs>,
        in1: DepProp<AndLegs, Reactive<bool, CircuitContext>>,
        in2: DepProp<AndLegs, Reactive<bool, CircuitContext>>,
        out: DepProp<AndLegs, Reactive<bool, CircuitContext>>,
    }

    impl AndLegsType {
        pub fn token(&self) -> &DepTypeToken<AndLegs> { &self.token }
        pub fn in1(&self) -> DepProp<AndLegs, Reactive<bool, CircuitContext>> { self.in1 }
        pub fn in2(&self) -> DepProp<AndLegs, Reactive<bool, CircuitContext>> { self.in2 }
        pub fn out(&self) -> DepProp<AndLegs, Reactive<bool, CircuitContext>> { self.out }
    }

    pub static AND_LEGS_TYPE: sync::Lazy<AndLegsType> = sync::Lazy::new(|| {
        let mut builder = DepTypeBuilder::new().expect("type locked");
        let in1 = builder.prop::<Reactive<bool, CircuitContext>>(|| Reactive::new(false));
        let in2 = builder.prop::<Reactive<bool, CircuitContext>>(|| Reactive::new(false));
        let out = builder.prop::<Reactive<bool, CircuitContext>>(|| Reactive::new(false));
        let token = builder.build();
        AndLegsType { token, in1, in2, out }
    });

    impl ChipLegs for AndLegs { }
}

mod not_chip {
    use crate::circuit::*;
    use dep_obj::dep::{DepObj, DepObjProps, DepProp};
    use dep_obj::reactive::Reactive;

    macro_attr! {
        #[derive(DepObjRaw!)]
        #[derive(Debug)]
        pub struct NotLegs {
            dep_props: DepObjProps<Self>,
        }
    }

    impl DepObj for NotLegs {
        fn dep_props(&self) -> &DepObjProps<Self> { &self.dep_props }
        fn dep_props_mut(&mut self) -> &mut DepObjProps<Self> { &mut self.dep_props }
    }

    impl ChipLegs for NotLegs { }

}

use circuit::*;

fn main() {
    
}
