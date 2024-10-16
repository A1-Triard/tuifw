use tuifw_window::{Window, WindowTree};

pub trait VirtScrollViewerWidgetExtension {
    fn set_offset(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: u32);
    fn set_viewport(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: u32);
    fn set_extent(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: u32);
}

pub trait VirtItemsPresenterWidgetExtension {
    fn set_offset(&self, tree: &mut WindowTree, window: Window, vertical: bool, value: u32);
}
