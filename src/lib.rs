#![feature(const_fn_trait_bound)]
#![feature(const_maybe_uninit_as_ptr)]
#![feature(const_ptr_offset_from)]
#![feature(const_raw_ptr_deref)]
#![feature(option_result_unwrap_unchecked)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]

#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::unit_arg)]
#![allow(clippy::option_map_unit_fn)]

pub use tuifw_screen_base::*;

pub mod view;

mod base;
pub use base::*;

mod desk_top;
pub use desk_top::*;

mod window;
pub use window::*;
