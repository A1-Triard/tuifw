use crate::common::*;
use crate::ncurses::*;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Allocator;
use core::cmp::{max, min};
use core::iter::{once, repeat};
use core::ops::Range;
use core::ptr::NonNull;
use core::str::{self};
use either::{Right, Left};
use errno_no_std::errno;
use libc::*;
use panicking::panicking;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;
use unicode_width::UnicodeWidthChar;

struct Line {
    window: NonNull<WINDOW>,
    invalidated: bool,
    data: Range<i16>,
}

pub struct Screen<A: Allocator> {
    error_alloc: &'static dyn Allocator,
    max_size: Option<(u16, u16)>,
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
    fn errno(&self) -> Error {
        Error::System(Box::new_in(errno(), self.error_alloc))
    }

    pub unsafe fn new_in(
        max_size: Option<(u16, u16)>,
        error_alloc: Option<&'static dyn Allocator>,
        alloc: A
    ) -> Result<Self, Error> where A: Clone {
        let error_alloc = error_alloc.unwrap_or(&GLOBAL);
        set_err(non_null(initscr()), "initscr", error_alloc)?;
        let size = size(max_size);
        let mut s = Screen {
            error_alloc,
            max_size,
            lines: Vec::with_capacity_in(usize::from(max_size.map_or(size.y as u16, |m| m.1)), alloc.clone()),
            cols: usize::from(size.x as u16),
            chs: Vec::with_capacity_in(
                usize::from(max_size.map_or(size.y as u16, |m| m.1))
                    .checked_mul(usize::from(max_size.map_or(size.x as u16, |m| m.0)))
                    .expect("OOM"),
                alloc
            ),
            cd: ICONV_ERR,
            dc: ICONV_ERR
        };
        s.cd = iconv_open(nl_langinfo(CODESET), b"UTF-8\0".as_ptr() as _);
        if s.cd == ICONV_ERR { return Err(s.errno()); }
        s.dc = iconv_open(b"UTF-8\0".as_ptr() as _, nl_langinfo(CODESET));
        if s.dc == ICONV_ERR { return Err(s.errno()); }
        init_settings(error_alloc)?;
        s.resize()?;
        Ok(s)
    }

    fn resize(&mut self) -> Result<(), Error> {
        for line in &self.lines {
            set_err(non_err(unsafe { delwin(line.window.as_ptr()) }), "delwin", self.error_alloc)?;
        }
        self.lines.clear();
        let space = b' ' as c_char as chtype;
        let size = self.size();
        self.lines.reserve(usize::from(size.y as u16));
        self.cols = usize::from(size.x as u16);
        self.chs.resize(usize::from(size.y as u16).checked_mul(self.cols).expect("OOM"), space);
        for y in 0 .. size.y {
            let window = non_null(unsafe { newwin(1, 0, y as _, 0) }).unwrap();
            set_err(non_err(unsafe { keypad(window.as_ptr(), true) }), "keypad", self.error_alloc)?;
            self.lines.push(Line { window, invalidated: false, data: 0 .. size.x });
        }
        Ok(())
    }

