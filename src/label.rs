use crate::widget;
use alloc::boxed::Box;
use alloc::string::String;
use dynamic_cast::impl_supports_interfaces;
use tuifw_screen_base::{Point, Rect, Vector, Key, Error};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, Timer, label_width, label};
use tuifw_window::{COLOR_LABEL, COLOR_HOTKEY, COLOR_DISABLED, App, Color};

pub const CMD_LABEL_CLICK: u16 = 160;

widget! {
    #[widget(LabelWidget, init=init_palette, drop=drop_timers)]
    pub struct Label {
        #[property(str, measure)]
        text: String,
        click_timer: Option<Timer>,
        #[property(copy)]
        cmd: u16,
        #[property(window)]
        focus: Option<Window>,
    }
}

impl Label {
    fn init_palette(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        window.palette_mut(tree, |palette| {
            palette.set(0, Color::Palette(COLOR_LABEL));
            palette.set(1, Color::Palette(COLOR_HOTKEY));
            palette.set(2, Color::Palette(COLOR_DISABLED));
        });
        Ok(())
    }

    fn drop_timers(&mut self, tree: &mut WindowTree, _app: &mut dyn App) {
        if let Some(click_timer) = self.click_timer.take() {
            click_timer.drop_timer(tree);
        }
    }
}

#[derive(Clone, Default)]
struct LabelWidget;

impl_supports_interfaces!(LabelWidget);

impl Widget for LabelWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(Label {
            text: String::new(), click_timer: None, cmd: CMD_LABEL_CLICK, focus: None
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        Label::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
    ) {
        let is_enabled = window.actual_is_enabled(tree);
        let data = window.data::<Label>(tree);
        let color = window.color(tree, if is_enabled { 0 } else { 2 });
        let color_hotkey = window.color(tree, if is_enabled { 1 } else { 2 });
        rp.label(Point { x: 0, y: 0 }, color, color_hotkey, &data.text);
    }

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _app: &mut dyn App,
    ) -> Vector {
        let data = window.data::<Label>(tree);
        Vector { x: label_width(&data.text), y: 1 }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        _final_inner_bounds: Rect,
        _app: &mut dyn App,
    ) -> Vector {
        let data = window.data::<Label>(tree);
        Vector { x: label_width(&data.text), y: 1 }
    }

    fn update(
        &self,
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        _event_source: Window,
        _app: &mut dyn App,
    ) -> bool {
        let data = window.data::<Label>(tree);
        let label = label(&data.text);
        let Some(label) = label else { return false; };
        if event == Event::PostProcessKey(Key::Alt(label)) || event == Event::PostProcessKey(Key::Char(label)) {
            if window.actual_is_enabled(tree) {
                let click_timer = Timer::new(tree, 0, Box::new(move |tree, app| {
                    let data = window.data_mut::<Label>(tree);
                    data.click_timer = None;
                    if window.actual_is_enabled(tree) {
                        let data = window.data_mut::<Label>(tree);
                        let cmd = data.cmd;
                        let focus = data.focus;
                        window.raise(tree, Event::Cmd(cmd), app);
                        if let Some(focus) = focus {
                            focus.set_focused_primary(tree, true);
                        }
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
