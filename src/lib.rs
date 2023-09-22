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
use tuifw_screen_base::{Bg, /*Event,*/ Fg, /*Key,*/ Point, Range1d, Rect, Vector};
use tuifw_window::{RenderPort, Window, WindowTree};
use unicode_width::UnicodeWidthChar;

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

pub trait Widget<Tag, State: ?Sized>: DynClone {
    fn render(
        &self,
        tree: &WindowTree<Tag, State>,
        window: Window<Tag>,
        port: &mut RenderPort,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    );

    fn measure(
        &self,
        tree: &mut WindowTree<Tag, State>,
        window: Window<Tag>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) -> Vector;

    fn arrange(
        &self,
        tree: &mut WindowTree<Tag, State>,
        window: Window<Tag>,
        final_inner_bounds: Rect,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) -> Vector;
}

clone_trait_object!(<Tag, State: ?Sized> Widget<Tag, State>);

pub struct Workspace<Tag, State: ?Sized> {
    window_tag_map_filter: fn(&Tag) -> Option<&(Box<dyn Widget<Tag, State>>, Box<dyn Any>)>,
    window_tag_map_filter_mut: fn(&mut Tag) -> Option<&mut (Box<dyn Widget<Tag, State>>, Box<dyn Any>)>,
}

impl<Tag, State: ?Sized> Workspace<Tag, State> {
    pub fn new(
        window_tag_map_filter: fn(&Tag) -> Option<&(Box<dyn Widget<Tag, State>>, Box<dyn Any>)>,
        window_tag_map_filter_mut: fn(&mut Tag) -> Option<&mut (Box<dyn Widget<Tag, State>>, Box<dyn Any>)>,
    ) -> Self {
        Workspace {
            window_tag_map_filter,
            window_tag_map_filter_mut,
        }
    }

    pub fn data<'a>(&self, tree: &'a WindowTree<Tag, State>, window: Window<Tag>) -> &'a dyn Any {
        (self.window_tag_map_filter)(window.tag(tree)).expect("unmanaged window").1.as_ref()
    }

    pub fn render(
        tree: &WindowTree<Tag, State>,
        window: Window<Tag>,
        port: &mut RenderPort,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) {
        let this = descriptor(state);
        let widget = (this.window_tag_map_filter)(window.tag(tree)).expect("unmanaged window").0.clone();
        widget.render(tree, window, port, state, descriptor)
    }

    pub fn measure(
        tree: &mut WindowTree<Tag, State>,
        window: Window<Tag>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) -> Vector {
        let this = descriptor(state);
        let widget = (this.window_tag_map_filter)(window.tag(tree)).expect("unmanaged window").0.clone();
        widget.measure(tree, window, available_width, available_height, state, descriptor)
    }

    pub fn arrange(
        tree: &mut WindowTree<Tag, State>,
        window: Window<Tag>,
        final_inner_bounds: Rect,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) -> Vector {
        let this = descriptor(state);
        let widget = (this.window_tag_map_filter)(window.tag(tree)).expect("unmanaged window").0.clone();
        widget.arrange(tree, window, final_inner_bounds, state, descriptor)
    }
}

pub struct StackPanel {
    vertical: bool,
}

#[derive(Clone)]
pub struct StackPanelWidget;

impl<Tag, State: ?Sized> Widget<Tag, State> for StackPanelWidget {
    fn render(
        &self,
        _tree: &WindowTree<Tag, State>,
        _window: Window<Tag>,
        _port: &mut RenderPort,
        _state: &mut State,
        _descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) { }

    fn measure(
        &self,
        tree: &mut WindowTree<Tag, State>,
        window: Window<Tag>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) -> Vector {
        let workspace = descriptor(state);
        let vertical = workspace.data(tree, window).downcast_ref::<StackPanel>().expect("StackPanel").vertical;
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
        tree: &mut WindowTree<Tag, State>,
        window: Window<Tag>,
        final_inner_bounds: Rect,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) -> Vector {
        let workspace = descriptor(state);
        let vertical = workspace.data(tree, window).downcast_ref::<StackPanel>().expect("StackPanel").vertical;
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
    color: (Fg, Bg),
    text: String,
}

#[derive(Clone)]
pub struct StaticTextWidget;

impl<Tag, State: ?Sized> Widget<Tag, State> for StaticTextWidget {
    fn render(
        &self,
        tree: &WindowTree<Tag, State>,
        window: Window<Tag>,
        port: &mut RenderPort,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) {
        let workspace = descriptor(state);
        let data = workspace.data(tree, window).downcast_ref::<StaticText>().expect("StackPanel");
        port.out(Point { x: 0, y: 0 }, data.color.0, data.color.1, &data.text);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<Tag, State>,
        window: Window<Tag>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) -> Vector {
        let workspace = descriptor(state);
        let data = workspace.data(tree, window).downcast_ref::<StaticText>().expect("StackPanel");
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<Tag, State>,
        window: Window<Tag>,
        final_inner_bounds: Rect,
        state: &mut State,
        descriptor: fn(&mut State) -> &mut Workspace<Tag, State>,
    ) -> Vector {
        let workspace = descriptor(state);
        let data = workspace.data(tree, window).downcast_ref::<StaticText>().expect("StackPanel");
        let width = data.text
            .chars()
            .filter_map(|c| if c == '\0' { None } else { c.width() })
            .fold(0i16, |s, c| s.wrapping_add(i16::try_from(c).unwrap()))
        ;
        Vector { x: width, y: 1 }
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
