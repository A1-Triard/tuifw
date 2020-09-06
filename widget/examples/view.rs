use tuifw_widget::view::{ViewTree, View};
use tuifw_widget::view::panels::{CanvasPanel, CanvasLayout};
use tuifw_widget::view::decorators::{BorderDecorator};

fn main() {
    let screen = unsafe { tuifw_screen::init() }.unwrap();
    let tree = &mut ViewTree::new(screen, |_| ((), |tree| tree));
    CanvasPanel::new(tree, tree.root());
    let view = View::new(tree, tree.root(), |view| ((), view));
    CanvasLayout::new(tree, view);
    BorderDecorator::new(tree, view);
}
