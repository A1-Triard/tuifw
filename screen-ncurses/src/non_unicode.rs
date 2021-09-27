use crate::common::*;
use crate::ncurses::*;
use alloc::vec;
use alloc::vec::Vec;
use core::cmp::{max, min};
use core::ops::Range;
use core::ptr::NonNull;
use core::str::{self};
use either::{Right, Left};
use errno::{Errno, errno};
use libc::*;
use panicking::panicking;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;
use unicode_normalization::UnicodeNormalization;
use unicode_width::UnicodeWidthChar;

struct Line {
    window: NonNull<WINDOW>,
    invalidated: bool,
    cols: Vec<chtype>,
}

pub struct Screen {
    lines: Vec<Line>,
    cd: IconvT,
    dc: IconvT,
}

impl !Sync for Screen { }
impl !Send for Screen { }

impl Screen {
    pub unsafe fn new() -> Result<Self, Errno> {
        if no_null(initscr()).is_err() { return Err(Errno(EINVAL)); }
        let mut s = Screen {
            lines: Vec::with_capacity(max(0, min(LINES, i16::MAX as _)) as i16 as u16 as usize),
            cd: IconvT::ERROR,
            dc: IconvT::ERROR
        };
        s.cd = IconvT::new(iconv_open(nl_langinfo(CODESET), b"UTF-8\0".as_ptr() as _));
        if s.cd.is_error() { return Err(errno()); }
        s.dc = IconvT::new(iconv_open(b"UTF-8\0".as_ptr() as _, nl_langinfo(CODESET)));
        if s.dc.is_error() { return Err(errno()); }
        init_settings()?;
        s.resize()?;
        Ok(s)
    }

    fn resize(&mut self) -> Result<(), Errno> {
        for line in &self.lines {
            no_err(unsafe { delwin(line.window.as_ptr()) })?;
        }
        self.lines.clear();
        let space = b' ' as c_char as chtype;
        let size = self.size();
        for y in 0 .. size.y {
            let window = no_null(unsafe { newwin(1, 0, y as _, 0) }).unwrap();
            no_err(unsafe { keypad(window.as_ptr(), true) })?;
            self.lines.push(Line {
                window,
                invalidated: true,
                cols: vec![space; size.x as u16 as usize],
            });
        }
        Ok(())
    }

    unsafe fn drop_raw(&mut self) -> Result<(), Errno> {
        no_err(endwin())?;
        if let Some(cd) = self.cd.ok() {
            if iconv_close(cd) == -1 {
                return Err(errno());
            }
        }
        if let Some(dc) = self.dc.ok() {
            if iconv_close(dc) == -1 {
                return Err(errno());
            }
        }
        Ok(())
    }

    fn update_raw(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Errno> {
        no_err(unsafe { curs_set(0) })?;
        for line in self.lines.iter_mut().filter(|l| l.invalidated) {
            line.invalidated = false;
            if line.cols.is_empty() { continue; }
            no_err(unsafe { wmove(line.window.as_ptr(), 0, 0) })?;
            for &col in &line.cols {
                let _ = unsafe { waddch(line.window.as_ptr(), col) };
            }
            no_err(unsafe { wnoutrefresh(line.window.as_ptr()) })?;
        }
        no_err(unsafe { doupdate() })?;
        let cursor = cursor.and_then(|cursor| {
            if (Rect { tl: Point { x: 0, y: 0 }, size: self.size() }).contains(cursor) {
                Some(cursor)
            } else {
                None
            }
        });
        let window = if let Some(cursor) = cursor {
            let window = self.lines[cursor.y as u16 as usize].window;
            no_err(unsafe { wmove(window.as_ptr(), 0, cursor.x as _) })?;
            no_err(unsafe { curs_set(1) })?;
            Some(window)
        } else if let Some(line) = self.lines.first() {
            if line.cols.is_empty() {
                None
            } else {
                let window = line.window;
                no_err(unsafe { wmove(window.as_ptr(), 0, 0) })?;
                Some(window)
            }
        } else {
            None
        };
        let window = window.unwrap_or_else(|| unsafe { NonNull::new(stdscr).unwrap() });
        unsafe { no_err(nodelay(window.as_ptr(), !wait)) }?;
        let dc = self.dc.ok().expect("iconv not initialized (2)");
        let e = read_event(window, |w| {
            let c = unsafe { wgetch(w.as_ptr()) };
            if c == ERR { return None; }
            if c & KEY_CODE_YES == 0 { return Some(Right(decode_char(dc, c as c_char as u8))); }
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

impl Drop for Screen {
    #![allow(clippy::panicking_unwrap)]
    fn drop(&mut self) {
        let e = unsafe { self.drop_raw() };
        if e.is_err() && !panicking() { e.unwrap(); }
    }
}

fn encode_char(cd: iconv_t, c: char) -> u8 {
    let mut buf = [0; 4];
    let c = c.encode_utf8(&mut buf);
    let mut c_len = c.len() as size_t;
    let mut c_ptr = c.as_ptr() as *const c_char as *mut c_char;
    let mut encoded = 0u8;
    let mut encoded_ptr = (&mut encoded) as *mut _ as *mut c_char;
    let mut encoded_len: size_t = 1;
    let invalid = unsafe { iconv(
        cd,
        (&mut c_ptr) as *mut _,
        (&mut c_len) as *mut _,
        (&mut encoded_ptr) as *mut _,
        (&mut encoded_len) as *mut _
    ) };
    assert!(invalid == 0 || invalid == 1);
    assert_eq!(c_len, 0);
    assert_eq!(encoded_len, 0);
    if encoded < 32 || encoded == 127 { b' ' } else { encoded }
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

impl base_Screen for Screen {
    fn size(&self) -> Vector {
        Vector {
            x: max(0, min(unsafe { COLS }, i16::MAX as _)) as i16,
            y: max(0, min(unsafe { LINES }, i16::MAX as _)) as i16
        }
    }

    fn out(
        &mut self,
        p: Point,
        fg: Color,
        bg: Option<Color>,
        attr: Attr,
        text: &str,
        hard: Range<i16>,
        soft: Range<i16>
    ) -> Range<i16> {
        debug_assert!(p.y >= 0 && p.y < self.size().y);
        debug_assert!(hard.start >= 0 && hard.end > hard.start && hard.end <= self.size().x);
        debug_assert!(soft.start >= 0 && soft.end > soft.start && soft.end <= self.size().x);
        let text_end = if soft.end <= p.x { return 0 .. 0 } else { soft.end.saturating_sub(p.x) };
        let text_start = if soft.start <= p.x { 0 } else { soft.start.saturating_sub(p.x) };
        let cd = self.cd.ok().expect("iconv not initialized (1)");
        let line = &mut self.lines[p.y as u16 as usize];
        line.invalidated = true;
        let attr = unsafe { attr_ch(fg, bg, attr) };
        let text = text.nfc().filter(|c| c.width() == Some(1))
            .map(|c| encode_char(cd, c))
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
                line.cols[x as u16 as usize] = c as chtype | attr;
            }
            x += 1;
        }
        x0 .. x
    }

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Errno> {
        self.update_raw(cursor, wait)
    }
}
