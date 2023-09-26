use alloc::boxed::Box;
use core::mem::replace;
use tuifw_screen_base::{Error, Rect, Screen, Vector, Thickness, Point};
use tuifw_window::{Event, Layout, RenderPort, Widget, Window, WindowTree};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Dock { Left, Top, Right, Bottom }

pub struct DockLayout {
    pub dock: Option<Dock>,
}

impl Layout for DockLayout { }

pub struct DockPanel { }

impl DockPanel {
    pub fn new() -> Self {
        DockPanel { }
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        Window::new(tree, Box::new(DockPanelWidget), Box::new(self), parent, prev)
    }

    pub fn window_tree<State: ?Sized>(
        self,
        screen: Box<dyn Screen>
    ) -> Result<WindowTree<State>, Error> {
        WindowTree::new(screen, Box::new(DockPanelWidget), Box::new(self))
    }

    pub fn set_layout<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>, dock: Option<Dock>) {
        window.layout_mut(tree, |layout| replace(layout, Some(Box::new(DockLayout { dock }))));
    }
}

#[derive(Clone)]
pub struct DockPanelWidget;

impl<State: ?Sized> Widget<State> for DockPanelWidget {
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
        mut available_width: Option<i16>,
        mut available_height: Option<i16>,
        state: &mut State,
    ) -> Vector {
        if let Some(first_child) = window.first_child(tree) {
            let mut size = Vector::null();
            let mut docked = Thickness::all(0);
            let mut child = first_child;
            loop {
                let dock = child.layout::<DockLayout>(tree).and_then(|x| x.dock);
                match dock {
                    None => { },
                    Some(Dock::Left) => {
                        child.measure(tree, None, available_height, state);
                        if let Some(available_width) = available_width.as_mut() {
                            *available_width =
                                (*available_width as u16).saturating_sub(child.desired_size(tree).x as u16) as i16;
                        }
                        size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                        let docked_child = Thickness::new(i32::from(child.desired_size(tree).x), 0, 0, 0);
                        docked += docked_child;
                        size = docked_child.shrink_rect_size(size);
                    },
                    Some(Dock::Top) => {
                        child.measure(tree, available_width, None, state);
                        if let Some(available_height) = available_height.as_mut() {
                            *available_height =
                                (*available_height as u16).saturating_sub(child.desired_size(tree).y as u16) as i16;
                        }
                        size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                        let docked_child = Thickness::new(0, i32::from(child.desired_size(tree).y), 0, 0);
                        docked += docked_child;
                        size = docked_child.shrink_rect_size(size);
                    },
                    Some(Dock::Right) => {
                        child.measure(tree, None, available_height, state);
                        if let Some(available_width) = available_width.as_mut() {
                            *available_width =
                                (*available_width as u16).saturating_sub(child.desired_size(tree).x as u16) as i16;
                        }
                        size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                        let docked_child = Thickness::new(0, 0, i32::from(child.desired_size(tree).x), 0);
                        docked += docked_child;
                        size = docked_child.shrink_rect_size(size);
                    },
                    Some(Dock::Bottom) => {
                        child.measure(tree, available_width, None, state);
                        if let Some(available_height) = available_height.as_mut() {
                            *available_height =
                                (*available_height as u16).saturating_sub(child.desired_size(tree).y as u16) as i16;
                        }
                        size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                        let docked_child = Thickness::new(0, 0, 0, i32::from(child.desired_size(tree).y));
                        docked += docked_child;
                        size = docked_child.shrink_rect_size(size);
                    },
                }
                child = child.next(tree);
                if child == first_child { break; }
            }
            let mut child = first_child;
            loop {
                let dock = child.layout::<DockLayout>(tree).and_then(|x| x.dock);
                if dock.is_none() {
                    child.measure(tree, available_width, available_height, state);
                    size = size.max(child.desired_size(tree));
                }
                child = child.next(tree);
                if child == first_child { break; }
            }
            docked.expand_rect_size(size)
        } else {
            Vector::null()
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        final_inner_bounds: Rect,
        state: &mut State,
    ) -> Vector {
        if let Some(first_child) = window.first_child(tree) {
            let mut size = Vector::null();
            let mut docked = Thickness::all(0);
            let mut child = first_child;
            loop {
                let bounds = docked.shrink_rect(final_inner_bounds);
                let dock = child.layout::<DockLayout>(tree).and_then(|x| x.dock);
                match dock {
                    None => { },
                    Some(Dock::Left) => {
                        child.arrange(
                            tree,
                            Rect { tl: bounds.tl, size: Vector { x: child.desired_size(tree).x, y: bounds.h() } },
                            state
                        );
                        size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                        let docked_child = Thickness::new(i32::from(child.desired_size(tree).x), 0, 0, 0);
                        docked += docked_child;
                        size = docked_child.shrink_rect_size(size);
                    },
                    Some(Dock::Top) => {
                        child.arrange(
                            tree,
                            Rect { tl: bounds.tl, size: Vector { x: bounds.w(), y: child.desired_size(tree).y } },
                            state
                        );
                        size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                        let docked_child = Thickness::new(0, i32::from(child.desired_size(tree).y), 0, 0);
                        docked += docked_child;
                        size = docked_child.shrink_rect_size(size);
                    },
                    Some(Dock::Right) => {
                        child.arrange(
                            tree,
                            Rect::from_tl_br(
                                Point {
                                    x: bounds.r().wrapping_sub(child.desired_size(tree).x),
                                    y: bounds.t()
                                },
                                bounds.br()
                            ),
                            state
                        );
                        size = size.max(Vector { x: 0, y: child.desired_size(tree).y });
                        let docked_child = Thickness::new(0, 0, i32::from(child.desired_size(tree).x), 0);
                        docked += docked_child;
                        size = docked_child.shrink_rect_size(size);
                    },
                    Some(Dock::Bottom) => {
                        child.arrange(
                            tree,
                            Rect::from_tl_br(
                                Point {
                                    x: bounds.l(),
                                    y: bounds.b().wrapping_sub(child.desired_size(tree).y)
                                },
                                bounds.br()
                            ),
                            state
                        );
                        size = size.max(Vector { x: child.desired_size(tree).x, y: 0 });
                        let docked_child = Thickness::new(0, 0, 0, i32::from(child.desired_size(tree).y));
                        docked += docked_child;
                        size = docked_child.shrink_rect_size(size);
                    },
                }
                child = child.next(tree);
                if child == first_child { break; }
            }
            let bounds = docked.shrink_rect(final_inner_bounds);
            let mut child = first_child;
            loop {
                let dock = child.layout::<DockLayout>(tree).and_then(|x| x.dock);
                if dock.is_none() {
                    child.arrange(tree, bounds, state);
                    size = size.max(child.render_bounds(tree).size);
                }
                child = child.next(tree);
                if child == first_child { break; }
            }
            docked.expand_rect_size(size)
        } else {
            Vector::null()
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
