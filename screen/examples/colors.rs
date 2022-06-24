#![windows_subsystem = "windows"]

#![deny(warnings)]

use std::iter::once;
use tuifw_screen::{Screen, Point, Color, Attr, Event, Key};

fn draw(screen: &mut dyn Screen) {
    let colors_count: u8 = Color::iter_variants().count().try_into().unwrap();
    let variants = Color::iter_variants()
        .map(Some)
        .enumerate()
        .map(|(n, c)| (n.try_into().unwrap(), c))
        .chain(once((colors_count, None)))
        .flat_map(|(bg_n, bg)| Color::iter_variants()
            .enumerate()
            .map(move |(fg_n, fg)| ((bg_n, bg), (fg_n, fg)))
        )
        .flat_map(|(bg, fg)| [false, true]
            .into_iter()
            .map(move |reverse| (bg, fg, reverse))
        )
        .flat_map(|(bg, fg, reverse)| [false, true]
            .into_iter()
            .map(move |intense| (bg, fg, reverse, intense))
        )
    ;
    let w = 0 .. screen.size().x;
    for (bg, fg, reverse, intense) in variants {
        let x = (3 * fg.0 as u16 + if reverse { 2 + 3 * colors_count as u16 } else { 0 }) as i16;
        let y = (bg.0 as u16 + if intense { 1 + (colors_count as u16 + 1) } else { 0 }) as i16;
        let attr =
            if reverse { Attr::REVERSE } else { Attr::empty() } |
            if intense { Attr::INTENSE } else { Attr::empty() };
        screen.out(Point { x, y }, fg.1, bg.1, attr, " ■ ", w.clone(), w.clone());
    }
}

fn main() {
    let mut screen = unsafe { tuifw_screen::init() }.unwrap();
    let screen = screen.as_mut();
    draw(screen);
    loop {
        if let Some(e) = screen.update(None, true).unwrap() {
            if matches!(e, Event::Key(_, Key::Escape)) { break; }
        }
    }
}
