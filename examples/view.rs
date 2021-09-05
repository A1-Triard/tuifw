#![deny(warnings)]
use dep_obj::binding::{Bindings, EventBinding1};
use dyn_context::state::{State, StateExt, StateRefMut};
use either::Right;
use std::borrow::Cow;
use tuifw::{Vector, Thickness, HAlign, VAlign, Point, Side, Rect, Key, Color};
use tuifw::view::{View, ViewBase, ViewTree, ViewBuilderViewAlignExt, ViewInput, ViewBuilderViewBaseExt};
use tuifw::view::panels::{CanvasLayout, ViewBuilderCanvasPanelExt};
use tuifw::view::panels::{ViewBuilderDockPanelExt};
use tuifw::view::decorators::{ViewBuilderBorderDecoratorExt};
use tuifw::view::decorators::{ViewBuilderLabelDecoratorExt};

fn build(state: &mut dyn State, bounds: Rect) -> View {
    let tree: &ViewTree = state.get();
    let root = tree.root();
    let mut border = None;
    root.build(state, |view| view
        .base(|base| base
            .fg(Color::Green)
        )
        .canvas_panel(|panel| panel
            .child(Some(&mut border), (), |layout| layout.tl(bounds.tl), |view| view
                .align(|align| align
                    .w(Some(bounds.w()))
                    .h(Some(bounds.h()))
                )
                .border_decorator(|view| view
                    .tl(Cow::Borrowed("╔"))
                    .tr(Cow::Borrowed("╗"))
                    .bl(Cow::Borrowed("╚"))
                    .br(Cow::Borrowed("╝"))
                    .l(Cow::Borrowed("║"))
                    .t(Cow::Borrowed("═"))
                    .r(Cow::Borrowed("║"))
                    .b(Cow::Borrowed("═"))
                )
                .dock_panel(|panel| panel
                    .child(None, (), |layout| layout.dock(Right(Side::Top)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed("↑")))
                    )
                    .child(None, (), |layout| layout.dock(Right(Side::Top)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed("k")))
                    )
                    .child(None, (), |layout| layout.dock(Right(Side::Bottom)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed("↓")))
                    )
                    .child(None, (), |layout| layout.dock(Right(Side::Bottom)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed("j")))
                    )
                    .child(None, (), |layout| layout.dock(Right(Side::Left)), |view| view
                        .align(|align| align.margin(Thickness::new(1, 0, 0, 0)))
                        .label_decorator(|label| label.text(Cow::Borrowed("←")))
                    )
                    .child(None, (), |layout| layout.dock(Right(Side::Left)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed("h")))
                    )
                    .child(None, (), |layout| layout.dock(Right(Side::Right)), |view| view
                        .align(|align| align.margin(Thickness::new(0, 0, 1, 0)))
                        .label_decorator(|label| label.text(Cow::Borrowed("→")))
                    )
                    .child(None, (), |layout| layout.dock(Right(Side::Right)), |view| view
                        .label_decorator(|label| label.text(Cow::Borrowed("l")))
                    )
                )
            )
        )
    );
    border.unwrap()
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let padding = Thickness::align(Vector { x: 13, y: 7 }, screen.size(), HAlign::Center, VAlign::Center);
    let bounds = padding.shrink_rect(Rect { tl: Point { x: 0, y: 0 }, size: screen.size() });
    let bindings = &mut Bindings::new();
    let tree = &mut ViewTree::new(screen, bindings, |_| ((), |tree| tree));
    tree.merge_mut_and_then(|state| {
        let border = build(state, bounds);
        let input_binding = EventBinding1::new(state, border, |
            state,
            border,
            (_, tl): (Point, Point),
            input: Option<&mut ViewInput>
        | input.map(|input| {
            let d = match input.key() {
                (n, Key::Left) | (n, Key::Char('h')) =>
                    -Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
                (n, Key::Right) | (n, Key::Char('l')) =>
                    Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
                (n, Key::Up) | (n, Key::Char('k')) =>
                    -Vector { x: 0, y: n.get() as i16 },
                (n, Key::Down) | (n, Key::Char('j')) =>
                    Vector { x: 0, y: n.get() as i16 },
                (_, Key::Escape) => { input.mark_as_handled(); ViewTree::quit(state); return; },
                _ => return,
            };
            input.mark_as_handled();
            CanvasLayout::TL.set_distinct(state, border.layout(), tl.offset(d));
        }));
        input_binding.set_event_source(state, &mut ViewBase::INPUT.source(border.base()));
        input_binding.set_source_1(state, &mut CanvasLayout::TL.source(border.layout()));
        border.base().add_binding(state, input_binding.into());

        border.focus(state);
        while ViewTree::update(state, true).unwrap() { }
        ViewTree::drop_self(state);
    }, bindings);
}
