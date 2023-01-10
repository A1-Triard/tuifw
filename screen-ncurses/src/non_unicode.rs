use crate::common::*;
use crate::ncurses::*;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::cmp::{max, min};
use core::ops::Range;
use core::ptr::NonNull;
use core::str::{self};
use either::{Right, Left};
use errno_no_std::{Errno, errno};
use libc::*;
use panicking::panicking;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;
use unicode_width::UnicodeWidthChar;

struct Line {
    window: NonNull<WINDOW>,
    invalidated: bool,
}

pub struct Screen<A: Allocator> {
    lines: Vec<Line, A>,
    cols: usize,
    chs: Vec<chtype, A>,
    cd: iconv_t,
    dc: iconv_t,
}

impl<A: Allocator> !Sync for Screen<A> { }
impl<A: Allocator> !Send for Screen<A> { }

const ICONV_ERR: iconv_t = (-1isize) as usize as iconv_t;

impl<A: Allocator> Screen<A> {
    pub unsafe fn new_in(alloc: A) -> Result<Self, Errno> where A: Clone {
        if non_null(initscr()).is_err() { return Err(Errno(EINVAL)); }
        let mut s = Screen {
            lines: Vec::with_capacity_in(usize::from(LINES.clamp(0, i16::MAX.into()) as i16 as u16), alloc.clone()),
            cols: usize::from(COLS.clamp(0, i16::MAX.into()) as i16 as u16),
            chs: Vec::with_capacity_in(
                usize::from(LINES.clamp(0, i16::MAX.into()) as i16 as u16)
                    .checked_mul(usize::from(COLS.clamp(0, i16::MAX.into()) as i16 as u16))
                    .expect("OOM"),
                alloc
            ),
            cd: ICONV_ERR,
            dc: ICONV_ERR
        };
        s.cd = iconv_open(nl_langinfo(CODESET), b"UTF-8\0".as_ptr() as _);
        if s.cd == ICONV_ERR { return Err(errno()); }
        s.dc = iconv_open(b"UTF-8\0".as_ptr() as _, nl_langinfo(CODESET));
        if s.dc == ICONV_ERR { return Err(errno()); }
        init_settings()?;
        s.resize()?;
        Ok(s)
    }

    fn resize(&mut self) -> Result<(), Errno> {
        for line in &self.lines {
            non_err(unsafe { delwin(line.window.as_ptr()) })?;
        }
        self.lines.clear();
        let space = b' ' as c_char as chtype;
        let size = self.size();
        self.lines.reserve(usize::from(size.y as u16));
        self.cols = usize::from(size.x as u16);
        self.chs.resize(usize::from(size.y as u16).checked_mul(self.cols).expect("OOM"), space);
        for y in 0 .. size.y {
            let window = non_null(unsafe { newwin(1, 0, y as _, 0) }).unwrap();
            non_err(unsafe { keypad(window.as_ptr(), true) })?;
            self.lines.push(Line { window, invalidated: true });
        }
        Ok(())
    }

