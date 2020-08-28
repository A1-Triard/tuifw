#![deny(warnings)]

#![no_std]
extern crate alloc;
pub(crate) mod std {
    pub use core::*;
}

#[macro_use]
extern crate derivative;

#[macro_use]
pub mod context;
pub mod property;

#[cfg(docsrs)]
pub mod example;
