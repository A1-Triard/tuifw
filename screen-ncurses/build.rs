#![deny(warnings)]
#![allow(clippy::needless_raw_string_hashes)]

use std::process::{Command, Stdio};
use std::fs::{File};
use std::io::{Write};
use std::path::{PathBuf};
use std::env::{self};
use pkg_config::Library;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PKG_CONFIG_PATH");
    let target = env::var("TARGET").unwrap();
    let target = target.split('-').nth(2).unwrap();
    if target != "windows" {
        let ncurses_lib = pkg_config::Config::new()
            .atleast_version("5.0")
            .probe("ncursesw")
            .unwrap();
        generate_curses_types_rs(&ncurses_lib);
    }
}

fn generate_curses_types_rs(lib: &Library) {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let c_file = out_dir.join("curses_types.c");
    let bin_file = out_dir.join("curses_types");
    let rs_file = out_dir.join("curses_types.rs");

    {
        let c_file_display = c_file.display();
        let mut c_file = File::create(&c_file).unwrap_or_else(|_| panic!("cannot create {c_file_display}"));
        c_file.write_all(br##"
#include <stdio.h>
#include <stdalign.h>
#include <iconv.h>
#include <limits.h>
#include <wchar.h>
#define _XOPEN_SOURCE_EXTENDED 1
#define NCURSES_WIDECHAR 1
#include <curses.h>
int main() {
    printf("pub type chtype = u%zd;\n", sizeof(chtype) * CHAR_BIT);
    printf("pub type wint_t = u%zd;\n", sizeof(wint_t) * CHAR_BIT);
    printf("pub const CCHARW_MAX: usize = %d;\n", CCHARW_MAX);
    printf("pub const BUTTON1_PRESSED: c_ulong = %lu;\n", (unsigned long)BUTTON1_PRESSED);
    printf("pub const BUTTON1_RELEASED: c_ulong = %lu;\n", (unsigned long)BUTTON1_RELEASED);
    return 0;
}
"##).unwrap_or_else(|_| panic!("cannot write {c_file_display}"));
    }

    let mut build = cc::Build::new();
    for include_path in lib.include_paths.iter() {
        build.include(include_path);
    }
    let mut compiler = build.try_get_compiler().unwrap().to_command();
    compiler.arg("-o").arg(&bin_file).arg(&c_file);
    let compiler_status = compiler.stdin(Stdio::null()).status()
        .unwrap_or_else(|_| panic!("cannot compile {}", c_file.display()));
    if !compiler_status.success() {
        panic!("{} compilation failed with non-zero {}", c_file.display(), compiler_status);
    }
    let rs_file = File::create(&rs_file)
        .unwrap_or_else(|_| panic!("cannot create {}", rs_file.display()));
    Command::new(&bin_file).stdin(Stdio::null()).stdout(rs_file).status()
        .unwrap_or_else(|_| panic!("{} failed", bin_file.display()));
}
