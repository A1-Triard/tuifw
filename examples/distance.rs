#![windows_subsystem = "windows"]

#![deny(warnings)]

use std::mem::replace;
use std::str::FromStr;
use timer_no_std::MonoClock;
use tuifw::{Background, Button, Dock, DockPanel, InputLine, StackPanel, StaticText};
use tuifw::{CMD_IS_VALID_EMPTY_CHANGED, IntRangeValidator, FloatRangeValidator};
use tuifw_screen::{HAlign, VAlign, Key, Thickness};
use tuifw_window::{Event, EventHandler, Window, WindowTree};

const CMD_CALC: u16 = 1000;

struct State {
    a: Window<State>,
    v: Window<State>,
    t: Window<State>,
    n: Window<State>,
    s: Window<State>,
    calc: Window<State>,
}

#[derive(Clone)]
struct RootEventHandler;

impl EventHandler<State> for RootEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree<State>,
        _window: Window<State>,
        event: Event,
        _event_source: Window<State>,
        state: &mut State
    ) -> bool {
        match event {
            Event::Key(_, Key::Escape) => {
                tree.quit();
                true
            },
            Event::Cmd(CMD_CALC) => {
                let a = f64::from_str(state.a.data::<InputLine>(tree).text()).unwrap();
                let v = f64::from_str(state.v.data::<InputLine>(tree).text()).unwrap();
                let t = f64::from_str(state.t.data::<InputLine>(tree).text()).unwrap();
                let n = f64::from(i32::from_str(state.n.data::<InputLine>(tree).text()).unwrap());
                let s = v * t + a * t * (n - 1.0) / (2.0 * n);
                StaticText::text_mut(tree, state.s, |value| replace(value, s.to_string()));
                true
            },
            Event::Cmd(CMD_IS_VALID_EMPTY_CHANGED) => {
                let a_empty = state.a.data::<InputLine>(tree).is_empty();
                let v_empty = state.v.data::<InputLine>(tree).is_empty();
                let t_empty = state.t.data::<InputLine>(tree).is_empty();
                let n_empty = state.n.data::<InputLine>(tree).is_empty();
                let a_valid = state.a.data::<InputLine>(tree).is_valid();
                let v_valid = state.v.data::<InputLine>(tree).is_valid();
                let t_valid = state.t.data::<InputLine>(tree).is_valid();
                let n_valid = state.n.data::<InputLine>(tree).is_valid();
                Button::set_is_enabled(tree, state.calc,
                    a_valid && v_valid && t_valid && n_valid && !a_empty && !v_empty && !t_empty && !n_empty
                );
                true
            },
            _ => false
        }
    }
}

