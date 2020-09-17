#![deny(warnings)]
use std::borrow::Cow;
use either::Right;
use dyn_context::ContextExt;
use tuifw::{Key, Vector, Thickness, HAlign, VAlign, Point, Side, Rect};
use tuifw::view::{View, ViewTree, view_base_type, ViewBuilderViewAlignExt};
use tuifw::view::panels::{ViewBuilderCanvasPanelExt, canvas_layout_type};
use tuifw::view::panels::{ViewBuilderDockPanelExt};
use tuifw::view::decorators::{ViewBuilderBorderDecoratorExt};
use tuifw::view::decorators::{ViewBuilderLabelDecoratorExt};

fn build(tree: &mut ViewTree, bounds: Rect) -> View {
    let mut border = None;
    tree.root().build(tree, |view| view
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
    let border = build(tree, bounds);
    border.base_on(tree, view_base_type().input(), |context, border, input| {
        let tree: &mut ViewTree = context.get_mut();
        let d = match input.key() {
            (n, Key::Left) | (n, Key::Char('h')) =>
                -Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
            (n, Key::Right) | (n, Key::Char('l')) =>
                Vector { x: (n.get() as i16).wrapping_mul(2), y: 0 },
            (n, Key::Up) | (n, Key::Char('k')) =>
                -Vector { x: 0, y: n.get() as i16 },
            (n, Key::Down) | (n, Key::Char('j')) =>
                Vector { x: 0, y: n.get() as i16 },
            (_, Key::Escape) => return tree.quit(),
            _ => return,
        };
        let tl = border.layout_get(tree, canvas_layout_type().tl()).offset(d);
        border.layout_set_distinct(tree, canvas_layout_type().tl(), tl);
    });
    border.focus(tree);
    while ViewTree::update(tree, true).unwrap() { }
}
