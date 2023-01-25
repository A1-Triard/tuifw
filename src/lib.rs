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
use alloc::vec::Vec;
use components_arena::{Arena, Id, Component};
use macro_attr_2018::macro_attr;
use tuifw_screen_base::{Bg, Event, Fg, Point, Range1d, Rect, Vector};
use tuifw_window::{RenderPort, Window, WindowTree};

pub trait RenderPortExt {
    fn fill_bg(&mut self, bg: Bg);
    fn h_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg);
    fn v_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg);
    fn tl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn tr_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn bl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn br_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
}

impl RenderPortExt for RenderPort {
    fn fill_bg(&mut self, bg: Bg) {
        self.fill(|rp, p| rp.out(p, Fg::LightGray, bg, " "));
    }

    fn h_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg) {
        let s = if double { "═" } else { "─" };
        for x in Range1d::new(start.x, start.x.wrapping_add(len)) {
            self.out(Point { x, y: start.y }, fg, bg, s);
        }
    }

    fn v_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg) {
        let s = if double { "║" } else { "│" };
        for y in Range1d::new(start.y, start.y.wrapping_add(len)) {
            self.out(Point { x: start.x, y }, fg, bg, s);
        }
    }

    fn tl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╔" } else { "┌" });
    }

    fn tr_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╗" } else { "┐" });
    }

    fn bl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╚" } else { "└" });
    }

    fn br_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg) {
        self.out(p, fg, bg, if double { "╝" } else { "┘" });
    }
}

pub trait WindowOwner<State: ?Sized> {
    fn move_xy(&mut self, tree: &mut WindowTree<State>, bounds: Rect);
}

struct WindowAsWindowOwner(Window);

impl<State: ?Sized> WindowOwner<State> for WindowAsWindowOwner {
    fn move_xy(&mut self, tree: &mut WindowTree<State>, bounds: Rect) {
        self.0.move_xy(tree, bounds);
    }
}

pub struct WindowManager<State: ?Sized> {
    windows: Vec<(Box<dyn WindowOwner<State>>, fn(Vector) -> Rect)>
}

impl<State: ?Sized> WindowManager<State> {
    pub fn new() -> Self {
        WindowManager { windows: Vec::new() }
    }

    pub fn update(&mut self, tree: &mut WindowTree<State>, event: Event) {
        if event == Event::Resize {
            let screen_size = tree.screen_size();
            for (window_owner, bounds) in &mut self.windows {
                window_owner.move_xy(tree, bounds(screen_size));
            }
        }
    }

    pub fn add_window_owner(
        &mut self,
        tree: &mut WindowTree<State>,
        mut window_owner: Box<dyn WindowOwner<State>>,
        bounds: fn(Vector) -> Rect
    ) {
        let initial_bounds = bounds(tree.screen_size());
        window_owner.move_xy(tree, initial_bounds);
        self.windows.push((window_owner, bounds));
    }

    pub fn add_window(
        &mut self,
        tree: &mut WindowTree<State>,
        window: Window,
        bounds: fn(Vector) -> Rect
    ) {
        self.add_window_owner(tree, Box::new(WindowAsWindowOwner(window)), bounds);
    }

    pub fn new_window(
        &mut self,
        tree: &mut WindowTree<State>,
        parent: Option<Window>,
        prev: Option<Window>,
        bounds: fn(Vector) -> Rect
    ) -> Window {
        let window = Window::new(tree, parent, prev);
        self.add_window(tree, window, bounds);
        window
    }
}

impl<State: ?Sized> Default for WindowManager<State> {
    fn default() -> Self { Self::new() }
}

macro_attr! {
    #[derive(Component!(class=WindowRenderFnClass))]
    struct WindowRenderFn<State: ?Sized>(fn(&WindowTree<State>, Window, &mut RenderPort, &mut State));
}

pub struct WindowRenderer<State: WindowRendererState + ?Sized + 'static> {
    render_fns: Arena<WindowRenderFn<State>>,
}

pub trait WindowRendererState {
    fn window_renderer(&self) -> &WindowRenderer<Self>;
}

impl<State: WindowRendererState + ?Sized + 'static> WindowRenderer<State> {
    pub fn new() -> Self {
        WindowRenderer { render_fns: Arena::new() }
    }

    pub fn add_window(
        &mut self,
        window: Window,
        tree: &mut WindowTree<State>,
        render: fn(&WindowTree<State>, Window, &mut RenderPort, &mut State)
    ) {
        let tag = self.render_fns.insert(|tag| (WindowRenderFn(render), tag));
        window.set_tag(tree, tag);
    }

    pub fn render(
        tree: &WindowTree<State>,
        window: Option<Window>,
        rp: &mut RenderPort,
        state: &mut State,
    ) {
        if let Some(window) = window {
            let tag: Id<WindowRenderFn<State>> = window.tag(tree).unwrap();
            let render_fn = state.window_renderer().render_fns[tag].0;
            render_fn(tree, window, rp, state);
        } else {
            rp.fill(|rp, p| rp.out(p, Fg::LightGray, Bg::None, " "));
        }
    }
}

impl<State: WindowRendererState + ?Sized + 'static> Default for WindowRenderer<State> {
    fn default() -> Self { Self::new() }
}
