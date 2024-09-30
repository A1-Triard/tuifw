use crate::widget;
use alloc::boxed::Box;
use alloc::string::String;
use dynamic_cast::impl_supports_interfaces;
use phantom_type::PhantomType;
use tuifw_screen_base::{Key, Point, Rect, Vector, Error};
use tuifw_window::{Event, RenderPort, Widget, WidgetData, Window, WindowTree, App, Color};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS, label_width, label};
use tuifw_window::{COLOR_LABEL, COLOR_HOTKEY, COLOR_DISABLED};

pub const CMD_RADIO_BUTTON_CLICK: u16 = 120;

widget! {
    #[widget(RadioButtonWidget, init=init_palette, drop=drop_controller)]
    pub struct RadioButton {
        #[property(copy, render)]
        is_on: bool,
        #[property(copy)]
        allow_turn_off: bool,
        #[property(copy)]
        cmd: u16,
        #[property(str, measure)]
        text: String,
        controller: RadioButtonController<RadioButton>,
    }
}

pub trait IsRadioButton: WidgetData + Sized {
    fn controller(&self) -> &RadioButtonController<Self>;
    fn controller_mut(&mut self) -> &mut RadioButtonController<Self>;
    fn cmd(&self) -> u16;
    fn label(&self) -> Option<char>;
    fn is_on(&self) -> bool;
    fn set_is_on(&mut self, value: bool);
    fn allow_turn_off(&self) -> bool;
}

impl IsRadioButton for RadioButton {
    fn controller(&self) -> &RadioButtonController<Self> {
        &self.controller
    }

    fn controller_mut(&mut self) -> &mut RadioButtonController<Self> {
        &mut self.controller
    }

    fn cmd(&self) -> u16 {
        self.cmd
    }

    fn label(&self) -> Option<char> {
        label(&self.text)
    }

    fn is_on(&self) -> bool {
        self.is_on
    }

    fn set_is_on(&mut self, value: bool) {
        self.is_on = value;
    }

    fn allow_turn_off(&self) -> bool {
        self.allow_turn_off
    }
}

pub struct RadioButtonController<RadioButton: IsRadioButton> {
    _phantom: PhantomType<RadioButton>,
}

impl<RadioButton: IsRadioButton> Default for RadioButtonController<RadioButton> {
    fn default() -> Self { RadioButtonController::new() }
}

impl<RadioButton: IsRadioButton> RadioButtonController<RadioButton> {
    pub fn new() -> Self {
        RadioButtonController {
            _phantom: PhantomType::new(),
        }
    }

    pub fn drop_controller(&mut self, _tree: &mut WindowTree, _app: &mut dyn App) {
    }

    fn click(tree: &mut WindowTree, window: Window, app: &mut dyn App) -> bool {
        let data = window.data_mut::<RadioButton>(tree);
        if !data.is_on() || data.allow_turn_off() {
            data.set_is_on(!data.is_on());
            let cmd = data.cmd();
            if data.is_on() {
                let mut sibling = window.next(tree);
                while sibling != window {
                    sibling.data_mut::<RadioButton>(tree).set_is_on(false);
                    sibling.invalidate_render(tree);
                    sibling = sibling.next(tree);
                }
            }
            window.invalidate_render(tree);
            window.raise(tree, Event::Cmd(cmd), app);
            true
        } else {
            false
        }
    }

    pub fn update(
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        _event_source: Window,
        app: &mut dyn App,
    ) -> bool {
        match event {
            Event::Key(Key::Char(' ')) => {
                if window.actual_is_enabled(tree) {
                    Self::click(tree, window, app)
                } else {
                    false
                }
            },
            Event::Click(_) => {
                if window.actual_is_enabled(tree) {
                    Self::click(tree, window, app)
                } else {
                    false
                }
            },
            Event::PostProcessKey(Key::Alt(c)) | Event::PostProcessKey(Key::Char(c)) => {
                if window.actual_is_enabled(tree) {
                    let data = window.data_mut::<RadioButton>(tree);
                    let label = data.label();
                    if Some(c) == label {
                        window.set_focused_primary(tree, true);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
            _ => false
        }
    }
}

impl RadioButton {
    fn init_palette(tree: &mut WindowTree, window: Window) -> Result<(), Error> {
        window.palette_mut(tree, |palette| {
            palette.set(0, Color::Palette(COLOR_LABEL));
            palette.set(1, Color::Palette(COLOR_HOTKEY));
            palette.set(2, Color::Palette(COLOR_DISABLED));
        });
        Ok(())
    }

    fn drop_controller(&mut self, tree: &mut WindowTree, app: &mut dyn App) {
        self.controller.drop_controller(tree, app);
    }
}

#[derive(Clone, Default)]
struct RadioButtonWidget;

impl_supports_interfaces!(RadioButtonWidget);

impl Widget for RadioButtonWidget {
    fn new(&self) -> Box<dyn WidgetData> {
        Box::new(RadioButton {
            is_on: false,
            allow_turn_off: false,
            cmd: CMD_RADIO_BUTTON_CLICK,
            text: String::new(),
            controller: RadioButtonController::new()
        })
    }

    fn clone_data(
        &self,
        tree: &mut WindowTree,
        source: Window,
        dest: Window,
        clone_window: Box<dyn Fn(&WindowTree, Window) -> Window>,
    ) {
        RadioButton::clone(tree, source, dest, clone_window);
    }

    fn render(
        &self,
        tree: &WindowTree,
        window: Window,
        rp: &mut RenderPort,
        _app: &mut dyn App,
    ) {
        let is_enabled = window.actual_is_enabled(tree);
        let focused = window.is_focused(tree);
        let data = window.data::<RadioButton>(tree);
        let color = window.color(tree, if is_enabled { 0 } else { 2 });
        let color_hotkey = window.color(tree, if is_enabled { 1 } else { 2 });
        rp.text(Point { x: 1, y: 0 }, color, if data.is_on { "•" } else { " " });
        rp.text(Point { x: 0, y: 0 }, color, "(");
        rp.text(Point { x: 2, y: 0 }, color, ")");
        if !data.text.is_empty() {
            rp.text(Point { x: 3, y: 0 }, color, " ");
            rp.label(Point { x: 4, y: 0 }, color, color_hotkey, &data.text);
        }
        if focused { rp.cursor(Point { x: 1, y: 0 }); }
    }

    fn measure(
        &self,
        tree: &mut WindowTree,
        window: Window,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _app: &mut dyn App,
    ) -> Vector {
        let data = window.data::<RadioButton>(tree);
        if data.text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
        }
    }

    fn arrange(
        &self,
        tree: &mut WindowTree,
        window: Window,
        _final_inner_bounds: Rect,
        _app: &mut dyn App,
    ) -> Vector {
        let data = window.data::<RadioButton>(tree);
        if data.text.is_empty() {
            Vector { x: 3, y: 1 }
        } else {
            Vector { x: label_width(&data.text).wrapping_add(4), y: 1 }
        }
    }

    fn update(
        &self,
        tree: &mut WindowTree,
        window: Window,
        event: Event,
        event_source: Window,
        app: &mut dyn App,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) | Event::Cmd(CMD_LOST_PRIMARY_FOCUS) => {
                window.invalidate_render(tree);
            },
            _ => { },
        }
        <RadioButtonController::<RadioButton>>::update(tree, window, event, event_source, app)
    }

    fn post_process(&self) -> bool { true }
}
