use tuifw::*;
use dep_obj::binding::{Bindings, b_immediate};
use dyn_context::state::State;
use std::any::{TypeId, Any};
use std::borrow::Cow;

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
    let mut window_3 = None;
    let desk_top = DeskTop::build(app, |desk_top| desk_top
        .window(None, |window| window
            .header(Cow::Borrowed("1"))
            .bounds(Rect::from_tl_br(Point { x: 5, y: 5}, Point { x: 25, y: 15 }))
        )
        .window(None, |window| window
            .header(Cow::Borrowed("2"))
            .bounds(Rect::from_tl_br(Point { x: 42, y: 5}, Point { x: 62, y: 15 }))
        )
        .window(Some(&mut window_3), |window| window
            .header(Cow::Borrowed("3"))
            .bounds(Rect::from_tl_br(Point { x: 79, y: 5}, Point { x: 99, y: 15 }))
        )
    );
    b_immediate(desk_top.load(app, root, |_, _| { }));
    window_3.unwrap().focus(app);
    while WidgetTree::update(app, true).unwrap() { }
    WidgetTree::drop_self(app);
}
