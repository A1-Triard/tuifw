use tuifw::*;
use dep_obj::binding::{Bindings, b_immediate};
use dyn_context::state::State;
use std::any::{TypeId, Any};

struct App {
    bindings: Bindings,
    widgets: WidgetTree,
}

impl State for App {
    fn get_raw(&self, ty: TypeId) -> Option<&dyn Any> {
        if let Some(res) = self.bindings.get_raw(ty) { return Some(res); }
        if let Some(res) = self.widgets.get_raw(ty) { return Some(res); }
        None
    }

    fn get_mut_raw(&mut self, ty: TypeId) -> Option<&mut dyn Any> {
        if let Some(res) = self.bindings.get_mut_raw(ty) { return Some(res); }
        if let Some(res) = self.widgets.get_mut_raw(ty) { return Some(res); }
        None
    }
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let mut bindings = Bindings::new();
    let widgets = WidgetTree::new(screen, &mut bindings);
    let root = widgets.root();
    let app = &mut App { bindings, widgets };
    let desk_top = DeskTop::new(app);
    b_immediate(desk_top.load(app, root, |_, _| { }));
    let window = Window::new(app);
    b_immediate(Window::BOUNDS.set(app, window.obj(), Rect::from_tl_br(Point { x: 5, y: 5}, Point { x: 25, y: 15 })));
    b_immediate(DeskTop::WINDOWS.push(app, desk_top.obj(), window));
    let window = Window::new(app);
    b_immediate(Window::BOUNDS.set(app, window.obj(), Rect::from_tl_br(Point { x: 42, y: 5}, Point { x: 62, y: 15 })));
    b_immediate(DeskTop::WINDOWS.push(app, desk_top.obj(), window));
    let window = Window::new(app);
    b_immediate(Window::BOUNDS.set(app, window.obj(), Rect::from_tl_br(Point { x: 79, y: 5}, Point { x: 99, y: 15 })));
    b_immediate(DeskTop::WINDOWS.push(app, desk_top.obj(), window));
    window.focus(app);
    while WidgetTree::update(app, true).unwrap() { }
    WidgetTree::drop_self(app);
}
