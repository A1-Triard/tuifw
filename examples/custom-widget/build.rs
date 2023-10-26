#![deny(warnings)]

use std::env::{self};
use std::path::PathBuf;
use tuifw_xaml::{self};
use tuifw_xaml::xaml::{XamlStruct, Xaml};
use tuifw_xaml::preprocessor::preprocess_xaml_file;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/ui.xaml");
    let mut xaml = Xaml::new();
    let r = tuifw_xaml::reg_widgets(&mut xaml);
    let floating_frame = XamlStruct::new(
        &mut xaml,
        Some(r.widget),
        "https://a1-triard.github.io/tuifw/2023/xaml/example",
        "FloatingFrame",
    );
    tuifw_xaml::set_widget_ctor(
        &mut xaml, floating_frame, "crate::floating_frame::FloatingFrame", r.widget_children
    );
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    preprocess_xaml_file(&xaml, "src/ui.xaml", out_dir.join("ui.rs")).unwrap();
}
