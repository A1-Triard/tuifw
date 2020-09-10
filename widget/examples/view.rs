#![deny(warnings)]
use std::borrow::Cow;
use dyn_context::ContextExt;
use tuifw_screen::{Key, Vector, Thickness, HAlign, VAlign, Point};
use tuifw_widget::view::{ViewTree, View, view_align_type, view_base_type};
use tuifw_widget::view::panels::{CanvasPanel, CanvasLayout, canvas_layout_type};
use tuifw_widget::view::decorators::{BorderDecorator, border_decorator_type};

fn double_border(tree: &mut ViewTree, view: View) {
    view.decorator_set_uncond(tree, border_decorator_type().tl(), Cow::Borrowed(&"╔"));
    view.decorator_set_uncond(tree, border_decorator_type().tr(), Cow::Borrowed(&"╗"));
    view.decorator_set_uncond(tree, border_decorator_type().bl(), Cow::Borrowed(&"╚"));
    view.decorator_set_uncond(tree, border_decorator_type().br(), Cow::Borrowed(&"╝"));
    view.decorator_set_uncond(tree, border_decorator_type().l(), Cow::Borrowed(&"║"));
    view.decorator_set_uncond(tree, border_decorator_type().t(), Cow::Borrowed(&"═"));
    view.decorator_set_uncond(tree, border_decorator_type().r(), Cow::Borrowed(&"║"));
    view.decorator_set_uncond(tree, border_decorator_type().b(), Cow::Borrowed(&"═"));
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let size = Vector { x: 13, y: 7 };
    let padding = Thickness::align(size, screen.size(), HAlign::Center, VAlign::Center);
    let tree = &mut ViewTree::new(screen, |_| ((), |tree| tree));
    CanvasPanel::new(tree, tree.root());
    let view = View::new(tree, tree.root(), |view| ((), view));
    CanvasLayout::new(tree, view);
    BorderDecorator::new(tree, view);
    double_border(tree, view);
    view.align_set_distinct(tree, view_align_type().w(), Some(size.x));
    view.align_set_distinct(tree, view_align_type().h(), Some(size.y));
    view.layout_set_distinct(tree, canvas_layout_type().tl(), Point { x: padding.l, y: padding.t });
    view.base_on(tree, view_base_type().input(), |view, context, input| {
        let tree = context.get_mut::<ViewTree>().expect("ViewTree required");
        let d = match input.key() {
            (n, Key::Left) | (n, Key::Char('h')) =>
                -Vector { x: (n.get() as i16).overflowing_mul(2).0, y: 0 },
            (n, Key::Right) | (n, Key::Char('l')) =>
                Vector { x: (n.get() as i16).overflowing_mul(2).0, y: 0 },
            (n, Key::Up) | (n, Key::Char('k')) =>
                -Vector { x: 0, y: n.get() as i16 },
            (n, Key::Down) | (n, Key::Char('j')) =>
                Vector { x: 0, y: n.get() as i16 },
            (_, Key::Escape) => return tree.quit(),
            _ => return,
        };
        let tl = view.layout_get(tree, canvas_layout_type().tl()).offset(d);
        view.layout_set_distinct(tree, canvas_layout_type().tl(), tl);
    });
    view.focus(tree);
    while ViewTree::update(tree, true).unwrap() { }
}
