#![cfg(windows)]

#![feature(allocator_api)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::collapsible_if)]
#![allow(clippy::many_single_char_names)]

#![no_std]

extern crate alloc;

use alloc::alloc::Global;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::char::{self};
use core::cmp::min;
use core::num::NonZeroU16;
use core::ops::Range;
use core::ptr::{null_mut};
use core::str::{self};
use either::{Either, Right, Left};
use errno_no_std::{Errno, errno};
use num_traits::identities::Zero;
use panicking::panicking;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;
use unicode_width::UnicodeWidthChar;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::{WCHAR, HANDLE};
use winapi::um::synchapi::Sleep;
use winapi::um::wincontypes::*;
use winapi::um::wincontypes::INPUT_RECORD_Event;
use winapi::um::wincon::*;
use winapi::um::winnt::*;
use winapi::um::fileapi::*;
use winapi::um::consoleapi::*;
use winapi::um::handleapi::*;
use winapi::um::winuser::*;

fn non_zero<Z: Zero>(r: Z) -> Result<Z, Errno> {
    if r.is_zero() {
        Err(errno())
    } else {
        Ok(r)
    }
}

fn valid_handle(h: HANDLE) -> Result<HANDLE, Errno> {
    if h == INVALID_HANDLE_VALUE {
        Err(errno())
    } else {
        Ok(h)
    }
}

pub struct Screen<A: Allocator = Global> {
    max_size: Option<(u16, u16)>,
    h_input: HANDLE,
    h_output: HANDLE,
    buf: Vec<CHAR_INFO, A>,
    size: Vector,
    invalidated: Rect,
    cursor_is_visible: bool, 
}

impl Screen {
    pub fn new(max_size: Option<(u16, u16)>) -> Result<Self, Error> {
        Self::new_in(max_size, Global)
    }
}

fn replace_control_chars(c: char) -> char {
    if c < ' ' { return char::from_u32(0x2400 + c as u32).unwrap(); }
    if c == '\x7F' { return '\u{2421}'; }
    if ('\u{0080}' ..= '\u{00FF}').contains(&c) { return '\u{2426}'; }
    c
}

impl<A: Allocator> Screen<A> {
    pub fn new_in(max_size: Option<(u16, u16)>, alloc: A) -> Result<Self, Error> {
        unsafe { FreeConsole() };
        non_zero(unsafe { AllocConsole() })?;
        let window = unsafe { GetConsoleWindow() };
        assert_ne!(window, null_mut());
        let system_menu = unsafe { GetSystemMenu(window, FALSE) };
        if !system_menu.is_null() { // Wine lacks GetSystemMenu implementation
            let _ = non_zero(unsafe { DeleteMenu(system_menu, SC_CLOSE as UINT, MF_BYCOMMAND) }); // non-fatal
        }
        let mut s = Screen {
            max_size,
            h_input: INVALID_HANDLE_VALUE,
            h_output: INVALID_HANDLE_VALUE,
            buf: if let Some(max_size) = max_size {
                Vec::with_capacity_in(usize::from(max_size.0).checked_mul(usize::from(max_size.1)).expect("OOM"), alloc)
            } else {
                Vec::new_in(alloc)
            },
            size: Vector::null(),
            invalidated: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
            cursor_is_visible: false,
        };
        s.h_input = valid_handle(unsafe { CreateFileA(
            "CONIN$\0".as_ptr() as _,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut())
        })?;
        s.h_output = valid_handle(unsafe { CreateFileA(
            "CONOUT$\0".as_ptr() as _,
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            null_mut())
        })?;
        non_zero(unsafe { SetConsoleMode(s.h_input, ENABLE_EXTENDED_FLAGS | ENABLE_WINDOW_INPUT) })?;
        non_zero(unsafe { SetConsoleMode(s.h_output, 0) })?;
        s.resize()?;
        Ok(s)
    }

