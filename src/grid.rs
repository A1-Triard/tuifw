use crate::widget;
use alloc::boxed::Box;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Rect, Vector};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App};

#[derive(Clone)]
struct GridLayout {
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
}

impl Layout for GridLayout { }

impl Default for GridLayout {
    fn default() -> Self {
        GridLayout {
            row: 0,
            column: 0,
            row_span: 1,
            column_span: 1,
        }
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum GridLength {
    Auto,
    Fixed(i16),
    Star(f32),
}

#[derive(Clone)]
pub struct Column {
    pub width: GridLength,
    pub min_width: i16,
    pub max_width: i16,
}

#[derive(Clone)]
pub struct Row {
    pub height: GridLength,
    pub min_height: i16,
    pub max_height: i16,
}

widget! {
    #[widget(GridWidget)]
    pub struct Grid {
        #[property(ref, measure)]
        columns: Vec<Column>,
        #[property(ref, measure)]
        rows: Vec<Row>,
    }
}

impl Grid {
    pub fn row(tree: &WindowTree, window: Window) -> usize {
        window.layout::<GridLayout>(tree).map_or(0, |x| x.row)
    }

    pub fn column(tree: &WindowTree, window: Window) -> usize {
        window.layout::<GridLayout>(tree).map_or(0, |x| x.column)
    }

    pub fn row_span(tree: &WindowTree, window: Window) -> usize {
        window.layout::<GridLayout>(tree).map_or(1, |x| x.row_span)
    }

    pub fn column_span(tree: &WindowTree, window: Window) -> usize {
        window.layout::<GridLayout>(tree).map_or(1, |x| x.column_span)
    }

    pub fn set_row(tree: &mut WindowTree, window: Window, value: usize) {
        window.layout_mut(tree, |layout: &mut GridLayout| layout.row = value);
    }

    pub fn set_column(tree: &mut WindowTree, window: Window, value: usize) {
        window.layout_mut(tree, |layout: &mut GridLayout| layout.column = value);
    }

    pub fn set_row_span(tree: &mut WindowTree, window: Window, value: usize) {
        window.layout_mut(tree, |layout: &mut GridLayout| layout.row_span = value);
    }

    pub fn set_column_span(tree: &mut WindowTree, window: Window, value: usize) {
        window.layout_mut(tree, |layout: &mut GridLayout| layout.column_span = value);
    }
}

#[derive(Clone, Default)]
pub struct GridWidget;

impl_supports_interfaces!(GridWidget);

impl Widget for GridWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(Grid {
            columns: Vec::new(), rows: Vec::new()
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        Grid::clone(tree, source, dest, clone_window);
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
        available_width: Option<i16>,
        available_height: Option<i16>,
        app: &mut dyn App,
    ) -> Vector {
        let vertical = window.data::<Grid>(tree).vertical;
        if vertical {
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    child.measure(tree, available_width, None, app);
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
                    child.measure(tree, None, available_height, app);
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
        tree: &mut WindowTree,
        window: Window,
        final_inner_bounds: Rect,
        app: &mut dyn App,
    ) -> Vector {
        let vertical = window.data::<Grid>(tree).vertical;
        if vertical {
            let mut pos = final_inner_bounds.tl;
            let mut size = Vector::null();
            if let Some(first_child) = window.first_child(tree) {
                let mut child = first_child;
                loop {
                    let child_size = Vector { x: final_inner_bounds.w(), y: child.desired_size(tree).y };
                    child.arrange(tree, Rect { tl: pos, size: child_size }, app);
                    pos = pos.offset(Vector { x: 0, y: child_size.y });
                    size += Vector { x: 0, y: child_size.y };
                    size = size.max(Vector { x: child_size.x, y: 0 });
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
                    let child_size = Vector { x: child.desired_size(tree).x, y: final_inner_bounds.h() };
                    child.arrange(tree, Rect { tl: pos, size: child_size }, app);
                    pos = pos.offset(Vector { x: child_size.x, y: 0 });
                    size += Vector { x: child_size.x, y: 0 };
                    size = size.max(Vector { x: 0, y: child_size.y });
                    child = child.next(tree);
                    if child == first_child { break; }
                }
            }
            size
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
