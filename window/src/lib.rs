#![feature(never_type)]

#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::blocks_in_if_conditions)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::type_complexity)]

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::cmp::{max, min};
use core::mem::replace;
use components_arena::{Arena, Component, ComponentId, Id, NewtypeComponentId, RawId};
use educe::Educe;
use macro_attr_2018::macro_attr;
use tuifw_screen_base::{Bg, Error, Event, Fg, Point, Rect, Screen, Vector};

fn invalidate_rect(screen: &mut dyn Screen, rect: Rect) {
    let rect = rect.intersect(Rect { tl: Point { x: 0, y: 0 }, size: screen.size() });
    if rect.is_empty() { return; }
    let l = rect.l();
    let r = rect.r();
    for y in rect.t() .. rect.b() {
        let row = screen.line_invalidated_range_mut(y);
        row.start = min(row.start, l);
        row.end = max(row.end, r);
    }
}

fn rect_invalidated(screen: &dyn Screen, rect: Rect) -> bool {
    let rect = rect.intersect(Rect { tl: Point { x: 0, y: 0 }, size: screen.size() });
    if rect.is_empty() { return false; }
    let l = rect.l();
    let r = rect.r();
    for y in rect.t() .. rect.b() {
        let row = screen.line_invalidated_range(y);
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
    offset: Vector,
    size: Vector,
    cursor: Option<Point>,
}

impl RenderPort {
    pub fn out(&mut self, p: Point, fg: Fg, bg: Bg, text: &str) {
        let screen_size = self.screen.size();
        if p.y as u16 >= self.size.y as u16 || self.size.x == 0 { return; }
        let p = p.offset(self.offset);
        if p.y < 0 || p.y >= self.screen.size().y { return; }
        let row = self.screen.line_invalidated_range(p.y).clone();
        if p.x >= row.end { return; }

        let window_start = Point { x: 0, y: 0 }.offset(self.offset).x;
        let window_end = Point { x: 0, y: 0 }.offset(self.size + self.offset).x;
        let chunks = if window_start <= window_end {
            if window_end <= 0 || window_start >= screen_size.x { return; }
            [max(0, window_start) .. min(screen_size.x, window_end), 0 .. 0]
        } else {
            if window_end > 0 && window_start < screen_size.x {
                [0 .. window_end, window_start .. screen_size.x]
            } else if window_end > 0 {
                [0 .. window_end, 0 .. 0]
            } else if window_start < screen_size.x {
                [window_start .. screen_size.x, 0 .. 0]
            } else {
                return
            }
        };

        for chunk in &chunks {
            if chunk.start >= chunk.end { continue; }
            let out = self.screen.out(p, fg, bg, text, chunk.clone(), row.clone());
            if out.start >= out.end { continue; }
            let row = self.screen.line_invalidated_range_mut(p.y);
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
        let row = &self.screen.line_invalidated_range(p.y);
        if p.x < row.start || p.x >= row.end { return; }
        self.cursor = Some(p);
    }

    pub fn fill(&mut self, mut f: impl FnMut(&mut Self, Point)) {
        for y in 0 .. self.screen.size().y {
            for x in self.screen.line_invalidated_range(y).clone() {
                f(self, Point { x, y }.offset(-self.offset));
            }
        }
    }
}

macro_attr! {
    #[derive(Component!)]
    #[derive(Debug)]
    struct WindowNode {
        parent: Option<Id<WindowNode>>,
        prev: Id<WindowNode>,
        next: Id<WindowNode>,
        first_child: Option<Id<WindowNode>>,
        bounds: Rect,
        tag: Option<RawId>,
    }
}

fn offset_from_root<State: ?Sized>(
    mut window: Id<WindowNode>,
    tree: &WindowTree<State>
) -> Vector {
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
    #[derive(NewtypeComponentId!)]
    #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
    pub struct Window(Id<WindowNode>);
}

impl Window {
    pub fn new<State: ?Sized>(
        tree: &mut WindowTree<State>,
        parent: Option<Self>,
        prev: Option<Self>,
    ) -> Result<Window, Error> {
        tree.arena.try_reserve().map_err(|_| Error::Oom)?;
        let parent = parent.map_or(tree.root, |w| w.0);
        let window = tree.arena.insert(|window| {
            (WindowNode {
                parent: Some(parent),
                prev: window,
                next: window,
                first_child: None,
                bounds: Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() },
                tag: None
            }, Window(window))
        });
        window.attach(tree, parent, prev);
        Ok(window)
    }

    pub fn bounds<State: ?Sized>(
        self,
        tree: &WindowTree<State>
    ) -> Rect {
        tree.arena[self.0].bounds
    }

    pub fn inner_bounds<State: ?Sized>(
        self,
        tree: &WindowTree<State>
    ) -> Rect {
        let bounds = self.bounds(tree);
        Rect { tl: Point { x: 0, y: 0 }, size: bounds.size }
    }

    pub fn set_tag<Tag: ComponentId, State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        tag: Tag
    ) {
        tree.arena[self.0].tag = Some(tag.into_raw());
    }

    pub fn reset_tag<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>
    ) {
        tree.arena[self.0].tag = None;
    }

    pub fn tag<Tag: ComponentId, State: ?Sized>(
        self,
        tree: &WindowTree<State>
    ) -> Option<Tag> {
        tree.arena[self.0].tag.map(Tag::from_raw)
    }

    pub fn parent<State: ?Sized>(
        self,
        tree: &WindowTree<State>
    ) -> Option<Window> {
        let parent = tree.arena[self.0].parent.unwrap();
        if parent == tree.root { None } else { Some(Window(parent)) }
    }

    pub fn first_child<State: ?Sized>(
        self,
        tree: &WindowTree<State>
    ) -> Option<Window> {
        tree.arena[self.0].first_child.map(Window)
    }

    pub fn prev<State: ?Sized>(
        self,
        tree: &WindowTree<State>
    ) -> Window {
        Window(tree.arena[self.0].prev)
    }

    pub fn next<State: ?Sized>(
        self,
        tree: &WindowTree<State>
    ) -> Window {
        Window(tree.arena[self.0].next)
    }

    pub fn move_xy<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        bounds: Rect
    ) {
        let parent = tree.arena[self.0].parent.unwrap();
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
        let bounds = replace(&mut tree.arena[self.0].bounds, bounds);
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }

    pub fn move_z<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        prev: Option<Window>
    ) {
        let parent = self.detach(tree);
        self.attach(tree, parent, prev);
        let bounds = tree.arena[self.0].bounds;
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }

    fn detach<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>
    ) -> Id<WindowNode> {
        let node = &mut tree.arena[self.0];
        let prev = replace(&mut node.prev, self.0);
        let next = replace(&mut node.next, self.0);
        let parent = node.parent.take().unwrap();
        tree.arena[prev].next = next;
        tree.arena[next].prev = prev;
        let parent_node = &mut tree.arena[parent];
        if parent_node.first_child.unwrap() == self.0 {
            parent_node.first_child = if next == self.0 { None } else { Some(next) };
        }
        parent
    }

    fn attach<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Id<WindowNode>,
        prev: Option<Window>
    ) {
        let (prev, next) = if let Some(prev) = prev {
            assert_eq!(tree.arena[prev.0].parent.unwrap(), parent);
            let prev = prev.0;
            let next = replace(&mut tree.arena[prev].next, self.0);
            tree.arena[next].prev = self.0;
            (prev, next)
        } else {
            let parent_node = &mut tree.arena[parent];
            let next = parent_node.first_child.replace(self.0).unwrap_or(self.0);
            let prev = replace(&mut tree.arena[next].prev, self.0);
            tree.arena[prev].next = self.0;
            (prev, next)
        };
        let node = &mut tree.arena[self.0];
        node.parent = Some(parent);
        node.prev = prev;
        node.next = next;
    }

    pub fn drop_window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>
    ) {
        let parent = self.detach(tree);
        let node = tree.arena.remove(self.0);
        let screen_bounds = node.bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
        Self::drop_node_tree(node, tree);
    }

    fn drop_node_tree<State: ?Sized>(
        node: WindowNode,
        tree: &mut WindowTree<State>
    ) {
        if let Some(first_child) = node.first_child {
            let mut child = first_child;
            loop {
                let child_node = tree.arena.remove(child);
                child = child_node.next;
                Self::drop_node_tree(child_node, tree);
                if child == first_child { break; }
            }
        }
    }

    pub fn invalidate_rect<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        rect: Rect
    ) {
        let bounds = tree.arena[self.0].bounds;
        let rect = rect.offset(bounds.tl.offset_from(Point { x: 0, y: 0 })).intersect(bounds);
        let parent = tree.arena[self.0].parent.unwrap();
        let screen_rect = rect.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_rect);
    }
 
    pub fn invalidate<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>
    ) {
        let bounds = tree.arena[self.0].bounds;
        let parent = tree.arena[self.0].parent.unwrap();
        let screen_bounds = bounds.offset(offset_from_root(parent, tree));
        invalidate_rect(tree.screen(), screen_bounds);
    }
}

