#![deny(warnings)]
use boow::Bow;
use tuifw_screen::{Attr, Color};
use tuifw_widget::view::{Text, ViewTree, View, view_align_type};
use tuifw_widget::view::panels::{CanvasPanel, CanvasLayout};
use tuifw_widget::view::decorators::{BorderDecorator, border_decorator_type};

fn double_border(tree: &mut ViewTree, view: View) {
    view.decorator_set_uncond(tree, border_decorator_type().tl(), Some(Text {
        value: Bow::Borrowed(&"╔"), fg: Color::Green, bg: None, attr: Attr::empty()
    }));
    view.decorator_set_uncond(tree, border_decorator_type().tr(), Some(Text {
        value: Bow::Borrowed(&"╗"), fg: Color::Green, bg: None, attr: Attr::empty()
    }));
    view.decorator_set_uncond(tree, border_decorator_type().bl(), Some(Text {
        value: Bow::Borrowed(&"╚"), fg: Color::Green, bg: None, attr: Attr::empty()
    }));
    view.decorator_set_uncond(tree, border_decorator_type().br(), Some(Text {
        value: Bow::Borrowed(&"╝"), fg: Color::Green, bg: None, attr: Attr::empty()
    }));
    view.decorator_set_uncond(tree, border_decorator_type().l(), Some(Text {
        value: Bow::Borrowed(&"║"), fg: Color::Green, bg: None, attr: Attr::empty()
    }));
    view.decorator_set_uncond(tree, border_decorator_type().t(), Some(Text {
        value: Bow::Borrowed(&"═"), fg: Color::Green, bg: None, attr: Attr::empty()
    }));
    view.decorator_set_uncond(tree, border_decorator_type().r(), Some(Text {
        value: Bow::Borrowed(&"║"), fg: Color::Green, bg: None, attr: Attr::empty()
    }));
    view.decorator_set_uncond(tree, border_decorator_type().b(), Some(Text {
        value: Bow::Borrowed(&"═"), fg: Color::Green, bg: None, attr: Attr::empty()
    }));
}

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let tree = &mut ViewTree::new(screen, |_| ((), |tree| tree));
    CanvasPanel::new(tree, tree.root());
    let view = View::new(tree, tree.root(), |view| ((), view));
    CanvasLayout::new(tree, view);
    BorderDecorator::new(tree, view);
    double_border(tree, view);
    view.align_set_distinct(tree, view_align_type().w(), Some(8));
    view.align_set_distinct(tree, view_align_type().h(), Some(8));
    loop {
        ViewTree::update(tree, true).unwrap();
    }
}
