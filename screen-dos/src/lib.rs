#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]

#![no_std]

use core::cmp::{min, max};
use core::mem::{MaybeUninit, forget, transmute};
use core::num::NonZeroU16;
use core::ops::Range;
use core::ptr::{self};
use core::slice::{self};
use dos_cp::CodePage;
use errno_no_std::Errno;
use panicking::panicking;
use pc_ints::*;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;

pub struct Screen {
    original_mode: u8,
    code_page: &'static CodePage,
}

struct RmAlloc {
    selector: u16,
}

impl Drop for RmAlloc {
    fn drop(&mut self) {
       let _ = int_31h_ax_0101h_rm_free(self.selector);
    }
}

fn assert_dos_3_3() -> Result<(), Error> {
    let dos_ver = int_21h_ah_30h_dos_ver();
    if dos_ver.al_major < 3 || dos_ver.al_major == 3 && dos_ver.ah_minor < 30 {
        Err(Error {
            errno: Errno(DOS_ERR_ENVIRONMENT_INVALID.into()),
            msg: "DOS >= 3.3 required",
        })
    } else {
        Ok(())
    }
}

fn load_code_page() -> Result<&'static CodePage, Error> {
    let code_page_memory = int_31h_ax_0100h_rm_alloc(8)
        .map_err(|e| Errno(e.ax_err.into()))?;
    let code_page_selector = RmAlloc { selector: code_page_memory.dx_selector };
    let code_page_memory = unsafe { slice::from_raw_parts_mut(
        ((code_page_memory.ax_segment as u32) << 4) as *mut u8,
        512
    ) };
    let code_page_n = int_21h_ax_6601h_code_page()
        .map_err(|e| Errno(e.ax_err.into()))?
        .bx_active;
    if code_page_n > 999 {
        return Err(Error {
            errno: Errno(37),
            msg: "unsupported code page"
        });
    }
    let mut code_page: [MaybeUninit<u8>; 13] = unsafe { MaybeUninit::uninit().assume_init() };
    (&mut code_page[.. 9]).copy_from_slice(unsafe { transmute(&b"CODEPAGE\\"[..]) });
    code_page[9].write(b'0' + (code_page_n / 100) as u8);
    code_page[10].write(b'0' + ((code_page_n % 100) / 10) as u8);
    code_page[11].write(b'0' + (code_page_n % 10) as u8);
    code_page[12].write(0);
    let code_page: [u8; 13] = unsafe { transmute(code_page) };
    let code_page = int_21h_ah_3Dh_open(code_page.as_ptr(), 0x00)
        .map_err(|e| Error {
            errno: Errno(e.ax_err.into()),
            msg: "cannot open code page file"
        })?
        .ax_handle;
    let mut code_page_buf: &mut [MaybeUninit<u8>] = unsafe { transmute(&mut code_page_memory[..]) };
    loop {
        if code_page_buf.is_empty() {
            let mut byte: MaybeUninit<u8> = MaybeUninit::uninit();
            let read = int_21h_ah_3Fh_read(code_page, slice::from_mut(&mut byte))
                .map_err(|e| Error {
                    errno: Errno(e.ax_err.into()),
                    msg: "cannot read code page file"
                })?
                .ax_read;
            if read != 0 {
                return Err(Error {
                    errno: Errno(DOS_ERR_DATA_INVALID.into()),
                    msg: "invalid code page file: too big"
                });
            }
            break;
        }
        let read = int_21h_ah_3Fh_read(code_page, code_page_buf)
            .map_err(|e| Error {
                errno: Errno(e.ax_err.into()),
                msg: "cannot read code page file"
            })?
            .ax_read;
        if read == 0 { break; }
        code_page_buf = &mut code_page_buf[read as usize ..];
    }
    if !code_page_buf.is_empty() {
        return Err(Error {
            errno: Errno(DOS_ERR_DATA_INVALID.into()),
            msg: "invalid code page file: too small"
        });
    }
    let code_page = unsafe { &*(code_page_memory.as_ptr() as *const CodePage) };
    forget(code_page_selector);
    Ok(code_page)
}

impl Screen {
    /// # Safety
    ///
    /// This method may be invoked iff it is guaranteed the memory addresses
    /// in `0xB8000 .. 0xBBE80` are not used by Rust abstract machine.
    ///
    /// It is impossible to garantee this conditions on a library level.
    /// So this unsafity should be propagated through all wrappers to the final application.
    pub unsafe fn new() -> Result<Self, Error> {
        assert_dos_3_3()?;
        let code_page = load_code_page()?;
        let original_mode = int_10h_ah_0Fh_video_mode().al_mode;
        int_10h_ah_00h_set_video_mode(0x03).map_err(|_| Error {
            errno: Errno(DOS_ERR_NET_REQUEST_NOT_SUPPORTED.into()),
            msg: "cannot switch video mode"
        })?;
        Ok(Screen {
            code_page,
            original_mode,
        })
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        let e = int_10h_ah_00h_set_video_mode(self.original_mode).map_err(|_| Error {
            errno: Errno(DOS_ERR_NET_REQUEST_NOT_SUPPORTED.into()),
            msg: "cannot switch video mode back"
        });
        if e.is_err() && !panicking() { e.unwrap(); }
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

fn key(_ch: u8) -> Option<Key> {
    None
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
            .map(|c| self.code_page.from_char(c).unwrap_or(b' '))
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
        let al_char = loop {
            if let Some(AlChar { al_char }) = int_21h_ah_06h_dl_FFh_inkey() {
                break Some(al_char)
            } else {
                if !wait {
                    break None;
                }
            }
        };
        if let Some(al_char) = al_char {
            if al_char != 0 {
                self.code_page.to_char(al_char).map(Key::Char)
            } else {
                key(
                    int_21h_ah_06h_dl_FFh_inkey()
                        .ok_or(Errno(DOS_ERR_READ_FAULT.into()))?
                        .al_char
                )
            }.map(|x| Ok(Event::Key(unsafe { NonZeroU16::new_unchecked(1) }, x))).transpose()
        } else {
            Ok(None)
        }
    }
}
