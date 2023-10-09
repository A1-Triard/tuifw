#![deny(warnings)]

use std::env::{self};
use std::path::PathBuf;
use tuifw_xaml::{self};
use tuifw_xaml::xaml::Xaml;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ui.xaml");
    let mut xaml = Xaml::new();
    tuifw_xaml::reg_widgets(&mut xaml);
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    xaml.process_file("src/ui.xaml", out_dir.join("ui.rs")).unwrap();
}