#[derive(Educe)]
#[educe(Debug)]
pub struct WindowTree<State: ?Sized> {
    #[educe(Debug(ignore))]
    screen: Option<Box<dyn Screen>>,
    arena: Arena<WindowNode>,
    root: Id<WindowNode>,
    #[educe(Debug(ignore))]
    render: fn(
        tree: &WindowTree<State>,
        window: Option<Window>,
        port: &mut RenderPort,
        state: &mut State,
    ),
    cursor: Option<Point>,
    screen_size: Vector,
}

impl<State: ?Sized> WindowTree<State> {
    pub fn new(
        screen: Box<dyn Screen>,
        render: fn(
            tree: &WindowTree<State>,
            window: Option<Window>,
            port: &mut RenderPort,
            state: &mut State,
        )
    ) -> Result<Self, Error> {
        let mut arena = Arena::new();
        arena.try_reserve().map_err(|_| Error::Oom)?;
        let screen_size = screen.size();
        let root = arena.insert(|window| (WindowNode {
            parent: None,
            prev: window,
            next: window,
            first_child: None,
            bounds: Rect { tl: Point { x: 0, y: 0 }, size: screen_size },
            tag: None
        }, window));
        Ok(WindowTree { screen: Some(screen), arena, root, render, cursor: None, screen_size })
    }

