use tuifw::*;
use dep_obj::binding::{Binding1, Bindings, b_immediate};
use dyn_context::state::State;
use std::any::{TypeId, Any};
use std::borrow::Cow;
use tuifw::view::ViewInput;

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
    let mut window_1 = None;
    let mut window_2 = None;
    let mut window_3 = None;
    let desk_top = DeskTop::build(app, |desk_top| desk_top
        .window(Some(&mut window_1), |window| window
            .header(Cow::Borrowed("1"))
            .bounds(Rect::from_tl_br(Point { x: 5, y: 0}, Point { x: 40, y: 15 }))
        )
        .window(Some(&mut window_2), |window| window
            .header(Cow::Borrowed("2"))
            .bounds(Rect::from_tl_br(Point { x: 30, y: 5}, Point { x: 62, y: 20 }))
        )
        .window(Some(&mut window_3), |window| window
            .header(Cow::Borrowed("3"))
            .bounds(Rect::from_tl_br(Point { x: 20, y: 10}, Point { x: 50, y: 22 }))
        )
    );
    b_immediate(desk_top.load(app, root, None, |_, _| { }));
    window_1.unwrap().focus(app);

    let focus_1 = Binding1::new(app, (), |(), input: Option<ViewInput>|
        input.filter(|input| input.key().1 == Key::Alt('1'))
    );
    focus_1.set_target_fn(app, (window_1.unwrap(), desk_top), |app, (window, desk_top), input| {
        input.mark_as_handled();
        window.focus(app);
        b_immediate(DeskTop::WINDOWS.move_(app, desk_top.obj(), 0, 2));
    });
    desk_top.base().add_binding(app, focus_1);
    focus_1.set_source_1(app, &mut WidgetBase::VIEW_INPUT.source(desk_top.base()));

    let focus_2 = Binding1::new(app, (), |(), input: Option<ViewInput>|
        input.filter(|input| input.key().1 == Key::Alt('2'))
    );
    focus_2.set_target_fn(app, window_2.unwrap(), |app, window, input| {
        input.mark_as_handled();
        window.focus(app);
    });
    desk_top.base().add_binding(app, focus_2);
    focus_2.set_source_1(app, &mut WidgetBase::VIEW_INPUT.source(desk_top.base()));

    let focus_3 = Binding1::new(app, (), |(), input: Option<ViewInput>|
        input.filter(|input| input.key().1 == Key::Alt('3'))
    );
    focus_3.set_target_fn(app, window_3.unwrap(), |app, window, input| {
        input.mark_as_handled();
        window.focus(app);
    });
    desk_top.base().add_binding(app, focus_3);
    focus_3.set_source_1(app, &mut WidgetBase::VIEW_INPUT.source(desk_top.base()));

    let quit = Binding1::new(app, (), |(), input: Option<ViewInput>|
        input.filter(|input| input.key().1 == Key::Escape)
    );
    quit.set_target_fn(app, (), |app, (), input| {
        input.mark_as_handled();
        WidgetTree::quit(app);
    });
    desk_top.base().add_binding(app, quit);
    quit.set_source_1(app, &mut WidgetBase::VIEW_INPUT.source(desk_top.base()));
    while WidgetTree::update(app, true).unwrap() { }
    WidgetTree::drop_self(app);
}
