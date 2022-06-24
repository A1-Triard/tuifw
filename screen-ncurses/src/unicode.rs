#![allow(clippy::never_loop)]

use crate::common::*;
use crate::ncurses::*;
use alloc::vec;
use alloc::vec::Vec;
use core::char::{self};
use core::cmp::{max, min};
use core::mem::{size_of};
use core::ops::Range;
use core::ptr::NonNull;
use either::{Left, Right};
use errno_no_std::Errno;
use libc::*;
use panicking::panicking;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;
use unicode_width::UnicodeWidthChar;

struct Line {
    window: NonNull<WINDOW>,
    invalidated: bool,
    cols: Vec<([char; CCHARW_MAX], attr_t)>,
}

pub struct Screen {
    lines: Vec<Line>,
}

impl !Sync for Screen { }
impl !Send for Screen { }

impl Screen {
    pub unsafe fn new() -> Result<Self, Errno> {
        if non_null(initscr()).is_err() { return Err(Errno(EINVAL)); }
        let mut s = Screen {
            lines: Vec::with_capacity(max(0, min(LINES, i16::MAX as _)) as i16 as u16 as usize),
        };
        init_settings()?;
        s.resize()?;
        Ok(s)
    }

    fn resize(&mut self) -> Result<(), Errno> {
        for line in &self.lines {
            non_err(unsafe { delwin(line.window.as_ptr()) })?;
        }
        self.lines.clear();
        let mut space_gr = ['\0'; CCHARW_MAX];
        space_gr[0] = ' ';
        space_gr[1] = '\0';
        let space = (space_gr, WA_NORMAL);
        let size = self.size();
        for y in 0 .. size.y {
            let window = non_null(unsafe { newwin(1, 0, y as _, 0) }).unwrap();
            non_err(unsafe { keypad(window.as_ptr(), true) })?;
            self.lines.push(Line {
                window,
                invalidated: true,
                cols: vec![space; size.x as u16 as usize],
            });
        }
        Ok(())
    }

    fn start_text(line: &mut Line, x: i16) {
        if x <= 0 { return; }
        let mut x = x as u16;
        if let Some(col) = line.cols.get(x as usize) {
            if col.0[0] != '\0' { return; }
        } else {
            return;
        }
        loop {
            debug_assert!(x > 0);
            x -= 1;
            let col = &mut line.cols[x as usize];
            let stop = col.0[0] != '\0';
            col.0[0] = ' ';
            col.0[1] = '\0';
            if stop { break; }
        }
    }

    fn end_text(line: &mut Line, mut x: i16) {
        if x <= 0 { return; }
        while let Some(ref mut col) = line.cols.get_mut(x as u16 as usize) {
            if col.0[0] != '\0' { break; }
            col.0[0] = ' ';
            col.0[1] = '\0';
            x += 1;
        }
    }

    fn update_raw(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Errno> {
        non_err(unsafe { curs_set(0) })?;
        assert_eq!(size_of::<char>(), size_of::<wchar_t>());
        for line in self.lines.iter_mut().filter(|l| l.invalidated) {
            line.invalidated = false;
            if line.cols.is_empty() { continue; }
            non_err(unsafe { wmove(line.window.as_ptr(), 0, 0) })?;
            for &col in &line.cols {
                if col.0[0] == '\0' { continue; }
                non_err(unsafe { wattrset(line.window.as_ptr(), col.1 as _) })?;
                let _ = unsafe { waddnwstr(line.window.as_ptr(), col.0.as_ptr() as _, CCHARW_MAX as _) };
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
            if line.cols.is_empty() {
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
            let mut c: wint_t = 0;
            let key = unsafe { wget_wch(w.as_ptr(), &mut c as *mut _) };
            if key == ERR { return None; }
            if key != KEY_CODE_YES { return Some(Right(char::from_u32(c as wchar_t as u32).unwrap())); }
            Some(Left(c as _))
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
        let e = unsafe { non_err(endwin()) };
        if e.is_err() && !panicking() { e.unwrap(); }
    }
}

struct Graphemes<'a>(&'a str);

impl<'a> Iterator for Graphemes<'a> {
    type Item = (&'a str, usize);

    fn next(&mut self) -> Option<(&'a str, usize)> {
        let mut chars = self.0.char_indices()
            .map(|(i, c)| (i, c.width().unwrap_or(1)))
            .skip_while(|&(_, w)| w == 0)
        ;
        if let Some((start, width)) = chars.next() {
            let end = 'r: loop {
                for _ in 1 .. CCHARW_MAX {
                    if let Some((i, w)) = chars.next() {
                        if w != 0 {
                            break 'r i;
                        }
                    } else {
                        break 'r self.0.len();
                    }
                }
                break 'r if let Some((i, _)) = chars.next() {
                    i
                } else {
                    self.0.len()
                };
            };
            let (item, tail) = self.0.split_at(end);
            self.0 = tail;
            Some((&item[start ..], width))
        } else {
            self.0 = &self.0[self.0.len() ..];
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.0.len()))
    }
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
        let line = &mut self.lines[p.y as u16 as usize];
        line.invalidated = true;
        let attr = unsafe { attr_ch(fg, bg) };
        let text = Graphemes(text);
        let mut x0 = None;
        let mut x = p.x;
        let mut n = 0i16;
        for (g, w) in text {
            if x >= hard.end { break; }
            if n >= text_end { break; }
            let w = min(w, i16::MAX as u16 as usize) as u16 as i16;
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
                    Self::start_text(line, hard.start);
                    x0 = Some(hard.start);
                    for i in hard.start .. x {
                        let col = &mut line.cols[i as u16 as usize];
                        col.0[0] = ' ';
                        col.0[1] = '\0';
                    }
                }
                continue;
            }
            if x0.is_none() {
                Self::start_text(line, x);
                x0 = Some(x);
            }
            let next_x = min(hard.end, x.saturating_add(w));
            if next_x - x < w {
                for i in x .. next_x {
                    let col = &mut line.cols[i as u16 as usize];
                    col.0[0] = ' ';
                    col.0[1] = '\0';
                }
                x = next_x;
                break;
            }
            let col = &mut line.cols[x as u16 as usize];
            let mut i = 0;
            for c in g.chars() {
                let c = if c < ' ' || c == '\x7F' || c.width().is_none() { ' ' } else { c };
                col.0[i] = c;
                i += 1;
            }
            if i <= CCHARW_MAX {
                col.0[i] = '\0';
            }
            col.1 = attr;
            for i in x + 1 .. next_x {
               line.cols[i as u16 as usize].0[0] = '\0';
            }
            x = next_x;
        }
        if let Some(x0) = x0 {
            Self::end_text(line, x);
            x0 .. x
        } else {
            0 .. 0
        }
    }

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Errno> {
        self.update_raw(cursor, wait)
    }
}
