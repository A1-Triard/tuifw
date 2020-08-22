#![deny(warnings)]
#![feature(never_type)]

#[macro_use]
extern crate components_arena;
#[macro_use]
extern crate derivative;

use std::cmp::{min, max};
use std::hint::unreachable_unchecked;
use std::mem::replace;
use std::ops::Range;
use components_arena::{Arena, Id, ComponentClassMutex};
use tuifw_screen_base::{Screen, Rect, Point, Vector, Attr, Color, Event};

fn invalidate_rect(invalidated: (&mut Vec<Range<i16>>, Vector), rect: Rect) {
    debug_assert_eq!(invalidated.0.len(), invalidated.1.y as u16 as usize);
    let rect = rect.intersect(Rect { tl: Point { x: 0, y: 0 }, size: invalidated.1 });
    if rect.is_empty() { return; }
    let l = rect.l();
    let r = rect.r();
    for y in rect.t() .. rect.b() {
        let row = &mut invalidated.0[y as u16 as usize];
        row.start = min(row.start, l);
        row.end = max(row.end, r);
    }
}

fn rect_invalidated(invalidated: (&Vec<Range<i16>>, Vector), rect: Rect) -> bool {
    debug_assert_eq!(invalidated.0.len(), invalidated.1.y as u16 as usize);
    let rect = rect.intersect(Rect { tl: Point { x: 0, y: 0 }, size: invalidated.1 });
    if rect.is_empty() { return false; }
    let l = rect.l();
    let r = rect.r();
    for y in rect.t() .. rect.b() {
        let row = &invalidated.0[y as u16 as usize];
        if row.end == row.start { continue; }
        if l < row.start {
            if r > row.end { return true; }
        } else if l < row.end {
            return true;
        }
    }
    false
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct DrawingPort<Error> {
    #[derivative(Debug="ignore")]
    screen: Box<dyn Screen<Error=Error>>,
    invalidated: Vec<Range<i16>>,
    offset: Vector,
    size: Vector,
    cursor: Option<Point>,
}

impl<Error> DrawingPort<Error> {
    pub fn out(&mut self, p: Point, fg: Color, bg: Option<Color>, attr: Attr, text: &str) {
        if p.y as u16 >= self.size.y as u16 || self.size.x == 0 { return; }
        let p = p.offset(self.offset);
        if p.y < 0 || p.y >= self.screen.size().y { return; }
        let row = &mut self.invalidated[p.y as u16 as usize];
        if p.x >= row.end { return; }

        let window_start = Point { x: 0, y: 0 }.offset(self.offset).x;
        let window_end = Point { x: 0, y: 0 }.offset(self.size + self.offset).x;
        let chunks = if window_start <= window_end {
            if window_end <= 0 || window_start >= self.screen.size().x { return; }
            [max(0, window_start) .. min(self.screen.size().x,  window_end), 0 .. 0]
        } else {
            if window_end > 0 && window_start < self.screen.size().x {
                [0 .. window_end, window_start .. self.screen.size().x]
            } else if window_end > 0 {
                [0 .. window_end, 0 .. 0]
            } else if window_start < self.screen.size().x {
                [window_start .. self.screen.size().x, 0 .. 0]
            } else {
                return
            }
        };

        for chunk in &chunks {
            if chunk.start >= chunk.end { continue; }
            let out = self.screen.out(p, fg, bg, attr, text, chunk.clone(), row.clone());
            if out.start >= out.end { continue; }
            row.start = min(row.start, out.start);
            row.end = max(row.end, out.end);
            if let Some(cursor) = self.cursor {
                if cursor.y == p.y && cursor.x >= out.start && cursor.x < out.end {
                    self.cursor = None;
                }
            }
        }
    }

    pub fn cursor(&mut self, p: Point) {
        if self.cursor.is_some() { return; }
        let p = p.offset(self.offset);
        if p.y < 0 || p.y >= self.screen.size().y { return; }
        let row = &self.invalidated[p.y as u16 as usize];
        if p.x < row.start || p.x >= row.end { return; }
        self.cursor = Some(p);
    }

    pub fn fill(&mut self, mut f: impl FnMut(&mut Self, Point)) {
        for y in 0 .. self.screen.size().y {
            for x in self.invalidated[y as u16 as usize].clone() {
                f(self, Point { x, y }.offset(-self.offset));
            }
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
struct WindowData<Tag> {
    parent: Option<Id<WindowData<Tag>>>,
    next: Id<WindowData<Tag>>,
    last_child: Option<Id<WindowData<Tag>>>,
    bounds: Rect,
    #[derivative(Debug="ignore")]
    tag: Tag,
}

Component!((class=WindowDataComponent) struct WindowData<Tag> { ... });

static WINDOW_DATA: ComponentClassMutex<WindowDataComponent> = ComponentClassMutex::new();

fn offset_from_root<Tag, Error>(mut window: Id<WindowData<Tag>>, tree: &WindowTree<Tag, Error>) -> Vector {
    let mut offset = Vector::null();
    loop {
        offset += tree.arena[window].bounds.tl.offset_from(Point { x: 0, y: 0 });
        if let Some(parent) = tree.arena[window].parent {
            window = parent;
        } else {
            break;
        }
    }
    offset
}

#[derive(Derivative)]
#[derivative(Debug(bound=""), Copy(bound=""), Clone(bound=""), Eq(bound=""), PartialEq(bound=""))]
#[derivative(Hash(bound=""), Ord(bound=""), PartialOrd(bound=""))]
pub struct Window<Tag>(Id<WindowData<Tag>>);

impl<Tag> Window<Tag> {
    pub fn new<Error>(
        tree: &mut WindowTree<Tag, Error>,
        parent: Option<Self>,
        bounds: Rect,
        tag: Tag
    ) -> Self {
        let parent = parent.map_or(tree.root, |w| w.0);
        let window = tree.arena.push(|this| WindowData {
            parent: Some(parent),
            next: this,
            last_child: None,
            bounds,
            tag
        });
        if let Some(prev) = tree.arena[parent].last_child.replace(window) {
            let next = replace(&mut tree.arena[prev].next, window);
            tree.arena[window].next = next;
        }
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
        Window(window)
    }

    pub fn move_<Error>(self, tree: &mut WindowTree<Tag, Error>, bounds: Rect) {
        let parent = tree.arena[self.0].parent.unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
        let bounds = replace(&mut tree.arena[self.0].bounds, bounds);
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
    }

    pub fn drop<Error>(self, tree: &mut WindowTree<Tag, Error>) {
        let this = tree.arena.pop(self.0);
        let parent = this.parent.unwrap_or_else(|| unsafe { unreachable_unchecked() });
        if tree.arena[parent].last_child.unwrap_or_else(|| unsafe { unreachable_unchecked() }) == self.0 {
            tree.arena[parent].last_child = if this.next == self.0 {
                None
            } else {
                Some(this.next)
            };
        }
        if let Some(mut prev) = tree.arena[parent].last_child {
            loop {
                let next = tree.arena[prev].next;
                if next == self.0 { break; }
                prev = next;
            }
            tree.arena[prev].next = this.next;
        }
        let screen_bounds = this.bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
    }

    pub fn size<Error>(self, tree: &WindowTree<Tag, Error>) -> Vector {
        tree.arena[self.0].bounds.size
    }

    pub fn invalidate_rect<Error>(self, tree: &mut WindowTree<Tag, Error>, rect: Rect) {
        let bounds = tree.arena[self.0].bounds;
        let rect = rect.offset(bounds.tl.offset_from(Point { x: 0, y: 0 })).intersect(bounds);
        let parent = tree.arena[self.0].parent.unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let screen_rect = rect.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_rect);
    }
 
    pub fn invalidate<Error>(self, tree: &mut WindowTree<Tag, Error>) {
        let bounds = tree.arena[self.0].bounds;
        let parent = tree.arena[self.0].parent.unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
    }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct WindowTree<Tag, Error> {
    #[derivative(Debug="ignore")]
    screen: Option<(Box<dyn Screen<Error=Error>>, Vec<Range<i16>>)>,
    arena: Arena<WindowData<Tag>>,
    root: Id<WindowData<Tag>>,
    #[derivative(Debug="ignore")]
    draw: fn(
        tree: &WindowTree<Tag, Error>,
        window: Option<Window<Tag>>,
        port: &mut DrawingPort<Error>,
        tag: &Tag
    ),
    cursor: Option<Point>,
    screen_size: Vector,
}

impl<Tag, Error> WindowTree<Tag, Error> {
    pub fn new(
        screen: Box<dyn Screen<Error=Error>>,
        draw: fn(
            tree: &WindowTree<Tag, Error>,
            window: Option<Window<Tag>>,
            port: &mut DrawingPort<Error>,
            tag: &Tag
        ),
        tag: Tag
    ) -> Self {
        let mut arena = Arena::new(&mut WINDOW_DATA.lock().unwrap());
        let root = arena.push(|this| WindowData {
            parent: None,
            next: this,
            last_child: None,
            bounds: Rect { tl: Point { x: 0, y: 0 }, size: screen.size() },
            tag
        });
        let screen_size = screen.size();
        let rows = screen_size.y as u16 as usize;
        let cols = screen_size.x;
        WindowTree { screen: Some((screen, vec![0 .. cols; rows])), arena, root, draw, cursor: None, screen_size }
    }

    pub fn screen_size(&self) -> Vector { self.screen_size }

    fn invalidated(&mut self) -> (&mut Vec<Range<i16>>, Vector) {
        let (screen, invalidated) = self.screen.as_mut().unwrap_or_else(|| unsafe { unreachable_unchecked() });
        (invalidated, screen.size())
    }

    fn draw_window(&mut self, window: Id<WindowData<Tag>>, offset: Vector) {
        let bounds = self.arena[window].bounds.offset(offset);
        let (invalidated, screen_size) = self.invalidated();
        if !rect_invalidated((invalidated, screen_size), bounds) { return; }
        let offset = offset + bounds.tl.offset_from(Point { x: 0, y: 0 });
        let (screen, invalidated) = self.screen.take().unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let mut port = DrawingPort {
            screen,
            invalidated,
            cursor: self.cursor,
            offset,
            size: bounds.size,
        };
        (self.draw)(self, if window == self.root { None } else { Some(Window(window)) }, &mut port, &self.arena[window].tag);
        self.screen.replace((port.screen, port.invalidated));
        self.cursor = port.cursor;
        if let Some(last_child) = self.arena[window].last_child {
            let mut child = last_child;
            loop {
                child = self.arena[child].next;
                self.draw_window(child, offset);
                if child == last_child { break; }
            }
        }
    }

    pub fn update(&mut self, wait: bool) -> Result<Option<Event>, Error> {
        if let Some(cursor) = self.cursor {
            let (invalidated, screen_size) = self.invalidated();
            if rect_invalidated((invalidated, screen_size), Rect { tl: cursor, size: Vector { x: 1, y: 1 } }) {
                self.cursor = None;
            }
        }
        self.draw_window(self.root, Vector::null());
        let (screen, invalidated) = self.screen.as_mut().unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let event = screen.update(self.cursor, wait)?;
        if event == Some(Event::Resize) {
            invalidated.clear();
            invalidated.resize(screen.size().y as u16 as usize, 0 .. screen.size(). x);
            self.screen_size = screen.size();
        }
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn window_tree_new_window() {
        fn draw(_: &WindowTree<u8, !>, _: Option<Window<u8>>, _: &mut DrawingPort, _: &u8) -> Result<(), !> {
            Ok(())
        }
        let screen = tuifw_screen_test::Screen::new(Vector::null());
        let screen = Box::new(screen) as _;
        let tree = &mut WindowTree::new(screen, draw, 0u8);
        assert!(tree.arena[tree.root].last_child.is_none());
        let one = Window::new(tree, None, true, 1);
        assert!(tree.arena[one.0].last_child.is_none());
        assert_eq!(tree.arena[one.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].last_child, Some(one.0));
        assert_eq!(tree.arena[one.0].next, one.0);
        let two = Window::new(tree, None, true, 2);
        assert!(tree.arena[two.0].last_child.is_none());
        assert_eq!(tree.arena[two.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].last_child, Some(two.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, one.0);
        let three = Window::new(tree, None, true, 2);
        assert!(tree.arena[three.0].last_child.is_none());
        assert_eq!(tree.arena[three.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].last_child, Some(three.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, three.0);
        assert_eq!(tree.arena[three.0].next, one.0);
        let four = Window::new(tree, None, true, 2);
        assert!(tree.arena[four.0].last_child.is_none());
        assert_eq!(tree.arena[four.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].last_child, Some(four.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, three.0);
        assert_eq!(tree.arena[three.0].next, four.0);
        assert_eq!(tree.arena[four.0].next, one.0);
    }
}