    fn init_screen_buffer(&mut self) -> Result<Vector, Errno> {
        for _ in 0 .. 50 {
            pump_messages();
        }
        let mut ci = CONSOLE_SCREEN_BUFFER_INFO {
            dwSize: COORD { X: 0, Y: 0 },
            dwCursorPosition: COORD { X: 0, Y: 0 },
            wAttributes: 0,
            srWindow: SMALL_RECT { Left: 0, Top: 0, Right: 0, Bottom: 0 },
            dwMaximumWindowSize: COORD { X: 0, Y: 0 }
        };
        non_zero(unsafe { GetConsoleScreenBufferInfo(self.h_output, &mut ci as *mut _) })?;
        let mut width = ci.srWindow.Right.saturating_sub(ci.srWindow.Left).saturating_add(1);
        let mut height = ci.srWindow.Bottom.saturating_sub(ci.srWindow.Top).saturating_add(1);
        ci.srWindow.Left = 0;
        ci.srWindow.Top = 0;
        ci.srWindow.Right = width - 1;
        ci.srWindow.Bottom = height - 1;
        let _ = non_zero(unsafe { SetConsoleWindowInfo(self.h_output, 1, &ci.srWindow as *const _) });
        let _ = non_zero(unsafe { SetConsoleScreenBufferSize(self.h_output, COORD { X: width, Y: height }) });
        non_zero(unsafe { FlushConsoleInputBuffer(self.h_input) })?;
        set_cursor_is_visible(self.h_output, self.cursor_is_visible)?;
        if let Some(max_size) = self.max_size {
            width = min(width as u16, max_size.0) as i16;
            height = min(height as u16, max_size.1) as i16;
        }
        Ok(Vector { x: width, y: height })
    }

    fn resize(&mut self) -> Result<(), Errno> {
        let size = self.init_screen_buffer()?;
        let mut space = CHAR_INFO {
            Attributes: 0,
            Char: CHAR_INFO_Char::default()
        };
        *unsafe { space.Char.UnicodeChar_mut() } = b' ' as WCHAR;
        self.size = size;
        self.buf.resize(usize::try_from(self.size.rect_area()).expect("OOM"), space);
        self.invalidated.size = Vector::null();
        Ok(())
    }

    fn encode_grapheme(g: char) -> Option<Either<u16, (u16, u16)>> {
        let g = replace_control_chars(g);
        let width = g.width()?;
        let mut buf = [0u16; 2];
        let g = g.encode_utf16(&mut buf[..]);
        if g.len() != 1 { return None; }
        if width == 1 {
            Some(Left(g[0]))
        } else if width == 2 {
            Some(Right((g[0], g[0])))
        } else {
            None
        }
    }

    fn start_text(size_x: i16, line: &mut [CHAR_INFO], x: i16) -> i16 {
        if x > 0 && x < size_x {
            if line[x as u16 as usize].Attributes & COMMON_LVB_TRAILING_BYTE != 0 {
                let col = &mut line[(x as u16 as usize) - 1];
                debug_assert!(col.Attributes & COMMON_LVB_LEADING_BYTE != 0);
                col.Attributes &= !COMMON_LVB_LEADING_BYTE;
                *unsafe { col.Char.UnicodeChar_mut() } = b' ' as WCHAR;
                x - 1
            } else {
                x
            }
        } else {
            x
        }
    }

    fn end_text(size_x: i16, line: &mut [CHAR_INFO], x: i16) -> i16 {
        if x > 0 && x < size_x {
            let col = &mut line[x as u16 as usize];
            if col.Attributes & COMMON_LVB_TRAILING_BYTE != 0 {
                col.Attributes &= !COMMON_LVB_TRAILING_BYTE;
                *unsafe { col.Char.UnicodeChar_mut() } = b' ' as WCHAR;
                x + 1
            } else {
                x
            }
        } else {
            x
        }
    }

    unsafe fn drop_raw(&mut self) -> Result<(), Errno> {
        if self.h_input != INVALID_HANDLE_VALUE {
            non_zero(CloseHandle(self.h_input))?;
        }
        if self.h_output != INVALID_HANDLE_VALUE {
            non_zero(CloseHandle(self.h_output))?;
        }
        non_zero(FreeConsole())?;
        Ok(())
    }

