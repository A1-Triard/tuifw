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
extern crate pc_atomics;
extern crate rlibc;

mod no_std {
    use composable_allocators::{AsGlobal, NonWorking};
    use composable_allocators::stacked::{self, Stacked};
    use core::mem::MaybeUninit;

    #[global_allocator]
    static ALLOCATOR: AsGlobal<NonWorking> = AsGlobal(NonWorking);

    #[cfg(windows)]
    #[alloc_error_handler]
    fn rust_oom(_: core::alloc::Layout) -> ! {
        panic!("OOM")
    }

    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! { panic_no_std::panic(info, b'P') }

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

const CONTROL_CHARS: &str = "\
    \x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0A\x0B\x0C\x0D\x0E\x0F\
    \x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1A\x1B\x1C\x1D\x1E\x1F\
";

const DOS_CHARS: &str = "\
    .☺☻♥♦♣♠•◘○◙♂♀♪♫☼\
    ►◄↕‼¶§▬↨↑↓→←∟↔▲▼\
";

const WIDE_CHARS: &str = "好 女 子";

#[allow(clippy::redundant_clone)]
fn draw(screen: &mut dyn Screen) {
    let w = 0 .. screen.size().x;
    screen.out(Point { x: 0, y: 0 }, Fg::LightGray, Bg::Blue, CONTROL_CHARS, w.clone(), w.clone());
    screen.out(Point { x: 0, y: 1 }, Fg::LightGray, Bg::Blue, DOS_CHARS, w.clone(), w.clone());
    screen.out(Point { x: 0, y: 3 }, Fg::LightGray, Bg::Blue, WIDE_CHARS, w.clone(), w.clone());
}

extern {
    type PEB;
}

#[allow(non_snake_case)]
#[no_mangle]
extern "stdcall" fn mainCRTStartup(_: *const PEB) -> u64 {
    let mut screen = unsafe { tuifw_screen_dos::Screen::new(Some(&no_std::ERROR_ALLOCATOR)) }.unwrap();
    let screen = &mut screen;
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
