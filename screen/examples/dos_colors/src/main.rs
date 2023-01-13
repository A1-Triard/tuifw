#![feature(alloc_error_handler)]
#![feature(const_maybe_uninit_as_mut_ptr)]
#![feature(const_mut_refs)]
#![feature(const_ptr_write)]
#![feature(const_trait_impl)]
#![feature(extern_types)]
#![feature(generic_arg_infer)]
#![feature(lang_items)]
#![feature(panic_info_message)]
#![feature(ptr_metadata)]
#![feature(raw_ref_op)]
#![feature(start)]
#![feature(unsize)]

#![deny(warnings)]
#![allow(clippy::assertions_on_constants)]

#![windows_subsystem="console"]
#![no_std]
#![no_main]

extern crate alloc;
extern crate dos_errno_and_panic;
extern crate pc_atomics;
extern crate rlibc;

mod no_std {
    use composable_allocators::{AsGlobal};
    use composable_allocators::stacked::{self, Stacked};
    use core::mem::MaybeUninit;

    const MEM_SIZE: usize = 32;

    static mut MEM: [MaybeUninit<u8>; MEM_SIZE] = [MaybeUninit::uninit(); _];

    #[global_allocator]
    static ALLOCATOR: AsGlobal<Stacked<stacked::CtParams<MEM_SIZE>>> =
        AsGlobal(Stacked::from_static_array(unsafe { &mut MEM }));

    #[cfg(windows)]
    #[alloc_error_handler]
    fn rust_oom(_: core::alloc::Layout) -> ! {
        panic!("OOM")
    }

    const ERROR_MEM_SIZE: usize = 256;

    static mut ERROR_MEM: [MaybeUninit<u8>; ERROR_MEM_SIZE] = [MaybeUninit::uninit(); _];

    pub static ERROR_ALLOCATOR: Stacked<stacked::CtParams<ERROR_MEM_SIZE>> =
        Stacked::from_static_array(unsafe { &mut ERROR_MEM });
}

mod dos {
    #[no_mangle]
    extern "C" fn _aulldiv() -> ! { panic!("10") }
    #[no_mangle]
    extern "C" fn _aullrem() -> ! { panic!("11") }
    #[no_mangle]
    extern "C" fn _chkstk() { }
    #[no_mangle]
    extern "C" fn _fltused() -> ! { panic!("13") }
    #[no_mangle]
    extern "C" fn strlen() -> ! { panic!("14") }
}

use tuifw_screen_base::{Bg, Fg, Screen, Point, Event, Key};

fn draw(screen: &mut dyn Screen) {
    let w = 0 .. screen.size().x;
    for (bg_n, bg) in Bg::iter_variants().enumerate() {
        let bg_n: i16 = bg_n.try_into().unwrap();
        for (fg_n, fg) in Fg::iter_variants().enumerate() {
            let fg_n: i16 = fg_n.try_into().unwrap();
            screen.out(Point { x: 3 * fg_n, y: bg_n }, fg, bg, " ■ ", w.clone(), w.clone());
        }
    }
}

extern {
    type PEB;
}

#[allow(non_snake_case)]
#[no_mangle]
extern "stdcall" fn mainCRTStartup(_: *const PEB) -> u64 {
    let mut screen = unsafe { tuifw_screen::init(None, Some(&no_std::ERROR_ALLOCATOR)) }.unwrap();
    let screen = screen.as_mut();
    draw(screen);
    loop {
        if let Some(e) = screen.update(None, true).unwrap() {
            if matches!(e, Event::Key(_, Key::Escape)) { break; }
            if matches!(e, Event::Resize) {
                let w = 0 .. screen.size().x;
                for x in 0 .. screen.size().x {
                    for y in 0 .. screen.size().y {
                        screen.out(Point { x, y }, Fg::LightGray, Bg::None, " ", w.clone(), w.clone());
                    }
                }
                draw(screen);
            }
        }
    }
    0
}
