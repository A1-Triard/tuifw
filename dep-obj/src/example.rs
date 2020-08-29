use core::fmt::Display;

pub struct Data {
    pub x: i16,
    pub y: i16
}

context! {
    mod example_context {
        data (data_mut): mut Data,
        display: ref dyn Display,
        id: const usize,
    }
}

pub use example_context::Context as ExampleContext;
