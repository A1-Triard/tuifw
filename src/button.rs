use alloc::boxed::Box;
use alloc::string::{String, ToString};
use either::Left;
use tuifw_screen_base::{Error, Key, Point, Rect, Vector, text_width};
use tuifw_window::{Event, RenderPort, Timer, Widget, WidgetData, Window, WindowTree};
use tuifw_window::{CMD_GOT_PRIMARY_FOCUS, CMD_LOST_PRIMARY_FOCUS};
use tuifw_window::{CMD_GOT_SECONDARY_FOCUS, CMD_LOST_SECONDARY_FOCUS};

pub const CMD_CLICK: u16 = 100;

pub struct Button {
    border: (String, String),
    text: String,
    release_timer: Option<Timer>,
    cmd: u16,
    is_enabled: bool,
}

impl<State: ?Sized> WidgetData<State> for Button {
    fn drop_widget_data(&mut self, tree: &mut WindowTree<State>, _state: &mut State) {
        if let Some(release_timer) = self.release_timer.take() {
            release_timer.drop_timer(tree);
        }
    }
}

impl Button {
    pub fn new() -> Self {
        Button {
            border: ("[".to_string(), "]".to_string()),
            text: String::new(),
            release_timer: None,
            cmd: CMD_CLICK,
            is_enabled: true,
        }
    }

    fn set_palette<State: ?Sized>(tree: &mut WindowTree<State>, window: Window<State>) {
        window.palette_mut(tree, |palette| {
            palette.set(0, Left(14));
            palette.set(1, Left(15));
            palette.set(2, Left(16));
            palette.set(3, Left(17));
        });
    }

    pub fn window<State: ?Sized>(
        self,
        tree: &mut WindowTree<State>,
        parent: Window<State>,
        prev: Option<Window<State>>
    ) -> Result<Window<State>, Error> {
        let w = Window::new(tree, Box::new(ButtonWidget), Box::new(self), parent, prev)?;
        Self::set_palette(tree, w);
        Ok(w)
    }

    pub fn text(&self) -> &String {
        &self.text
    }

    pub fn text_mut<State: ?Sized, T>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: impl FnOnce(&mut String) -> T
    ) -> T {
        let data = &mut window.data_mut::<Button>(tree).text;
        let res = value(data);
        window.invalidate_measure(tree);
        res
    }

    pub fn border(&self) -> &(String, String) {
        &self.border
    }

    pub fn border_mut<State: ?Sized, T>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: impl FnOnce(&mut (String, String)) -> T
    ) -> T {
        let data = &mut window.data_mut::<Button>(tree).border;
        let res = value(data);
        window.invalidate_measure(tree);
        res
    }

    pub fn cmd(&self) -> u16 {
        self.cmd
    }

    pub fn set_cmd<State: ?Sized>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: u16
    ) {
        window.data_mut::<Button>(tree).cmd = value;
    }

    pub fn is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn set_is_enabled<State: ?Sized>(
        tree: &mut WindowTree<State>,
        window: Window<State>,
        value: bool
    ) {
        window.data_mut::<Button>(tree).is_enabled = value;
        window.invalidate_render(tree);
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct ButtonWidget;

impl<State: ?Sized> Widget<State> for ButtonWidget {
    fn render(
        &self,
        tree: &WindowTree<State>,
        window: Window<State>,
        rp: &mut RenderPort,
        _state: &mut State,
    ) {
        let bounds = window.inner_bounds(tree);
        let focused = window.is_focused(tree);
        let data = window.data::<Button>(tree);
        let pressed = data.release_timer.is_some();
        let color = if !data.is_enabled { 3 } else if pressed { 2 } else if focused { 1 } else { 0 };
        let color = window.color(tree, color);
        rp.out(Point { x: 1, y: 0 }, color.0, color.1, &data.text);
        rp.out(
            Point { x: 0, y: 0 },
            color.0,
            color.1,
            if pressed { " " } else { &data.border.0 }
        );
        rp.out(
            Point { x: bounds.r_inner(), y: 0 },
            color.0,
            color.1,
            if pressed { " " } else { &data.border.1 }
        );
    }

    fn measure(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        _available_width: Option<i16>,
        _available_height: Option<i16>,
        _state: &mut State,
    ) -> Vector {
        let data = window.data::<Button>(tree);
        Vector { x: text_width(&data.text).wrapping_add(2), y: 1 }
    }

    fn arrange(
        &self,
        _tree: &mut WindowTree<State>,
        _window: Window<State>,
        final_inner_bounds: Rect,
        _state: &mut State,
    ) -> Vector {
        final_inner_bounds.size
    }

    fn update(
        &self,
        tree: &mut WindowTree<State>,
        window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        state: &mut State,
    ) -> bool {
        match event {
            Event::Cmd(CMD_GOT_PRIMARY_FOCUS) | Event::Cmd(CMD_LOST_PRIMARY_FOCUS) |
            Event::Cmd(CMD_GOT_SECONDARY_FOCUS) | Event::Cmd(CMD_LOST_SECONDARY_FOCUS) => {
                window.invalidate_render(tree);
                true
            },
            Event::Key(_, Key::Enter) => {
                let data = window.data_mut::<Button>(tree);
                if data.is_enabled {
                    let release_timer = Timer::new(tree, 100, Box::new(move |tree, _state| {
                        let data = window.data_mut::<Button>(tree);
                        data.release_timer = None;
                        window.invalidate_render(tree);
                    }));
                    let data = window.data_mut::<Button>(tree);
                    let cmd = data.cmd;
                    if let Some(old_release_timer) = data.release_timer.replace(release_timer) {
                        old_release_timer.drop_timer(tree);
                    }
                    window.invalidate_render(tree);
                    window.raise(tree, Event::Cmd(cmd), state);
                    true
                } else {
                    false
                }
            },
            _ => false
        }
    }
}
