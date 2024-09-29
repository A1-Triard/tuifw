#![feature(const_ptr_write)]
#![feature(const_trait_impl)]
#![feature(extern_types)]
#![feature(generic_arg_infer)]
#![feature(ptr_metadata)]
#![feature(start)]
#![feature(unsize)]

#![deny(warnings)]
#![allow(clippy::assertions_on_constants)]

#![windows_subsystem="console"]
#![no_std]
#![no_main]

extern crate alloc;
extern crate rlibc_ext;

mod no_std {
    use composable_allocators::{AsGlobal};
    use composable_allocators::stacked::Stacked;
    use core::mem::MaybeUninit;
    use core::ptr::addr_of_mut;

    const MEM_SIZE: usize = 32;

    static mut MEM: [MaybeUninit<u8>; MEM_SIZE] = [MaybeUninit::uninit(); _];

    #[global_allocator]
    static ALLOCATOR: AsGlobal<Stacked> =
        AsGlobal(Stacked::from_static_array(unsafe { &mut *addr_of_mut!(MEM) }));

    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! { panic_no_std::panic(info, b'P') }

    const ERROR_MEM_SIZE: usize = 256;

    static mut ERROR_MEM: [MaybeUninit<u8>; ERROR_MEM_SIZE] = [MaybeUninit::uninit(); _];

    pub static ERROR_ALLOCATOR: Stacked =
        Stacked::from_static_array(unsafe { &mut *addr_of_mut!(ERROR_MEM) });
}

use exit_no_std::exit;
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

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn mainCRTStartup() -> ! {
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
    exit(0)
}
