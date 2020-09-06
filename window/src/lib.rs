#![deny(warnings)]
#![feature(never_type)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]
#![allow(clippy::blocks_in_if_conditions)]

use std::any::Any;
use std::cmp::{min, max};
use std::hint::{unreachable_unchecked};
use std::marker::PhantomData;
use std::mem::replace;
use std::ops::Range;
use std::panic::{UnwindSafe, RefUnwindSafe};
use components_arena::{Component, Arena, Id, ComponentClassMutex, ComponentId};
use tuifw_screen_base::{Screen, Rect, Point, Vector, Attr, Color, Event};
use educe::Educe;
use macro_attr_2018::macro_attr;

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

#[derive(Educe)]
#[educe(Debug)]
pub struct RenderPort {
    #[educe(Debug(ignore))]
    screen: Box<dyn Screen>,
    invalidated: Vec<Range<i16>>,
    offset: Vector,
    size: Vector,
    cursor: Option<Point>,
}

impl RenderPort {
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

macro_attr! {
    #[derive(Component!(class=WindowNodeComponent))]
    #[derive(Educe)]
    #[educe(Debug)]
    struct WindowNode<Tag> {
        parent: Option<Id<WindowNode<Tag>>>,
        next: Id<WindowNode<Tag>>,
        last_child: Option<Id<WindowNode<Tag>>>,
        bounds: Rect,
        #[educe(Debug(ignore))]
        tag: Tag,
    }
}

static WINDOW_NODE: ComponentClassMutex<WindowNodeComponent> = ComponentClassMutex::new();

fn offset_from_root<Tag, RenderContext>(mut window: Id<WindowNode<Tag>>, tree: &WindowTree<Tag, RenderContext>) -> Vector {
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

macro_attr! {
    #[derive(ComponentId!)]
    #[derive(Educe)]
    #[educe(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Window<Tag, RenderContext>(Id<WindowNode<Tag>>, PhantomData<RenderContext>);
}

impl<Tag, RenderContext> RefUnwindSafe for Window<Tag, RenderContext> { }
unsafe impl<Tag, RenderContext> Send for Window<Tag, RenderContext> { }
unsafe impl<Tag, RenderContext> Sync for Window<Tag, RenderContext> { }
impl<Tag, RenderContext> Unpin for Window<Tag, RenderContext> { }
impl<Tag, RenderContext> UnwindSafe for Window<Tag, RenderContext> { }

impl<Tag, RenderContext> Window<Tag, RenderContext> {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<T>(
        tree: &mut WindowTree<Tag, RenderContext>,
        parent: Option<Self>,
        bounds: Rect,
        tag: impl FnOnce(Self) -> (Tag, T)
    ) -> T {
        let parent = parent.map_or(tree.root, |w| w.0);
        let (window, result) = tree.arena.insert(|window| {
            let (tag, result) = tag(Window(window, PhantomData));
            (WindowNode {
                parent: Some(parent),
                next: window,
                last_child: None,
                bounds,
                tag
            }, (window, result))
        });
        if let Some(prev) = tree.arena[parent].last_child.replace(window) {
            let next = replace(&mut tree.arena[prev].next, window);
            tree.arena[window].next = next;
        }
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
        result
    }

    pub fn move_(self, tree: &mut WindowTree<Tag, RenderContext>, bounds: Rect) {
        let parent = tree.arena[self.0].parent.unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
        let bounds = replace(&mut tree.arena[self.0].bounds, bounds);
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
    }

    pub fn drop(self, tree: &mut WindowTree<Tag, RenderContext>) {
        let node = tree.arena.remove(self.0);
        let parent = node.parent.unwrap_or_else(|| unsafe { unreachable_unchecked() });
        if tree.arena[parent].last_child.unwrap_or_else(|| unsafe { unreachable_unchecked() }) == self.0 {
            tree.arena[parent].last_child = if node.next == self.0 {
                None
            } else {
                Some(node.next)
            };
        }
        if let Some(mut prev) = tree.arena[parent].last_child {
            loop {
                let next = tree.arena[prev].next;
                if next == self.0 { break; }
                prev = next;
            }
            tree.arena[prev].next = node.next;
        }
        let screen_bounds = node.bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
    }

    pub fn invalidate_rect(self, tree: &mut WindowTree<Tag, RenderContext>, rect: Rect) {
        let bounds = tree.arena[self.0].bounds;
        let rect = rect.offset(bounds.tl.offset_from(Point { x: 0, y: 0 })).intersect(bounds);
        let parent = tree.arena[self.0].parent.unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let screen_rect = rect.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_rect);
    }
 
    pub fn invalidate(self, tree: &mut WindowTree<Tag, RenderContext>) {
        let bounds = tree.arena[self.0].bounds;
        let parent = tree.arena[self.0].parent.unwrap_or_else(|| unsafe { unreachable_unchecked() });
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.invalidated(), screen_bounds);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct WindowTree<Tag, RenderContext> {
    #[educe(Debug(ignore))]
    screen: Option<(Box<dyn Screen>, Vec<Range<i16>>)>,
    arena: Arena<WindowNode<Tag>>,
    root: Id<WindowNode<Tag>>,
    #[educe(Debug(ignore))]
    render: fn(
        tree: &WindowTree<Tag, RenderContext>,
        window: Option<Window<Tag, RenderContext>>,
        port: &mut RenderPort,
        tag: &Tag,
        context: &mut RenderContext,
    ),
    cursor: Option<Point>,
    screen_size: Vector,
}

impl<Tag, RenderContext> WindowTree<Tag, RenderContext> {
    pub fn new(
        screen: Box<dyn Screen>,
        render: fn(
            tree: &WindowTree<Tag, RenderContext>,
            window: Option<Window<Tag, RenderContext>>,
            port: &mut RenderPort,
            tag: &Tag,
            context: &mut RenderContext,
        ),
        tag: Tag
    ) -> Self {
        let mut arena = Arena::new(&mut WINDOW_NODE.lock().unwrap());
        let root = arena.insert(|window| (WindowNode {
            parent: None,
            next: window,
            last_child: None,
            bounds: Rect { tl: Point { x: 0, y: 0 }, size: screen.size() },
            tag
        }, window));
        let screen_size = screen.size();
        let rows = screen_size.y as u16 as usize;
        let cols = screen_size.x;
        WindowTree { screen: Some((screen, vec![0 .. cols; rows])), arena, root, render, cursor: None, screen_size }
    }

    pub fn screen_size(&self) -> Vector { self.screen_size }

    pub fn invalidate_rect(&mut self, rect: Rect) {
        invalidate_rect(self.invalidated(), rect);
    }
 
    pub fn invalidate_screen(&mut self) {
        let size = self.screen_size;
        invalidate_rect(self.invalidated(), Rect { tl: Point { x: 0, y: 0 }, size });
    }

    fn invalidated(&mut self) -> (&mut Vec<Range<i16>>, Vector) {
        let (screen, invalidated) = self.screen.as_mut().expect("WindowTree is in invalid state");
        (invalidated, screen.size())
    }

    fn render_window(&mut self, window: Id<WindowNode<Tag>>, offset: Vector, render_context: &mut RenderContext) {
        let bounds = self.arena[window].bounds.offset(offset);
        let (invalidated, screen_size) = self.invalidated();
        if !rect_invalidated((invalidated, screen_size), bounds) { return; }
        let offset = offset + bounds.tl.offset_from(Point { x: 0, y: 0 });
        let (screen, invalidated) = self.screen.take().expect("WindowTree is in invalid state");
        let mut port = RenderPort {
            screen,
            invalidated,
            cursor: self.cursor,
            offset,
            size: bounds.size,
        };
        (self.render)(
            self,
            if window == self.root { None } else { Some(Window(window, PhantomData)) },
            &mut port,
            &self.arena[window].tag,
            render_context
        );
        self.screen.replace((port.screen, port.invalidated));
        self.cursor = port.cursor;
        if let Some(last_child) = self.arena[window].last_child {
            let mut child = last_child;
            loop {
                child = self.arena[child].next;
                self.render_window(child, offset, render_context);
                if child == last_child { break; }
            }
        }
    }

    pub fn update(&mut self, wait: bool, render_context: &mut RenderContext) -> Result<Option<Event>, Box<dyn Any>> {
        if let Some(cursor) = self.cursor {
            let (invalidated, screen_size) = self.invalidated();
            if rect_invalidated((invalidated, screen_size), Rect { tl: cursor, size: Vector { x: 1, y: 1 } }) {
                self.cursor = None;
            }
        }
        self.render_window(self.root, Vector::null(), render_context);
        let (screen, invalidated) = self.screen.as_mut().expect("WindowTree is in invalid state");
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
        fn render(_: &WindowTree<u8, ()>, _: Option<Window<u8, ()>>, _: &mut RenderPort, _: &u8, _: &mut ()) { }
        let screen = tuifw_screen_test::Screen::new(Vector::null());
        let screen = Box::new(screen) as _;
        let tree = &mut WindowTree::new(screen, render, 0u8);
        assert!(tree.arena[tree.root].last_child.is_none());
        let one = Window::new(tree, None, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }, |id| (1, id));
        assert!(tree.arena[one.0].last_child.is_none());
        assert_eq!(tree.arena[one.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].last_child, Some(one.0));
        assert_eq!(tree.arena[one.0].next, one.0);
        let two = Window::new(tree, None, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }, |id| (2, id));
        assert!(tree.arena[two.0].last_child.is_none());
        assert_eq!(tree.arena[two.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].last_child, Some(two.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, one.0);
        let three = Window::new(tree, None, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }, |id| (2, id));
        assert!(tree.arena[three.0].last_child.is_none());
        assert_eq!(tree.arena[three.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].last_child, Some(three.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, three.0);
        assert_eq!(tree.arena[three.0].next, one.0);
        let four = Window::new(tree, None, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() }, |id| (2, id));
        assert!(tree.arena[four.0].last_child.is_none());
        assert_eq!(tree.arena[four.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].last_child, Some(four.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, three.0);
        assert_eq!(tree.arena[three.0].next, four.0);
        assert_eq!(tree.arena[four.0].next, one.0);
    }
}
