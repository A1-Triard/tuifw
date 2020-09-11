use components_arena::{Id, Component};
use macro_attr_2018::macro_attr;

pub struct Widget(Id<WidgetNode>);

pub struct Model(Id<ModelNode>);

macro_attr! {
    #[derive(Debug, Component!)]
    struct WidgetNode {
    }
}

macro_attr! {
    #[derive(Debug, Component!)]
    struct ModelNode {
    }
}

pub struct WidgetTree {
}

impl WidgetTree {
}