    unsafe fn drop_raw(&mut self) -> Result<(), Error> {
        let e1 = set_err(non_err(endwin()).map(|_| ()), "endwin", self.error_alloc);
        let e2 = if self.cd != ICONV_ERR {
            if iconv_close(self.cd) == -1 {
                Err(self.errno())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        let e3 = if self.dc != ICONV_ERR {
            if iconv_close(self.dc) == -1 {
                Err(self.errno())
            } else {
                Ok(())
            }
        } else {
            Ok(())
        };
        if e1.is_err() { e1 } else if e2.is_err() { e2 } else { e3 }
    }

    fn update_raw(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Error> {
        set_err(non_err(unsafe { curs_set(0) }), "curs_set", self.error_alloc)?;
        for (chs, line) in self.chs.chunks(self.cols).zip(self.lines.iter_mut()).filter(|(_, l)| l.invalidated) {
            line.invalidated = false;
            if chs.is_empty() { continue; }
            set_err(non_err(unsafe { wmove(line.window.as_ptr(), 0, 0) }), "wmove", self.error_alloc)?;
            for &ch in chs {
                let _ = unsafe { waddch(line.window.as_ptr(), ch) };
            }
            set_err(non_err(unsafe { wnoutrefresh(line.window.as_ptr()) }), "wnoutrefresh", self.error_alloc)?;
        }
        set_err(non_err(unsafe { doupdate() }), "doupdate", self.error_alloc)?;
        let cursor = cursor.and_then(|cursor| {
            if (Rect { tl: Point { x: 0, y: 0 }, size: self.size() }).contains(cursor) {
                Some(cursor)
            } else {
                None
            }
        });
        let window = if let Some(cursor) = cursor {
            let window = self.lines[cursor.y as u16 as usize].window;
            set_err(non_err(unsafe { wmove(window.as_ptr(), 0, cursor.x as _) }), "wmove", self.error_alloc)?;
            set_err(non_err(unsafe { curs_set(1) }), "curs_set", self.error_alloc)?;
            Some(window)
        } else if let Some(line) = self.lines.first() {
            if self.cols == 0 {
                None
            } else {
                let window = line.window;
                set_err(non_err(unsafe { wmove(window.as_ptr(), 0, 0) }), "wmove", self.error_alloc)?;
                Some(window)
            }
        } else {
            None
        };
        let window = window.unwrap_or_else(|| unsafe { NonNull::new(stdscr).unwrap() });
        set_err(non_err(unsafe { nodelay(window.as_ptr(), !wait) }), "nodelay", self.error_alloc)?;
        let e = read_event(window, |w| {
            let c = unsafe { wgetch(w.as_ptr()) };
            if c == ERR { return None; }
            if c & KEY_CODE_YES == 0 { return Some(Right(decode_char(self.dc, c as c_char as u8))); }
            Some(Left(c & !KEY_CODE_YES))
        }, self.error_alloc)?;
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

fn size(max_size: Option<(u16, u16)>) -> Vector {
    let mut x = (unsafe { COLS }).clamp(0, i16::MAX.into()) as i16;
    let mut y = (unsafe { LINES }).clamp(0, i16::MAX.into()) as i16;
    if let Some(max_size) = max_size {
        x = min(max_size.0, x as u16) as i16;
        y = min(max_size.1, y as u16) as i16;
    }
    Vector { x, y }
}

fn encode_char(cd: iconv_t, c: char) -> Option<chtype> {
    match c {
        '→' => return Some(A_ALTCHARSET | 43),
        '←' => return Some(A_ALTCHARSET | 44),
        '↑' => return Some(A_ALTCHARSET | 45),
        '↓' => return Some(A_ALTCHARSET | 46),
        '█' => return Some(A_ALTCHARSET | 48),
        '♦' => return Some(A_ALTCHARSET | 96),
        '▒' => return Some(A_ALTCHARSET | 97),
        '°' => return Some(A_ALTCHARSET | 102),
        '±' => return Some(A_ALTCHARSET | 103),
        '░' => return Some(A_ALTCHARSET | 104),
        '␋' => return Some(A_ALTCHARSET | 105),
        '┘' => return Some(A_ALTCHARSET | 106),
        '┐' => return Some(A_ALTCHARSET | 107),
        '┌' => return Some(A_ALTCHARSET | 108),
        '└' => return Some(A_ALTCHARSET | 109),
        '┼' => return Some(A_ALTCHARSET | 110),
        '⎺' => return Some(A_ALTCHARSET | 111),
        '⎻' => return Some(A_ALTCHARSET | 112),
        '─' => return Some(A_ALTCHARSET | 113),
        '⎼' => return Some(A_ALTCHARSET | 114),
        '⎽' => return Some(A_ALTCHARSET | 115),
        '├' => return Some(A_ALTCHARSET | 116),
        '┤' => return Some(A_ALTCHARSET | 117),
        '┴' => return Some(A_ALTCHARSET | 118),
        '┬' => return Some(A_ALTCHARSET | 119),
        '│' => return Some(A_ALTCHARSET | 120),
        '≤' => return Some(A_ALTCHARSET | 121),
        '≥' => return Some(A_ALTCHARSET | 122),
        'π' => return Some(A_ALTCHARSET | 123),
        '≠' => return Some(A_ALTCHARSET | 124),
        '£' => return Some(A_ALTCHARSET | 125),
        '·' => return Some(A_ALTCHARSET | 126),
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
    if encoded_len == 0 {
        Some(encoded as chtype)
    } else {
        None
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
    fn size(&self) -> Vector { size(self.max_size) }

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
        let text = text.chars()
            .filter(|&x| x != '\0' && x.width().is_some())
            .flat_map(|c| encode_char(self.cd, c).map_or_else(
                || Left(repeat(A_ALTCHARSET | 96).take(c.width().unwrap())),
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
                chs[x as u16 as usize] = c | attr;
            }
            x += 1;
        }
        x0 .. x
    }

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Error> {
        Ok(self.update_raw(cursor, wait)?)
    }

    fn line_invalidated_range(&self, line: i16) -> &Range<i16> { &self.lines[usize::from(line as u16)].data }

    fn line_invalidated_range_mut(&mut self, line: i16) -> &mut Range<i16> { &mut self.lines[usize::from(line as u16)].data }
}