    fn update_raw(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Errno> {
        if !self.invalidated.is_empty() {
            let mut region = SMALL_RECT {
                Top: self.invalidated.t(),
                Left: self.invalidated.l(),
                Right: self.invalidated.r() - 1,
                Bottom: self.invalidated.b() - 1
            };
            non_zero(unsafe { WriteConsoleOutputW(
                self.h_output,
                self.buf.as_ptr(),
                COORD { X: self.size.x, Y: self.size.y },
                COORD { X: region.Left, Y: region.Top },
                &mut region as *mut _
            ) })?;
            self.invalidated.size = Vector::null();
        }
        let cursor = cursor.and_then(|cursor| {
            if (Rect { tl: Point { x: 0, y: 0 }, size: self.size() }).contains(cursor) {
                Some(cursor)
            } else {
                None
            }
        });
        self.cursor_is_visible = cursor.is_some();
        set_cursor_is_visible(self.h_output, self.cursor_is_visible)?;
        let (count, key, c, ctrl, alt) = loop {
            pump_messages();
            if !wait {
                let mut n: DWORD = 0;
                non_zero(unsafe { GetNumberOfConsoleInputEvents(self.h_input, &mut n as *mut _) })?;
                if n == 0 { return Ok(None); }
            }
            let mut input = INPUT_RECORD {
                EventType: 0,
                Event: INPUT_RECORD_Event::default()
            };
            let mut readed: DWORD = 0;
            non_zero(unsafe { ReadConsoleInputW(self.h_input, &mut input as *mut _, 1, &mut readed as *mut _) })?;
            assert_eq!(readed, 1);
            match input.EventType {
                WINDOW_BUFFER_SIZE_EVENT => {
                    self.resize()?;
                    return Ok(Some(Event::Resize));
                },
                KEY_EVENT => {
                    let e = unsafe { input.Event.KeyEvent() };
                    if e.bKeyDown != 0 {
                        break (
                            NonZeroU16::new(e.wRepeatCount).unwrap(),
                            e.wVirtualKeyCode,
                            *unsafe { e.uChar.UnicodeChar() },
                            e.dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) != 0,
                            e.dwControlKeyState & (LEFT_ALT_PRESSED | RIGHT_ALT_PRESSED) != 0
                        );
                    }
                },
                _ => { }
            }
        };
        Ok(match key as i32 {
            VK_RETURN => Some(Event::Key(count, Key::Enter)),
            VK_TAB => Some(Event::Key(count, Key::Tab)),
            VK_PRIOR => Some(Event::Key(count, Key::PageUp)),
            VK_NEXT => Some(Event::Key(count, Key::PageDown)),
            VK_HOME => Some(Event::Key(count, Key::Home)),
            VK_END => Some(Event::Key(count, Key::End)),
            VK_DOWN => Some(Event::Key(count, Key::Down)),
            VK_UP => Some(Event::Key(count, Key::Up)),
            VK_LEFT => Some(Event::Key(count, Key::Left)),
            VK_RIGHT => Some(Event::Key(count, Key::Right)),
            VK_DELETE => Some(Event::Key(count, Key::Delete)),
            VK_INSERT => Some(Event::Key(count, Key::Insert)),
            VK_F1 => Some(Event::Key(count, Key::F1)),
            VK_F2 => Some(Event::Key(count, Key::F2)),
            VK_F3 => Some(Event::Key(count, Key::F3)),
            VK_F4 => Some(Event::Key(count, Key::F4)),
            VK_F5 => Some(Event::Key(count, Key::F5)),
            VK_F6 => Some(Event::Key(count, Key::F6)),
            VK_F7 => Some(Event::Key(count, Key::F7)),
            VK_F8 => Some(Event::Key(count, Key::F8)),
            VK_F9 => Some(Event::Key(count, Key::F9)),
            VK_F10 => Some(Event::Key(count, Key::F10)),
            VK_F11 => Some(Event::Key(count, Key::F11)),
            VK_F12 => Some(Event::Key(count, Key::F12)),
            VK_ESCAPE => Some(Event::Key(count, Key::Escape)),
            VK_BACK => Some(Event::Key(count, Key::Backspace)),
            0x32 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::At))),
            0x41 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::A))),
            0x42 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::B))),
            0x43 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::C))),
            0x44 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::D))),
            0x45 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::E))),
            0x46 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::F))),
            0x47 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::G))),
            0x48 if ctrl => Some(Event::Key(count, Key::Backspace)),
            0x49 if ctrl => Some(Event::Key(count, Key::Tab)),
            0x4A if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::J))),
            0x4B if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::K))),
            0x4C if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::L))),
            0x4D if ctrl => Some(Event::Key(count, Key::Enter)),
            0x4E if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::N))),
            0x4F if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::O))),
            0x50 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::P))),
            0x51 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::Q))),
            0x52 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::R))),
            0x53 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::S))),
            0x54 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::T))),
            0x55 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::U))),
            0x56 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::V))),
            0x57 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::W))),
            0x58 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::X))),
            0x59 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::Y))),
            0x5A if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::Z))),
            VK_OEM_4 if ctrl => Some(Event::Key(count, Key::Escape)),
            VK_OEM_5 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::Backslash))),
            VK_OEM_6 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::Bracket))),
            0x36 if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::Caret))),
            VK_OEM_MINUS if ctrl => Some(Event::Key(count, Key::Ctrl(Ctrl::Underscore))),
            VK_OEM_2 if ctrl => Some(Event::Key(count, Key::Backspace)),
            _ => {
                assert!(!(0xD800 .. 0xDC00).contains(&c));
                let c = if (0xDC00 .. 0xE000).contains(&c) {
                    assert_eq!(count.get(), 1);
                    let mut input = INPUT_RECORD {
                        EventType: 0,
                        Event: INPUT_RECORD_Event::default()
                    };
                    let mut n: DWORD = 0;
                    non_zero(unsafe { GetNumberOfConsoleInputEvents(self.h_input, &mut n as *mut _) })?;
                    assert_ne!(n, 0);
                    let mut readed: DWORD = 0;
                    non_zero(unsafe { ReadConsoleInputW(self.h_input, &mut input as *mut _, 1, &mut readed as *mut _) })?;
                    assert_eq!(readed, 1);
                    assert_eq!(input.EventType, KEY_EVENT);
                    let e = unsafe { input.Event.KeyEvent() };
                    assert!(e.bKeyDown != 0);
                    assert_eq!(e.wRepeatCount, 1);
                    let h = *unsafe { e.uChar.UnicodeChar() };
                    assert!((0xD800 .. 0xDC00).contains(&h));
                    ((h as u32 - 0xD800) << 10) | (c as u32 - 0xDC00)
                } else {
                    c as u32
                };
                let c = char::from_u32(c).unwrap();
                if c >= ' ' && c != '\x7F' {
                    if alt {
                        Some(Event::Key(count, Key::Alt(c)))
                    } else {
                        Some(Event::Key(count, Key::Char(c)))
                    }
                } else {
                    None
                }
            }
        })
    }
}