fn main() {
    let clock = unsafe { MonoClock::new() };
    let screen = unsafe { tuifw_screen::init(None, None) }.unwrap();
    let tree = &mut Background::new().window_tree(screen, &clock).unwrap();
    let root = tree.root();
    root.set_event_handler(tree, Some(Box::new(RootEventHandler)));

    let controls = StackPanel::new().window(tree, root, None).unwrap();
    controls.set_h_align(tree, Some(HAlign::Center));
    controls.set_v_align(tree, Some(VAlign::Center));

    let edits_with_labels = DockPanel::new().window(tree, controls, None).unwrap();
    let labels = StackPanel::new().window(tree, edits_with_labels, None).unwrap();
    DockPanel::set_layout(tree, labels, Some(Dock::Left));
    let edits = StackPanel::new().window(tree, edits_with_labels, Some(labels)).unwrap();
    edits.set_width(tree, 12);

    let a_label = StaticText::new().window(tree, labels, None).unwrap();
    StaticText::text_mut(tree, a_label, |value| replace(value, "A =".to_string()));
    a_label.set_margin(tree, Thickness::new(1, 1, 0, 1));
    let a = InputLine::new().window(tree, edits, None).unwrap();
    InputLine::validator_mut(tree, a, |value| value.replace(
        Box::new(FloatRangeValidator { min: f64::MIN, max: f64::MAX })
    ));
    InputLine::default_mut(tree, a, |value| replace(value, "0".to_string()));
    InputLine::text_mut(tree, a, |value| replace(value, "0".to_string()));
    a.set_margin(tree, Thickness::new(1, 1, 1, 1));

    let v_label = StaticText::new().window(tree, labels, Some(a_label)).unwrap();
    StaticText::text_mut(tree, v_label, |value| replace(value, "V =".to_string()));
    v_label.set_margin(tree, Thickness::new(1, 0, 0, 1));
    let v = InputLine::new().window(tree, edits, Some(a)).unwrap();
    InputLine::validator_mut(tree, v, |value| value.replace(
        Box::new(FloatRangeValidator { min: f64::MIN, max: f64::MAX })
    ));
    InputLine::default_mut(tree, v, |value| replace(value, "0".to_string()));
    InputLine::text_mut(tree, v, |value| replace(value, "1".to_string()));
    v.set_margin(tree, Thickness::new(1, 0, 1, 1));

    let t_label = StaticText::new().window(tree, labels, Some(v_label)).unwrap();
    StaticText::text_mut(tree, t_label, |value| replace(value, "T =".to_string()));
    t_label.set_margin(tree, Thickness::new(1, 0, 0, 1));
    let t = InputLine::new().window(tree, edits, Some(v)).unwrap();
    InputLine::validator_mut(tree, t, |value| value.replace(
        Box::new(FloatRangeValidator { min: f64::MIN, max: f64::MAX })
    ));
    InputLine::default_mut(tree, t, |value| replace(value, "0".to_string()));
    InputLine::text_mut(tree, t, |value| replace(value, "0".to_string()));
    t.set_margin(tree, Thickness::new(1, 0, 1, 1));

    let n_label = StaticText::new().window(tree, labels, Some(t_label)).unwrap();
    StaticText::text_mut(tree, n_label, |value| replace(value, "N =".to_string()));
    n_label.set_margin(tree, Thickness::new(1, 0, 0, 1));
    let n = InputLine::new().window(tree, edits, Some(t)).unwrap();
    InputLine::validator_mut(tree, n, |value| value.replace(
        Box::new(IntRangeValidator { min: 1, max: i32::MAX })
    ));
    InputLine::default_mut(tree, n, |value| replace(value, "1".to_string()));
    InputLine::text_mut(tree, n, |value| replace(value, "1".to_string()));
    n.set_margin(tree, Thickness::new(1, 0, 1, 1));

    let calc = Button::new().window(tree, controls, Some(edits_with_labels)).unwrap();
    Button::text_mut(tree, calc, |value| replace(value, "Calculate".to_string()));
    Button::set_cmd(tree, calc, CMD_CALC);
    calc.set_h_align(tree, Some(HAlign::Center));

    let result = DockPanel::new().window(tree, controls, Some(calc)).unwrap();
    let s_label = StaticText::new().window(tree, result, None).unwrap();
    s_label.set_margin(tree, Thickness::new(1, 1, 0, 1));
    StaticText::text_mut(tree, s_label, |value| replace(value, "S =".to_string()));
    DockPanel::set_layout(tree, s_label, Some(Dock::Left));
    let result_value = Background::new().window(tree, result, Some(s_label)).unwrap();
    result_value.set_width(tree, 12);
    let s = StaticText::new().window(tree, result_value, None).unwrap();
    s.set_h_align(tree, Some(HAlign::Right));
    s.set_margin(tree, Thickness::new(1, 1, 1, 1));

    a.set_next_focus(tree, v);
    v.set_next_focus(tree, t);
    t.set_next_focus(tree, n);
    n.set_next_focus(tree, a);

    let state = &mut State { a, v, t, n, s, calc };
    a.focus(tree, true, state);
    calc.focus(tree, false, state);
    tree.run(state).unwrap();
}
