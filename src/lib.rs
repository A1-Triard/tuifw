#![feature(const_ptr_offset_from)]
#![feature(generic_associated_types)]
#![feature(never_type)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::unit_arg)]
#![allow(clippy::option_map_unit_fn)]

#![no_std]

extern crate alloc;

pub use tuifw_screen_base::*;

pub mod view;

mod base;
pub use base::*;

mod desk_top;
pub use desk_top::*;

mod window;
pub use window::*;

mod static_text;
pub use static_text::*;
