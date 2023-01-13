#![deny(warnings)]

use std::env::var;

fn main() {
    if var("CARGO_CFG_TARGET_OS").unwrap() == "dos" {
        dos_cp_generator::build();
    }
}
