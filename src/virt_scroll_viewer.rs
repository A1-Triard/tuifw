use tuifw_window::{Window, WindowTree};

pub trait VirtScrollViewerWidgetExtension {
    fn set_offset(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: i16);
    fn set_viewport(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: i16);
    fn set_extent(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: i16);
}

pub trait VirtItemsPresenterWidgetExtension { }
