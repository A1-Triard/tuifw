use std::char::{self};
use std::cmp::{max, min};
use std::io::{self};
use std::mem::{size_of};
use std::ops::Range;
use std::ptr::NonNull;
use std::thread::{self};
use libc::*;
use tuifw_screen_base::*;
use tuifw_screen_base::Screen as base_Screen;
use crate::ncurses::*;
use crate::common::*;
use unicode_width::UnicodeWidthChar;
use unicode_segmentation::UnicodeSegmentation;
use either::{Left, Right};

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
    pub unsafe fn new() -> io::Result<Self> {
        if no_null(initscr()).is_err() { return Err(io::ErrorKind::Other.into()); }
        let mut s = Screen {
            lines: Vec::with_capacity(max(0, min(LINES, i16::MAX as _)) as i16 as u16 as usize),
        };
        init_settings()?;
        s.resize()?;
        Ok(s)
    }

    fn resize(&mut self) -> io::Result<()> {
        for line in &self.lines {
            no_err(unsafe { delwin(line.window.as_ptr()) })?;
        }
        self.lines.clear();
        let mut space_gr = ['\0'; CCHARW_MAX];
        space_gr[0] = ' ';
        space_gr[1] = '\0';
        let space = (space_gr, WA_NORMAL);
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

    fn start_text(line: &mut Line, x: i16) {
        if x <= 0 { return; }
        let mut x = x as u16;
        if let Some(ref col) = line.cols.get(x as usize) {
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
}

impl Drop for Screen {
    fn drop(&mut self) {
        let e = unsafe { no_err(endwin()) };
        if e.is_err() && !thread::panicking() { e.unwrap(); }
    }
}

fn sanitize_grapheme(g: &str) -> &str {
    let s = g
        .char_indices()
        .find_map(|(i, c)| if c.width().unwrap_or(1) > 0 { Some(i) } else { None })
        .unwrap_or_else(|| g.len())
    ;
    let g = &g[s ..];
    if let Some(w) = g.chars().next() {
        if g.chars().skip(1).any(|c| c.width().unwrap_or(1) > 0) {
            &g[.. w.len_utf8()]
        } else {
            let e = g.chars().take(CCHARW_MAX).map(|c| c.len_utf8()).sum();
            &g[.. e]
        }
    } else {
        g
    }
}

impl base_Screen for Screen {
    type Error = io::Error;

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
        let line = &mut self.lines[p.y as u16 as usize];
        line.invalidated = true;
        let attr = unsafe { attr_ch(fg, bg, attr) };
        let text = text.graphemes(true)
            .map(sanitize_grapheme)
            .filter_map(|g| g.chars().next().map(|w| (g, w.width().unwrap_or(1) as u16 as i16)))
        ;
        let mut x0 = None;
        let mut x = p.x;
        let mut n = 0i16;
        for (g, w) in text {
            if x >= hard.end { break; }
            if n >= text_end { break; }
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

    fn update(&mut self, cursor: Option<Point>, wait: bool) -> Result<Option<Event>, Self::Error> {
        no_err(unsafe { curs_set(0) })?;
        assert_eq!(size_of::<char>(), size_of::<wchar_t>());
        for line in self.lines.iter_mut().filter(|l| l.invalidated) {
            line.invalidated = false;
            if line.cols.is_empty() { continue; }
            no_err(unsafe { wmove(line.window.as_ptr(), 0, 0) })?;
            for &col in &line.cols {
                if col.0[0] == '\0' { continue; }
                no_err(unsafe { wattrset(line.window.as_ptr(), col.1 as _) })?;
                let _ = unsafe { waddnwstr(line.window.as_ptr(), col.0.as_ptr() as _, CCHARW_MAX as _) };
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
