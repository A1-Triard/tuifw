use crate::widget;
use alloc::boxed::Box;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Rect, Vector, Thickness, Point};
use tuifw_window::{Event, Layout, RenderPort, Widget, WidgetData, Window, WindowTree, App};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Dock { Left, Top, Right, Bottom }

#[derive(Clone)]
struct DockLayout {
    dock: Option<Dock>,
}

impl Layout for DockLayout { }

impl Default for DockLayout {
    fn default() -> Self { DockLayout { dock: None } }
}

widget! {
    #[widget(DockPanelWidget)]
    pub struct DockPanel { }
}

impl DockPanel {
    pub fn dock(tree: &WindowTree, window: Window) -> Option<Dock> {
        window.layout::<DockLayout>(tree).and_then(|x| x.dock)
    }

    pub fn set_dock(tree: &mut WindowTree, window: Window, value: Option<Dock>) {
        window.layout_mut(tree, |x: &mut DockLayout| x.dock = value);
    }
}

#[derive(Clone, Default)]
pub struct DockPanelWidget;

impl_supports_interfaces!(DockPanelWidget);

impl Widget for DockPanelWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(DockPanel { })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        DockPanel::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        _tree: &WindowTree,
        _window: Window,
        _rp: &mut RenderPort,
        _app: &mut dyn App,
    ) { }

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        mut available_width: Option<i16>,
        mut available_height: Option<i16>,
        app: &mut dyn App,
    ) -> Vector {
        if let Some(first_child) = window.first_child(tree) {
            let mut size = Vector::null();
            let mut docked = Thickness::all(0);
            let mut child = first_child;
            loop {
                let dock = DockPanel::dock(tree, child);
                match dock {
                    None => { },
                    Some(Dock::Left) => {
                        child.measure(tree, None, available_height, app);
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
                        child.measure(tree, available_width, None, app);
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
                        child.measure(tree, None, available_height, app);
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
                        child.measure(tree, available_width, None, app);
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
                let dock = DockPanel::dock(tree, child);
                if dock.is_none() {
                    child.measure(tree, available_width, available_height, app);
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
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        app: &mut dyn App,
    ) -> Vector {
        if let Some(first_child) = window.first_child(tree) {
            let mut size = Vector::null();
            let mut docked = Thickness::all(0);
            let mut child = first_child;
            loop {
                let bounds = docked.shrink_rect(final_inner_bounds);
                let dock = DockPanel::dock(tree, child);
                match dock {
                    None => { },
                    Some(Dock::Left) => {
                        child.arrange(
                            tree,
                            Rect { tl: bounds.tl, size: Vector { x: child.desired_size(tree).x, y: bounds.h() } },
                            app
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
                            app
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
                            app
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
                            app
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
                let dock = DockPanel::dock(tree, child);
                if dock.is_none() {
                    child.arrange(tree, bounds, app);
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
        _tree: &mut WindowTree,
        _window: Window,
        _event: Event,
        _event_source: Window,
        _app: &mut dyn App,
    ) -> bool {
        false
    }
}
