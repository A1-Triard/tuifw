#![deny(warnings)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::unit_arg)]
#![allow(clippy::option_map_unit_fn)]

#[macro_use]
extern crate macro_attr;
#[macro_use]
extern crate components_arena;
#[macro_use]
extern crate downcast;
#[macro_use]
extern crate dep_obj;

pub mod view;