fn pump_messages() {
    unsafe {
        Sleep(1);
        let mut msg = MSG::default();
        while PeekMessageA(&mut msg as *mut _, null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&mut msg as *mut _); 
            DispatchMessageA(&msg as *const _); 
        }
    }
}

fn set_cursor_is_visible(h_output: HANDLE, cursor_is_visible: bool) -> Result<(), Error> {
    let mut cursor = CONSOLE_CURSOR_INFO { dwSize: 100, bVisible: TRUE };
    cursor.bVisible = if !cursor_is_visible { TRUE } else { FALSE };
    non_zero(unsafe { SetConsoleCursorInfo(h_output, &cursor as *const _) })?;
    pump_messages();
    cursor.dwSize = 25;
    non_zero(unsafe { SetConsoleCursorInfo(h_output, &cursor as *const _) })?;
    pump_messages();
    cursor.bVisible = if cursor_is_visible { TRUE } else { FALSE };
    non_zero(unsafe { SetConsoleCursorInfo(h_output, &cursor as *const _) })?;
    pump_messages();
    Ok(())
}

impl<A: Allocator> Drop for Screen<A> {
    #[allow(clippy::panicking_unwrap)]
    fn drop(&mut self) {
        let e = unsafe { self.drop_raw() };
        if e.is_err() && !panicking() { e.unwrap(); }
    }
}

