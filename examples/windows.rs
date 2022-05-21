#![feature(explicit_generic_args_with_impl_trait)]

#![deny(warnings)]

use tuifw::*;
use dep_obj::DepObjId;
use dep_obj::binding::{Binding1, Bindings};
use dyn_context::{State, Stop};
use std::borrow::Cow;
use tuifw::view::ViewInput;
use dep_obj::{DepVecItemPos, DepVecInsertPos};

#[derive(State, Stop)]
struct App {
    #[state(part)]
    bindings: Bindings,
    #[state]
    #[stop]
    widgets: WidgetTree,
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
    let desk_top = DeskTop::new(app).build(app, |desk_top| desk_top
        .desk_top(|desk_top| desk_top
            .window(Some(&mut window_1), |window| window
                .window(|window| window
                    .header(Cow::Borrowed("1"))
                    .bounds(Rect::from_tl_br(Point { x: 5, y: 0}, Point { x: 40, y: 15 }))
                    .content(None, |x| StaticText::new(x).build(x, |static_text| static_text
                        .static_text(|static_text| static_text
                            .text(Cow::Borrowed("First Window"))
                        )
                    ))
                )
            )
            .window(Some(&mut window_2), |window| window
                .window(|window| window
                    .header(Cow::Borrowed("2"))
                    .bounds(Rect::from_tl_br(Point { x: 30, y: 5}, Point { x: 62, y: 20 }))
                    .content(None, |x| StaticText::new(x).build(x, |static_text| static_text
                        .static_text(|static_text| static_text
                            .text(Cow::Borrowed("Second Window"))
                        )
                    ))
                )
            )
            .window(Some(&mut window_3), |window| window
                .window(|window| window
                    .header(Cow::Borrowed("3"))
                    .bounds(Rect::from_tl_br(Point { x: 20, y: 10}, Point { x: 50, y: 22 }))
                    .content(None, |x| StaticText::new(x).build(x, |static_text| static_text
                        .static_text(|static_text| static_text
                            .text(Cow::Borrowed("Third Window"))
                        )
                    ))
                )
            )
        )
    );
    desk_top.load(app, root, None, |_, _| { }).immediate();
    window_1.unwrap().focus(app);
    DeskTop::WINDOWS.move_(app, desk_top, DepVecItemPos::Item(window_1.unwrap()), DepVecInsertPos::AfterLastItem).immediate();

    let focus_1 = Binding1::new(app, (), |(), input: Option<ViewInput>|
        input.filter(|input| input.key().1 == Key::Alt('1'))
    );
    focus_1.set_target_fn(app, (window_1.unwrap(), desk_top), |app, (window, desk_top), input| {
        input.mark_as_handled();
        window.focus(app);
        DeskTop::WINDOWS.move_(app, desk_top, DepVecItemPos::Item(window), DepVecInsertPos::AfterLastItem).immediate();
    });
    desk_top.add_binding::<WidgetBase, _>(app, focus_1);
    focus_1.set_source_1(app, &mut WidgetBase::VIEW_INPUT.source(desk_top));

    let focus_2 = Binding1::new(app, (), |(), input: Option<ViewInput>|
        input.filter(|input| input.key().1 == Key::Alt('2'))
    );
    focus_2.set_target_fn(app, (window_2.unwrap(), desk_top), |app, (window, desk_top), input| {
        input.mark_as_handled();
        window.focus(app);
        DeskTop::WINDOWS.move_(app, desk_top, DepVecItemPos::Item(window), DepVecInsertPos::AfterLastItem).immediate();
    });
    desk_top.add_binding::<WidgetBase, _>(app, focus_2);
    focus_2.set_source_1(app, &mut WidgetBase::VIEW_INPUT.source(desk_top));

    let focus_3 = Binding1::new(app, (), |(), input: Option<ViewInput>|
        input.filter(|input| input.key().1 == Key::Alt('3'))
    );
    focus_3.set_target_fn(app, (window_3.unwrap(), desk_top), |app, (window, desk_top), input| {
        input.mark_as_handled();
        window.focus(app);
        DeskTop::WINDOWS.move_(app, desk_top, DepVecItemPos::Item(window), DepVecInsertPos::AfterLastItem).immediate();
    });
    desk_top.add_binding::<WidgetBase, _>(app, focus_3);
    focus_3.set_source_1(app, &mut WidgetBase::VIEW_INPUT.source(desk_top));

    let quit = Binding1::new(app, (), |(), input: Option<ViewInput>|
        input.filter(|input| input.key().1 == Key::Escape)
    );
    quit.set_target_fn(app, (), |app, (), input| {
        input.mark_as_handled();
        WidgetTree::quit(app);
    });
    desk_top.add_binding::<WidgetBase, _>(app, quit);
    quit.set_source_1(app, &mut WidgetBase::VIEW_INPUT.source(desk_top));
    while WidgetTree::update(app, true).unwrap() { }
    App::stop(app);
}
