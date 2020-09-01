#![deny(warnings)]

#![no_std]
#[macro_use]
extern crate alloc;
pub(crate) mod std {
    pub use core::*;
}

#[macro_use]
extern crate derivative;

#[doc(hidden)]
pub use core::ops::FnOnce as std_ops_FnOnce;

#[cfg(docsrs)]
pub mod context;

#[cfg(not(docsrs))]
mod context;

mod dep;
pub use dep::*;
