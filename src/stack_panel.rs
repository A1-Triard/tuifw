use alloc::boxed::Box;
use tuifw_screen_base::{Error, Rect, Screen, Vector};
use tuifw_window::{Event, RenderPort, Widget, Window, WindowTree};

pub struct StackPanel {
    vertical: bool,
}

impl StackPanel {
    pub fn new() -> Self {
        StackPanel { vertical: true }
    }

    pub fn vertical(&self) -> bool { self.vertical }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        Window::new(tree, Box::new(StackPanelWidget), Box::new(self), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<State>, Error> {
        WindowTree::new(screen, Box::new(StackPanelWidget), Box::new(self))
    }

    pub fn set_vertical<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, value: bool) {
        window.data_mut::<StackPanel>(tree).vertical = value;
        window.invalidate_measure(tree);
    }
}

#[derive(Clone)]
pub struct StackPanelWidget;

impl<State: ?Sized> Widget<State> for StackPanelWidget {
    fn render(
        &self,
        _tree: &WindowTree<State>,
        _window: Window<State>,
        _rp: &mut RenderPort,
        _state: &mut State,
    ) { }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        available_width: Option<i16>,
        available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        let vertical = window.data::<StackPanel>(tree).vertical;
        if vertical {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, available_width, None, state);
                    size += Vector { x: 0, y: child.desired_size(tree).y };
                    size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        } else {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, None, available_height, state);
                    size += Vector { x: child.desired_size(tree).x, y: 0 };
                    size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        let vertical = window.data::<StackPanel>(tree).vertical;
        if vertical {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_desired_size = child.desired_size(tree);
                    child.arrange(tree, Rect { tl: pos, size: child_desired_size }, state);
                    pos = pos.offset(Vector { x: 0, y: child_desired_size.y });
                    size += Vector { x: 0, y: child.desired_size(tree).y };
                    size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        } else {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_desired_size = child.desired_size(tree);
                    child.arrange(tree, Rect { tl: pos, size: child_desired_size }, state);
                    pos = pos.offset(Vector { x: child_desired_size.x, y: 0 });
                    size += Vector { x: child.desired_size(tree).x, y: 0 };
                    size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
        }
    }

    fn update(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        _event: Event,
        _preview: bool,
        _state: &mut State,
    ) -> bool {
        false
    }
}
