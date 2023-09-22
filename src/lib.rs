#![feature(effects)]

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
use alloc::string::String;
//use alloc::vec::Vec;
//use components_arena::{Arena, Id, Component};
use core::any::Any;
//use core::fmt::Debug;
//use core::ops::Range;
//use downcast_rs::{Downcast, impl_downcast};
use dyn_clone::{DynClone, clone_trait_object};
//use macro_attr_2018::macro_attr;
//use phantom_type::PhantomType;
use tuifw_screen_base::{Bg, Error, /*Event,*/ Fg, /*Key,*/ Point, Range1d, Rect, Screen, Vector};
use tuifw_window::{RenderPort, Window, WindowTree};
use unicode_width::UnicodeWidthChar;

pub trait RenderPortExt {
    fn fill_bg(&mut self, bg: Bg, fg: Option<Fg>);
    fn h_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg);
    fn v_line(&mut self, start: Point, len: i16, double: bool, fg: Fg, bg: Bg);
    fn tl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn tr_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn bl_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
    fn br_edge(&mut self, p: Point, double: bool, fg: Fg, bg: Bg);
}

impl RenderPortExt for RenderPort {
    fn fill_bg(&mut self, bg: Bg, fg: Option<Fg>) {
        self.fill(|rp, p| rp.out(p, fg.unwrap_or(Fg::LightGray), bg, if fg.is_some() { "░" } else { " " }));
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

pub type WidgetTag<State> = (Box<dyn Widget<State>>, Box<dyn Any>);

pub trait Widget<State: ?Sized>: DynClone {
    fn render(
        &self,
        tree: &WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        port: &mut RenderPort,
        state: &mut State,
    );

    fn measure(
        &self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector;

    fn arrange(
        &self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector;
}

clone_trait_object!(<State: ?Sized> Widget<State>);

pub fn widget_render<State: ?Sized>(
    tree: &WindowTree<WidgetTag<State>, State>,
    window: Window<WidgetTag<State>>,
    port: &mut RenderPort,
    state: &mut State,
) {
    let widget = window.tag(tree).0.clone();
    widget.render(tree, window, port, state)
}

pub fn widget_measure<State: ?Sized>(
    tree: &mut WindowTree<WidgetTag<State>, State>,
    window: Window<WidgetTag<State>>,
    available_width: Option<i16>,
    available_height: Option<i16>,
    state: &mut State,
) -> Vector {
    let widget = window.tag(tree).0.clone();
    widget.measure(tree, window, available_width, available_height, state)
}

pub fn widget_arrange<State: ?Sized>(
    tree: &mut WindowTree<WidgetTag<State>, State>,
    window: Window<WidgetTag<State>>,
    final_inner_bounds: Rect,
    state: &mut State,
) -> Vector {
    let widget = window.tag(tree).0.clone();
    widget.arrange(tree, window, final_inner_bounds, state)
}

pub struct StackPanel {
    pub vertical: bool,
}

impl StackPanel {
    pub fn widget_tag<State: ?Sized>(self) -> WidgetTag<State> {
        (Box::new(StackPanelWidget), Box::new(self))
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        parent: Window<WidgetTag<State>>,
        prev: Option<Window<WidgetTag<State>>>
    ) -> Result<Window<WidgetTag<State>>, Error> {
        Window::new(tree, self.widget_tag(), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<WidgetTag<State>, State>, Error> {
        WindowTree::new(screen, widget_render, widget_measure, widget_arrange, self.widget_tag())
    }
}

#[derive(Clone)]
pub struct StackPanelWidget;

impl<State: ?Sized> Widget<State> for StackPanelWidget {
    fn render(
        &self,
        _tree: &WindowTree<WidgetTag<State>, State>,
        _window: Window<WidgetTag<State>>,
        _port: &mut RenderPort,
        _state: &mut State,
    ) { }

    fn measure(
        &self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        let vertical = window.tag(tree).1.downcast_ref::<StackPanel>().expect("StackPanel").vertical;
        if vertical {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, available_width, None, state);
                    size += Vector { x: 0, y: child.desired_size(tree).y };
                    size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        } else {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, None, available_height, state);
                    size += Vector { x: child.desired_size(tree).x, y: 0 };
                    size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        let vertical = window.tag(tree).1.downcast_ref::<StackPanel>().expect("StackPanel").vertical;
        if vertical {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_desired_size = child.desired_size(tree);
                    child.arrange(tree, Rect { tl: pos, size: child_desired_size }, state);
                    pos = pos.offset(Vector { x: 0, y: child_desired_size.y });
                    size += Vector { x: 0, y: child.desired_size(tree).y };
                    size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        } else {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_desired_size = child.desired_size(tree);
                    child.arrange(tree, Rect { tl: pos, size: child_desired_size }, state);
                    pos = pos.offset(Vector { x: child_desired_size.x, y: 0 });
                    size += Vector { x: child.desired_size(tree).x, y: 0 };
                    size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        }
    }
}

pub struct StaticText {
    pub color: (Fg, Bg),
    pub text: String,
}

impl StaticText {
    pub fn widget_tag<State: ?Sized>(self) -> WidgetTag<State> {
        (Box::new(StaticTextWidget), Box::new(self))
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        parent: Window<WidgetTag<State>>,
        prev: Option<Window<WidgetTag<State>>>
    ) -> Result<Window<WidgetTag<State>>, Error> {
        Window::new(tree, self.widget_tag(), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<WidgetTag<State>, State>, Error> {
        WindowTree::new(screen, widget_render, widget_measure, widget_arrange, self.widget_tag())
    }
}

#[derive(Clone)]
pub struct StaticTextWidget;

impl<State: ?Sized> Widget<State> for StaticTextWidget {
    fn render(
        &self,
        tree: &WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        port: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.tag(tree).1.downcast_ref::<StaticText>().expect("StaticText");
        port.out(Point { x: 0, y: 0 }, data.color.0, data.color.1, &data.text);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.tag(tree).1.downcast_ref::<StaticText>().expect("StaticText");
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        _final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let data = window.tag(tree).1.downcast_ref::<StaticText>().expect("StaticText");
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }
}

pub struct Background {
    pub bg: Bg,
    pub fg: Option<Fg>,
}

impl Background {
    pub fn widget_tag<State: ?Sized>(self) -> WidgetTag<State> {
        (Box::new(BackgroundWidget), Box::new(self))
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        parent: Window<WidgetTag<State>>,
        prev: Option<Window<WidgetTag<State>>>
    ) -> Result<Window<WidgetTag<State>>, Error> {
        Window::new(tree, self.widget_tag(), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<WidgetTag<State>, State>, Error> {
        WindowTree::new(screen, widget_render, widget_measure, widget_arrange, self.widget_tag())
    }
}

#[derive(Clone)]
pub struct BackgroundWidget;

impl<State: ?Sized> Widget<State> for BackgroundWidget {
    fn render(
        &self,
        tree: &WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        port: &mut RenderPort,
        _state: &mut State,
    ) {
        let data = window.tag(tree).1.downcast_ref::<Background>().expect("Background");
        port.fill_bg(data.bg, data.fg);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        let mut size = Vector::null();
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.measure(tree, available_width, available_height, state);
                size = size.max(child.desired_size(tree));
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        size
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<WidgetTag<State>, State>,
        window: Window<WidgetTag<State>>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        if let Some(first_child) = window.first_child(tree) {
            let mut child = first_child;
            loop {
                child.arrange(tree, final_inner_bounds, state);
                child = child.next(tree);
                if child == first_child { break; }
            }
        }
        final_inner_bounds.size
    }
}

/*
pub struct StackPanel;

impl 


fn stack_panel_measure<Tag, State: ?Sized>(
    tree: &mut WindowTree<Tag, State>,
    window: Window<Tag>,
    available_width: Option<i16>,
    available_height: Option<i16>,
    workspace: &mut Workspace,
) -> Vector {
    if workspace.data::<StackPanelData>(window).vertical {
    } else {
    }
}



fn stack_panel_measure<Tag, State: ?Sized>(


pub struct Workspace {
    pub fn new_stack_panel()
}

impl View {
    pub fn new() -> Self {
        View { init: |_| { } }
    }
}

pub struct Group {
    pub view: View,
}

impl Group {
    pub fn new() -> Self {
        Group { view: View::new() }
    }
}

pub struct Application {
    pub group: Group,
    base_init: fn(&mut Self),
}

impl Application {
    pub fn new() -> Self {
    }
}
*/
