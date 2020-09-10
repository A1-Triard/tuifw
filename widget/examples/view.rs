#![deny(warnings)]
use std::borrow::Cow;
use dyn_context::ContextExt;
use tuifw_screen::{Key, Vector, Thickness, HAlign, VAlign, Point, Side, Rect};
use tuifw_widget::view::{ViewTree, View, view_align_type, view_base_type};
use tuifw_widget::view::panels::{CanvasPanel, CanvasLayout, canvas_layout_type};
use tuifw_widget::view::panels::{DockPanel, DockLayout, dock_layout_type};
use tuifw_widget::view::decorators::{BorderDecorator, border_decorator_type};
use tuifw_widget::view::decorators::{LabelDecorator, label_decorator_type};

fn double_border(tree: &mut ViewTree, view: View) {
    view.decorator_set_distinct(tree, border_decorator_type().tl(), Cow::Borrowed("╔"));
    view.decorator_set_uncond(tree, border_decorator_type().tr(), Cow::Borrowed("╗"));
    view.decorator_set_uncond(tree, border_decorator_type().bl(), Cow::Borrowed("╚"));
    view.decorator_set_uncond(tree, border_decorator_type().br(), Cow::Borrowed("╝"));
    view.decorator_set_uncond(tree, border_decorator_type().l(), Cow::Borrowed("║"));
    view.decorator_set_uncond(tree, border_decorator_type().t(), Cow::Borrowed("═"));
    view.decorator_set_uncond(tree, border_decorator_type().r(), Cow::Borrowed("║"));
    view.decorator_set_uncond(tree, border_decorator_type().b(), Cow::Borrowed("═"));
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let padding = Thickness::align(Vector { x: 13, y: 7 }, screen.size(), HAlign::Center, VAlign::Center);
    let bounds = padding.shrink_rect(Rect { tl: Point { x: 0, y: 0 }, size: screen.size() });
    let tree = &mut ViewTree::new(screen, |_| ((), |tree| tree));
    CanvasPanel::new(tree, tree.root());
    let border = View::new(tree, tree.root(), |view| ((), view));
    CanvasLayout::new(tree, border);
    BorderDecorator::new(tree, border);
    double_border(tree, border);
    border.align_set_distinct(tree, view_align_type().w(), Some(bounds.w()));
    border.align_set_distinct(tree, view_align_type().h(), Some(bounds.h()));
    border.layout_set_distinct(tree, canvas_layout_type().tl(), bounds.tl);
    DockPanel::new(tree, border);
    let l_arrow = View::new(tree, border, |view| ((), view));
    DockLayout::new(tree, l_arrow);
    LabelDecorator::new(tree, l_arrow);
    l_arrow.decorator_set_distinct(tree, label_decorator_type().text(), Cow::Borrowed("←"));
    l_arrow.layout_set_distinct(tree, dock_layout_type().dock(), Some(Side::Left));
    border.base_on(tree, view_base_type().input(), |border, context, input| {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
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