fn attr_w(fg: Fg, bg: Bg) -> WORD {
    let fg = match fg {
        Fg::Black => 0,
        Fg::DarkGray => FOREGROUND_INTENSITY,
        Fg::Red => FOREGROUND_RED,
        Fg::LightRed => FOREGROUND_INTENSITY | FOREGROUND_RED,
        Fg::Green => FOREGROUND_GREEN,
        Fg::LightGreen => FOREGROUND_INTENSITY | FOREGROUND_GREEN,
        Fg::Brown => FOREGROUND_RED | FOREGROUND_GREEN,
        Fg::Yellow => FOREGROUND_INTENSITY | FOREGROUND_RED | FOREGROUND_GREEN,
        Fg::Blue => FOREGROUND_BLUE,
        Fg::LightBlue => FOREGROUND_INTENSITY | FOREGROUND_BLUE,
        Fg::Magenta => FOREGROUND_RED | FOREGROUND_BLUE,
        Fg::LightMagenta => FOREGROUND_INTENSITY | FOREGROUND_RED | FOREGROUND_BLUE,
        Fg::Cyan => FOREGROUND_BLUE | FOREGROUND_GREEN,
        Fg::LightCyan => FOREGROUND_INTENSITY | FOREGROUND_BLUE | FOREGROUND_GREEN,
        Fg::LightGray => FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED,
        Fg::White => FOREGROUND_INTENSITY | FOREGROUND_BLUE | FOREGROUND_GREEN | FOREGROUND_RED,
    };
    let bg = match bg {
        Bg::Black | Bg::None => 0,
        Bg::Red => BACKGROUND_RED,
        Bg::Green => BACKGROUND_GREEN,
        Bg::Brown => BACKGROUND_RED | BACKGROUND_GREEN,
        Bg::Blue => BACKGROUND_BLUE,
        Bg::Magenta => BACKGROUND_RED | BACKGROUND_BLUE,
        Bg::Cyan => BACKGROUND_BLUE | BACKGROUND_GREEN,
        Bg::LightGray => BACKGROUND_BLUE | BACKGROUND_GREEN | BACKGROUND_RED
    };
    fg | bg
}

impl<A: Allocator> base_Screen for Screen<A> {
    fn size(&self) -> Vector { self.size }

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
        let size = self.size;
        let line = (p.y as u16 as usize) * (size.x as u16 as usize);
        let line = &mut self.buf[line .. line + size.x as u16 as usize];
        let attr = attr_w(fg, bg);
        let mut x0 = None;
        let mut x = p.x;
        let mut n = 0i16;
        for g in text.chars().filter_map(|g| Self::encode_grapheme(g)) {
            if x >= hard.end { break; }
            if n >= text_end { break; }
            let w = if g.is_left() { 1 } else { 2 };
            n = n.saturating_add(w);
            let before_text_start = n <= text_start;
            if before_text_start {
                x = min(hard.end, x.saturating_add(w));
                continue;
            }
            if x < hard.start {
                x = min(hard.end, x.saturating_add(w));
                if x > hard.start {
                    debug_assert!(x0.is_none());
                    let invalidated_l = Self::start_text(size.x, line, hard.start);
                    x0 = Some((invalidated_l, hard.start));
                    let col = &mut line[hard.start as u16 as usize];
                    col.Attributes = 0;
                    *unsafe { col.Char.UnicodeChar_mut() } = b' ' as WCHAR;
                }
                continue;
            }
            if x0.is_none() {
                let invalidated_l = Self::start_text(size.x, line, x);
                x0 = Some((invalidated_l, x));
            }
            let next_x = min(hard.end, x.saturating_add(w));
            if next_x - x < w {
                let col = &mut line[x as u16 as usize];
                col.Attributes = 0;
                *unsafe { col.Char.UnicodeChar_mut() } = b' ' as WCHAR;
                x = next_x;
                break;
            }
            match g {
                Left(c) => {
                    let col = &mut line[x as u16 as usize];
                    col.Attributes = attr;
                    *unsafe { col.Char.UnicodeChar_mut() } = c;
                    x += 1;
                },
                Right((l, t)) => {
                    let col = &mut line[x as u16 as usize];
                    col.Attributes = attr | COMMON_LVB_LEADING_BYTE;
                    *unsafe { col.Char.UnicodeChar_mut() } = l;
                    x += 1;
                    let col = &mut line[x as u16 as usize];
                    col.Attributes = attr | COMMON_LVB_TRAILING_BYTE;
                    *unsafe { col.Char.UnicodeChar_mut() } = t;
                    x += 1;
                }
            }
        }
        if let Some((invalidated_l, x0)) = x0 {
            let invalidated_r = Self::end_text(size.x, line, x);
            self.invalidated = self.invalidated
                .union(Rect::from_tl_br(Point { x: invalidated_l, y: p.y }, Point { x: invalidated_r, y: p.y + 1 }))
                .unwrap().right().unwrap()
            ;
            x0 .. x
        } else {
            0 .. 0
        }
    }

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Error> {
        Ok(self.update_raw(cursor, wait)?)
    }
}
