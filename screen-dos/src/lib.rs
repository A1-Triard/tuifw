#![feature(allocator_api)]
#![feature(effects)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::alloc::Allocator;
use core::cmp::{min, max};
use core::iter::{once, repeat};
use core::mem::{MaybeUninit, align_of, size_of};
use core::num::NonZeroU16;
use core::ops::Range;
use core::ptr::{self};
use core::slice::{self};
use dos_cp::CodePage;
use either::{Either, Left, Right};
use panicking::panicking;
use pc_ints::*;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;
use unicode_width::UnicodeWidthChar;

const GLOBAL: composable_allocators::Global = composable_allocators::Global;

pub struct Screen {
    error_alloc: &'static dyn Allocator,
    original_mode: u8,
    code_page: &'static CodePage,
}

impl Screen {
    /// # Safety
    ///
    /// This method may be invoked iff it is guaranteed the memory addresses
    /// in `0xB8000 .. 0xBBE80` are not used by Rust abstract machine.
    ///
    /// It is impossible to garantee this conditions on a library level.
    /// So this unsafity should be propagated through all wrappers to the final application.
    pub unsafe fn new(error_alloc: Option<&'static dyn Allocator>) -> Result<Self, Error> {
        let error_alloc = error_alloc.unwrap_or(&GLOBAL);
        let code_page = CodePage::load().map_err(|e| Error::System(Box::new_in(e, error_alloc)))?;
        let original_mode = int_10h_ah_0Fh_video_mode().al_mode;
        if original_mode != 0x03 {
            int_10h_ah_00h_set_video_mode(0x03).map_err(|_| Error::System(Box::new_in("cannot switch video mode", error_alloc)))?;
        } else {
            int_10h_ah_05h_set_video_active_page(0);
        }
        let video_ptr = (0xB800usize << 4) as *mut i16;
        assert!(size_of::<Range<i16>>() <= 80 * size_of::<i16>());
        assert!(align_of::<Range<i16>>() <= 32); // 32 is (0xB800 << 4) + 80 * 25 * 2 * 2 divisor
        let third_page_ptr = video_ptr.add(80 * 25 * 2) as *mut MaybeUninit<Range<i16>>;
        let data = slice::from_raw_parts_mut(third_page_ptr, 25);
        data.fill_with(|| MaybeUninit::new(0 .. 80));
        Ok(Screen {
            error_alloc,
            code_page,
            original_mode,
        })
    }
}

impl Drop for Screen {
    #[allow(clippy::panicking_unwrap)]
    fn drop(&mut self) {
        if self.original_mode != 0x03 {
            let e = int_10h_ah_00h_set_video_mode(self.original_mode)
                .map_err(|_| Error::System(Box::new_in("cannot switch video mode back", self.error_alloc)));
            if e.is_err() && !panicking() { e.unwrap(); }
        }
    }
}

fn attr(fg: Fg, bg: Bg) -> u8 {
    let fg = match fg {
        Fg::Black => 0x00,
        Fg::Blue => 0x01,
        Fg::Green => 0x02,
        Fg::Cyan => 0x03,
        Fg::Red => 0x04,
        Fg::Magenta => 0x05,
        Fg::Brown => 0x06,
        Fg::LightGray => 0x07,
        Fg::DarkGray => 0x08,
        Fg::LightBlue => 0x09,
        Fg::LightGreen => 0x0A,
        Fg::LightCyan => 0x0B,
        Fg::LightRed => 0x0C,
        Fg::LightMagenta => 0x0D,
        Fg::Yellow => 0x0E,
        Fg::White => 0x0F,
    };
    let bg = match bg {
        Bg::None | Bg::Black => 0x00,
        Bg::Blue => 0x10,
        Bg::Green => 0x20,
        Bg::Cyan => 0x30,
        Bg::Red => 0x40,
        Bg::Magenta => 0x50,
        Bg::Brown => 0x60,
        Bg::LightGray => 0x70,
    };
    fg | bg
}

