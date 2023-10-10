use crate::{prop_string_measure, prop_value, widget};
use alloc::boxed::Box;
use alloc::string::String;
use either::Left;
use tuifw_screen_base::{Point, Rect, Vector, Key};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, Timer, label_width, label};

pub const CMD_LABEL_CLICK: u16 = 110;

pub struct Label {
    text: String,
    click_timer: Option<Timer>,
    cmd: u16,
}

impl<State: ?Sized> WidgetData<State> for Label {
    fn drop_widget_data(&mut self, tree: &mut WindowTree<State>, _state: &mut State) {
        if let Some(click_timer) = self.click_timer.take() {
            click_timer.drop_timer(tree);
        }
    }
}

impl Label {
    pub fn new() -> Self {
        Label { text: String::new(), click_timer: None, cmd: CMD_LABEL_CLICK }
    }

    fn init_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(12));
            palette.set(1, Left(13));
            palette.set(2, Left(14));
        });
    }

    widget!(LabelWidget; init_palette);
    prop_string_measure!(text);
    prop_value!(cmd: u16);
}

impl Default for Label {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct LabelWidget;

impl<State: ?Sized> Widget<State> for LabelWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let is_enabled = window.actual_is_enabled(tree);
        let data = window.data::<Label>(tree);
        let color = window.color(tree, if is_enabled { 0 } else { 1 });
        let color_hotkey = window.color(tree, if is_enabled { 2 } else { 1 });
        rp.label(Point { x: 0, y: 0 }, color, color_hotkey, &data.text);
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<Label>(tree);
        Vector { x: label_width(&data.text), y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<Label>(tree);
        Vector { x: label_width(&data.text), y: 1 }
    }

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        _state: &mut State,
    ) -> bool {
        let data = window.data::<Label>(tree);
        let label = label(&data.text);
        let Some(label) = label else { return false; };
        if event == Event::PostProcessKey(Key::Alt(label)) || event == Event::PostProcessKey(Key::Char(label)) {
            if window.is_enabled(tree) {
                let click_timer = Timer::new(tree, 0, Box::new(move |tree, state| {
                    let data = window.data_mut::<Label>(tree);
                    data.click_timer = None;
                    if window.is_enabled(tree) {
                        let data = window.data_mut::<Label>(tree);
                        let cmd = data.cmd;
                        window.raise(tree, Event::Cmd(cmd), state);
                        let focus = window.actual_focus_tab(tree);
                        focus.set_focused_primary(tree, true);
                    }
                }));
                let data = window.data_mut::<Label>(tree);
                if let Some(old_click_timer) = data.click_timer.replace(click_timer) {
                    old_click_timer.drop_timer(tree);
                }
                return true;
            }
        }
        false
    }

    fn post_process(&self) -> bool { true }
}
