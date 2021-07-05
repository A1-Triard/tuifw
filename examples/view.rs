#![deny(warnings)]
use dep_obj::Dispatcher;
use dyn_context::{State, StateExt, StateRefMut};
use either::Right;
use std::borrow::Cow;
use tuifw::{Key, Vector, Thickness, HAlign, VAlign, Point, Side, Rect};
use tuifw::view::{View, ViewTree, ViewBase, ViewBuilderViewAlignExt};
use tuifw::view::panels::{ViewBuilderCanvasPanelExt, CanvasLayout};
use tuifw::view::panels::{ViewBuilderDockPanelExt};
use tuifw::view::decorators::{ViewBuilderBorderDecoratorExt};
use tuifw::view::decorators::{ViewBuilderLabelDecoratorExt};

fn build(state: &mut dyn State, bounds: Rect) -> View {
    let tree: &ViewTree = state.get();
    let root = tree.root();
    let mut border = None;
    root.build(state, |view| view
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
    let tree = &mut ViewTree::new(screen, |_| ((), |tree| tree));
    let dispatcher = &mut Dispatcher::new();
    tree.merge_mut_and_then(|state| {
        let border = build(state, bounds);
        let tree: &mut ViewTree = state.get_mut();
        border.base(tree).on(ViewBase::INPUT, |state, border, input| {
            let tree: &mut ViewTree = state.get_mut();
            let d = match input.key() {
                (n, Key::Left) | (n, Key::Char('h')) =>
                    -Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
                (n, Key::Right) | (n, Key::Char('l')) =>
                    Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
                (n, Key::Up) | (n, Key::Char('k')) =>
                    -Vector { x: 0, y: n.get() as i16 },
                (n, Key::Down) | (n, Key::Char('j')) =>
                    Vector { x: 0, y: n.get() as i16 },
                (_, Key::Escape) => { input.mark_as_handled(); return tree.quit(); },
                _ => return,
            };
            input.mark_as_handled();
            let tl = border.layout_ref(tree).get(CanvasLayout::TL).offset(d);
            border.layout_mut(state).set_distinct(CanvasLayout::TL, tl);
        });
        border.focus(tree);
        while ViewTree::update(state, true).unwrap() {
            Dispatcher::dispatch(state);
        }
        while Dispatcher::dispatch(state) { }
    }, dispatcher);
}