fn use_dos_graph_chars(c: char) -> char {
    match c {
        '☺' => '\x01',
        '☻' => '\x02',
        '♥' => '\x03',
        '♦' => '\x04',
        '♣' => '\x05',
        '♠' => '\x06',
        '•' => '\x07',
        '◘' => '\x08',
        '○' => '\x09',
        '◙' => '\x0A',
        '♂' => '\x0B',
        '♀' => '\x0C',
        '♪' => '\x0D',
        '♫' => '\x0E',
        '☼' => '\x0F',
        '►' => '\x10',
        '◄' => '\x11',
        '↕' => '\x12',
        '‼' => '\x13',
        '¶' => '\x14',
        '§' => '\x15',
        '▬' => '\x16',
        '↨' => '\x17',
        '↑' => '\x18',
        '↓' => '\x19',
        '→' => '\x1A',
        '←' => '\x1B',
        '∟' => '\x1C',
        '↔' => '\x1D',
        '▲' => '\x1E',
        '▼' => '\x1F',
        c => c
    }
}
 
impl base_Screen for Screen {
    fn size(&self) -> Vector { Vector { x: 80, y: 25 } }

    fn out(
        &mut self,
        p: Point,
        fg: Fg,
        bg: Bg,
        text: &str,
        hard: Range<i16>,
        soft: Range<i16>
    ) -> Range<i16> {
        debug_assert!(p.y >= 0 && p.y < self.size().y);
        debug_assert!(hard.start >= 0 && hard.end > hard.start && hard.end <= self.size().x);
        debug_assert!(soft.start >= 0 && soft.end > soft.start && soft.end <= self.size().x);
        let text_end = if soft.end <= p.x { return 0 .. 0 } else { soft.end.saturating_sub(p.x) };
        let text_start = if soft.start <= p.x { 0 } else { soft.start.saturating_sub(p.x) };
        let line = p.y as u16 as usize;
        let line = ((0xB800usize << 4) + 80 * 25 * 2 + line * 80 * 2) as *mut u16;
        let attr = (attr(fg, bg) as u16) << 8;
        let text = text.chars()
            .filter(|&x| x != '\0' && x.width().is_some())
            .map(use_dos_graph_chars)
            .flat_map(|c| self.code_page.from_char(c).map_or_else(
                || Left(repeat(b'\x04').take(c.width().unwrap())),
                |c| Right(once(c))
            ))
            .take(text_end as u16 as usize)
        ;
        let mut before_hard_start = min(p.x, hard.start);
        let mut before_text_start = 0i16;
        let x0 = max(hard.start, p.x);
        let mut x = x0;
        for c in text {
            if x >= hard.end { break; }
            let visible_1 = if before_text_start < text_start {
                before_text_start += 1;
                false
            } else {
                true
            };
            let visible_2 = if before_hard_start < hard.start {
                before_hard_start += 1;
                false
            } else {
                true
            };
            if visible_1 && visible_2 {
                unsafe { ptr::write_volatile(line.add(x as u16 as usize), attr | c as u16); }
            }
            x += 1;
        }
        x0 .. x
    }