    unsafe fn drop_raw(&mut self) -> Result<(), Errno> {
        let e1 = non_err(endwin()).map(|_| ());
        let e2 = if self.cd != ICONV_ERR {
            if iconv_close(self.cd) == -1 {
                Err(errno())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        let e3 = if self.dc != ICONV_ERR {
            if iconv_close(self.dc) == -1 {
                Err(errno())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        if e1.is_err() { e1 } else if e2.is_err() { e2 } else { e3 }
    }

    fn update_raw(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Errno> {
        non_err(unsafe { curs_set(0) })?;
        for (chs, line) in self.chs.chunks(self.cols).zip(self.lines.iter_mut()).filter(|(_, l)| l.invalidated) {
            line.invalidated = false;
            if chs.is_empty() { continue; }
            non_err(unsafe { wmove(line.window.as_ptr(), 0, 0) })?;
            for &ch in chs {
                let _ = unsafe { waddch(line.window.as_ptr(), ch) };
            }
            non_err(unsafe { wnoutrefresh(line.window.as_ptr()) })?;
        }
        non_err(unsafe { doupdate() })?;
        let cursor = cursor.and_then(|cursor| {
            if (Rect { tl: Point { x: 0, y: 0 }, size: self.size() }).contains(cursor) {
                Some(cursor)
            } else {
                None
            }
        });
        let window = if let Some(cursor) = cursor {
            let window = self.lines[cursor.y as u16 as usize].window;
            non_err(unsafe { wmove(window.as_ptr(), 0, cursor.x as _) })?;
            non_err(unsafe { curs_set(1) })?;
            Some(window)
        } else if let Some(line) = self.lines.first() {
            if self.cols == 0 {
                None
            } else {
                let window = line.window;
                non_err(unsafe { wmove(window.as_ptr(), 0, 0) })?;
                Some(window)
            }
        } else {
            None
        };
        let window = window.unwrap_or_else(|| unsafe { NonNull::new(stdscr).unwrap() });
        unsafe { non_err(nodelay(window.as_ptr(), !wait)) }?;
        let e = read_event(window, |w| {
            let c = unsafe { wgetch(w.as_ptr()) };
            if c == ERR { return None; }
            if c & KEY_CODE_YES == 0 { return Some(Right(decode_char(self.dc, c as c_char as u8))); }
            Some(Left(c & !KEY_CODE_YES))
        })?;
        match e {
            Some(Event::Resize) => self.resize()?,
            Some(Event::Key(_, Key::Ctrl(Ctrl::L))) => unsafe { clearok(curscr, true); },
            _ => { }
        }
        Ok(e)
    }
}

impl<A: Allocator> Drop for Screen<A> {
    #![allow(clippy::panicking_unwrap)]
    fn drop(&mut self) {
        let e = unsafe { self.drop_raw() };
        if e.is_err() && !panicking() { e.unwrap(); }
    }
}

fn encode_char(cd: iconv_t, c: char) -> chtype {
    match c {
        '→' => return A_ALTCHARSET | 43,
        '←' => return A_ALTCHARSET | 44,
        '↑' => return A_ALTCHARSET | 45,
        '↓' => return A_ALTCHARSET | 46,
        '█' => return A_ALTCHARSET | 48,
        '♦' => return A_ALTCHARSET | 96,
        '▒' => return A_ALTCHARSET | 97,
        '°' => return A_ALTCHARSET | 102,
        '±' => return A_ALTCHARSET | 103,
        '░' => return A_ALTCHARSET | 104,
        '␋' => return A_ALTCHARSET | 105,
        '┘' => return A_ALTCHARSET | 106,
        '┐' => return A_ALTCHARSET | 107,
        '┌' => return A_ALTCHARSET | 108,
        '└' => return A_ALTCHARSET | 109,
        '┼' => return A_ALTCHARSET | 110,
        '⎺' => return A_ALTCHARSET | 111,
        '⎻' => return A_ALTCHARSET | 112,
        '─' => return A_ALTCHARSET | 113,
        '⎼' => return A_ALTCHARSET | 114,
        '⎽' => return A_ALTCHARSET | 115,
        '├' => return A_ALTCHARSET | 116,
        '┤' => return A_ALTCHARSET | 117,
        '┴' => return A_ALTCHARSET | 118,
        '┬' => return A_ALTCHARSET | 119,
        '│' => return A_ALTCHARSET | 120,
        '≤' => return A_ALTCHARSET | 121,
        '≥' => return A_ALTCHARSET | 122,
        'π' => return A_ALTCHARSET | 123,
        '≠' => return A_ALTCHARSET | 124,
        '£' => return A_ALTCHARSET | 125,
        '·' => return A_ALTCHARSET | 126,
        _ => { },
    }
    let mut buf = [0; 4];
    let c = c.encode_utf8(&mut buf);
    let mut c_len = c.len() as size_t;
    let mut c_ptr = c.as_ptr() as *const c_char as *mut c_char;
    let mut encoded = 0u8;
    let mut encoded_ptr = (&mut encoded) as *mut _ as *mut c_char;
    let mut encoded_len: size_t = 1;
    unsafe { iconv(
        cd,
        (&mut c_ptr) as *mut _,
        (&mut c_len) as *mut _,
        (&mut encoded_ptr) as *mut _,
        (&mut encoded_len) as *mut _
    ) };
    if encoded_len == 0 && encoded != 127 && encoded >= 32 {
        encoded as chtype
    } else {
        A_ALTCHARSET | 96
    }
}

fn decode_char(dc: iconv_t, c: u8) -> char {
    let mut c_ptr = &c as *const u8 as *const c_char as *mut c_char;
    let mut c_len: size_t = 1;
    let mut buf = [0; 4];
    let mut buf_ptr = buf.as_mut_ptr() as *mut c_char;
    let mut buf_len = buf.len() as size_t;
    let invalid = unsafe { iconv(
        dc,
        (&mut c_ptr) as *mut _,
        (&mut c_len) as *mut _,
        (&mut buf_ptr) as *mut _,
        (&mut buf_len) as *mut _
    ) };
    assert!(invalid == 0);
    assert_eq!(c_len, 0);
    str::from_utf8(&buf[.. (buf.len() - buf_len)]).unwrap().chars().next().unwrap()
}

impl<A: Allocator> base_Screen for Screen<A> {
    fn size(&self) -> Vector {
        Vector {
            x: (unsafe { COLS }).clamp(0, i16::MAX.into()) as i16,
            y: (unsafe { LINES }).clamp(0, i16::MAX.into()) as i16
        }
    }

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
        let chs = &mut self.chs[usize::from(p.y as u16) * self.cols .. (usize::from(p.y as u16) + 1) * self.cols];
        self.lines[p.y as u16 as usize].invalidated = true;
        let attr = unsafe { attr_ch(fg, bg) };
        let text = text.chars().filter(|c| c.width() == Some(1))
            .map(|c| encode_char(self.cd, c))
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
                chs[x as u16 as usize] = c | attr;
            }
            x += 1;
        }
        x0 .. x
    }

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Error> {
        Ok(self.update_raw(cursor, wait)?)
    }
}
