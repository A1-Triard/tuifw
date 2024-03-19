#![deny(warnings)]

use std::env::{self};
use std::path::PathBuf;
use tuifw_xaml::xaml::Xaml;
use tuifw_xaml::preprocessor::preprocess_xaml_file;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ui.xaml");
    let mut xaml = Xaml::new();
    tuifw_xaml::reg_widgets(&mut xaml);
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    preprocess_xaml_file(&xaml, "src/ui.xaml", out_dir.join("ui.rs")).unwrap();
}
