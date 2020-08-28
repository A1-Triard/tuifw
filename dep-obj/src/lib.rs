#![deny(warnings)]

#![no_std]
#[macro_use]
extern crate alloc;
pub(crate) mod std {
    pub use core::*;
}

#[macro_use]
extern crate derivative;

#[macro_use]
pub mod context;
pub mod reactive;
pub mod dep;

#[cfg(docsrs)]
pub mod example;