    pub fn screen_size(&self) -> Vector { self.screen_size }

    pub fn invalidate_rect(&mut self, rect: Rect) {
        invalidate_rect(self.screen(), rect);
    }
 
    pub fn invalidate_screen(&mut self) {
        let size = self.screen_size;
        invalidate_rect(self.screen(), Rect { tl: Point { x: 0, y: 0 }, size });
    }

    fn screen(&mut self) -> &mut dyn Screen {
        self.screen.as_mut().expect("WindowTree is in invalid state").as_mut()
    }

    fn render_window(&mut self, window: Id<WindowNode>, offset: Vector, render_state: &mut State) {
        let bounds = self.arena[window].bounds.offset(offset);
        let screen = self.screen();
        if !rect_invalidated(screen, bounds) { return; }
        let offset = bounds.tl.offset_from(Point { x: 0, y: 0 });
        let screen = self.screen.take().expect("WindowTree is in invalid state");
        let mut port = RenderPort {
            screen,
            cursor: self.cursor,
            offset,
            size: bounds.size,
        };
        (self.render)(
            self,
            if window == self.root { None } else { Some(Window(window)) },
            &mut port,
            render_state
        );
        self.screen.replace(port.screen);
        self.cursor = port.cursor;
        if let Some(first_child) = self.arena[window].first_child {
            let mut child = first_child;
            loop {
                self.render_window(child, offset, render_state);
                child = self.arena[child].next;
                if child == first_child { break; }
            }
        }
    }

    pub fn update(&mut self, wait: bool, render_state: &mut State) -> Result<Option<Event>, Error> {
        if let Some(cursor) = self.cursor {
            let screen = self.screen();
            if rect_invalidated(screen, Rect { tl: cursor, size: Vector { x: 1, y: 1 } }) {
                self.cursor = None;
            }
        }
        self.render_window(self.root, Vector::null(), render_state);
        let screen = self.screen.as_mut().expect("WindowTree is in invalid state");
        let event = screen.update(self.cursor, wait)?;
        if event == Some(Event::Resize) {
            self.screen_size = screen.size();
            self.arena[self.root].bounds = Rect { tl: Point { x: 0, y: 0 }, size: self.screen_size };
        }
        Ok(event)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn window_tree_new_window() {
        fn render<State: ?Sized>(_: &WindowTree<State>, _: Option<Window>, _: &mut RenderPort, _: &mut State) { }
        let screen = tuifw_screen_test::Screen::new(Vector::null());
        let screen = Box::new(screen) as _;
        let tree = &mut WindowTree::<()>::new(screen, render).unwrap();
        assert!(tree.arena[tree.root].first_child.is_none());
        let one = Window::new(tree, None, None).unwrap();
        one.move_xy(tree, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() });
        assert!(tree.arena[one.0].first_child.is_none());
        assert_eq!(tree.arena[one.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].first_child, Some(one.0));
        assert_eq!(tree.arena[one.0].next, one.0);
        let two = Window::new(tree, None, Some(one)).unwrap();
        two.move_xy(tree, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() });
        assert!(tree.arena[two.0].first_child.is_none());
        assert_eq!(tree.arena[two.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].first_child, Some(one.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, one.0);
        let three = Window::new(tree, None, Some(two)).unwrap();
        three.move_xy(tree, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() });
        assert!(tree.arena[three.0].first_child.is_none());
        assert_eq!(tree.arena[three.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].first_child, Some(one.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, three.0);
        assert_eq!(tree.arena[three.0].next, one.0);
        let four = Window::new(tree, None, None).unwrap();
        four.move_xy(tree, Rect { tl: Point { x: 0, y: 0 }, size: Vector::null() });
        assert!(tree.arena[four.0].first_child.is_none());
        assert_eq!(tree.arena[four.0].parent, Some(tree.root));
        assert_eq!(tree.arena[tree.root].first_child, Some(four.0));
        assert_eq!(tree.arena[one.0].next, two.0);
        assert_eq!(tree.arena[two.0].next, three.0);
        assert_eq!(tree.arena[three.0].next, four.0);
        assert_eq!(tree.arena[four.0].next, one.0);
    }

    #[test]
    fn drop_subtree() {
        fn render<State: ?Sized>(_: &WindowTree<State>, _: Option<Window>, _: &mut RenderPort, _: &mut State) { }
        let screen = tuifw_screen_test::Screen::new(Vector::null());
        let screen = Box::new(screen) as _;
        let tree = &mut WindowTree::<()>::new(screen, render).unwrap();
        let w = Window::new(tree, None, None).unwrap();
        let _ = Window::new(tree, Some(w), None).unwrap();
        w.drop_window(tree);
        assert_eq!(tree.arena.items().len(), 1);
     }
}