    fn update(&mut self, _cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Error> {
        let video_ptr = (0xB800usize << 4) as *mut u16;
        for i in 0 .. 80 * 25 {
            unsafe {
                let c: u16 = ptr::read_volatile(video_ptr.add(80 * 25 + i));
                ptr::write_volatile(video_ptr.add(i), c);
            }
        }
        loop {
            if let Some(c) = self.code_page.inkey().map_err(|_| Error::System(Box::new_in("read key error", self.error_alloc)))? {
                break Ok(dos_key(c).map(|c| Event::Key(NonZeroU16::new(1).unwrap(), c)));
            } else {
                if !wait {
                    break Ok(None);
                }
            }
        }
    }

    fn line_invalidated_range(&self, line: i16) -> &Range<i16> {
        assert!((0 .. 25).contains(&line));
        assert!(size_of::<Range<i16>>() <= 80 * size_of::<i16>());
        let video_ptr = (0xB800usize << 4) as *const i16;
        assert!(align_of::<Range<i16>>() <= 32); // 32 is (0xB800 << 4) + 80 * 25 * 2 * 2 divisor
        let third_page_ptr = unsafe { video_ptr.add(80 * 25 * 2) } as *const Range<i16>;
        unsafe { &*third_page_ptr.add(usize::from(line as u16)) }
    }

    fn line_invalidated_range_mut(&mut self, line: i16) -> &mut Range<i16> {
        assert!((0 .. 25).contains(&line));
        assert!(size_of::<Range<i16>>() <= 80 * size_of::<i16>());
        let video_ptr = (0xB800usize << 4) as *mut i16;
        assert!(align_of::<Range<i16>>() <= 32); // 32 is (0xB800 << 4) + 80 * 25 * 2 * 2 divisor
        let third_page_ptr = unsafe { video_ptr.add(80 * 25 * 2) } as *mut Range<i16>;
        unsafe { &mut *third_page_ptr.add(usize::from(line as u16)) }
    }
}

fn dos_ctrl(c: char) -> Option<Ctrl> {
    match c {
        '\x00' => Some(Ctrl::At),
        '\x01' => Some(Ctrl::A),
        '\x02' => Some(Ctrl::B),
        '\x03' => Some(Ctrl::C),
        '\x04' => Some(Ctrl::D),
        '\x05' => Some(Ctrl::E),
        '\x06' => Some(Ctrl::F),
        '\x07' => Some(Ctrl::G),
        '\x0a' => Some(Ctrl::J),
        '\x0b' => Some(Ctrl::K),
        '\x0c' => Some(Ctrl::L),
        '\x0e' => Some(Ctrl::N),
        '\x0f' => Some(Ctrl::O),
        '\x10' => Some(Ctrl::P),
        '\x11' => Some(Ctrl::Q),
        '\x12' => Some(Ctrl::R),
        '\x13' => Some(Ctrl::S),
        '\x14' => Some(Ctrl::T),
        '\x15' => Some(Ctrl::U),
        '\x16' => Some(Ctrl::V),
        '\x17' => Some(Ctrl::W),
        '\x18' => Some(Ctrl::X),
        '\x19' => Some(Ctrl::Y),
        '\x1a' => Some(Ctrl::Z),
        '\x1c' => Some(Ctrl::Backslash),
        '\x1d' => Some(Ctrl::Bracket),
        '\x1e' => Some(Ctrl::Caret),
        '\x1f' => Some(Ctrl::Underscore),
        _ => None
    }
}

fn dos_key(c: Either<u8, char>) -> Option<Key> {
    Some(match c {
        Right(c) => {
            if let Some(ctrl) = dos_ctrl(c) {
                Key::Ctrl(ctrl)
            } else {
                match c {
                    '\r' => Key::Enter,
                    '\t' => Key::Tab,
                    '\x1b' => Key::Escape,
                    '\x08' => Key::Backspace,
                    '\x7f' => Key::Backspace,
                    c => Key::Char(c)
                }
            }
        },
        Left(80) => Key::Down,
        Left(72) => Key::Up,
        Left(75) => Key::Left,
        Left(77) => Key::Right,
        Left(71) => Key::Home,
        Left(79) => Key::End,
        Left(83) => Key::Delete,
        Left(82) => Key::Insert,
        Left(81) => Key::PageDown,
        Left(73) => Key::PageUp,
        Left(59) => Key::F1,
        Left(60) => Key::F2,
        Left(61) => Key::F3,
        Left(62) => Key::F4,
        Left(63) => Key::F5,
        Left(64) => Key::F6,
        Left(65) => Key::F7,
        Left(66) => Key::F8,
        Left(67) => Key::F9,
        Left(68) => Key::F10,
        Left(133) => Key::F11,
        Left(134) => Key::F12,
        _ => return None,
    })
}
