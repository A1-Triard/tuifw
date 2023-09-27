#![windows_subsystem = "windows"]

#![deny(warnings)]

use std::mem::replace;
use timer_no_std::MonoClock;
use tuifw::{Background, Button, Dock, DockPanel, InputLine, InputLineValueRange, StackPanel, StaticText};
use tuifw_screen::{HAlign, VAlign, Key, Thickness};
use tuifw_window::{Event, EventHandler, Window, WindowTree};

#[derive(Clone)]
struct RootEventHandler;

impl EventHandler<()> for RootEventHandler {
    fn invoke(
        &self,
        tree: &mut WindowTree<()>,
        _window: Window<()>,
        event: Event,
        _event_source: Window<()>,
        _state: &mut ()
    ) -> bool {
        if let Event::Key(_, Key::Escape) = event {
            tree.quit();
            true
        } else {
            false
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
    StaticText::text_mut(tree, a_label, |value| replace(value, "A:".to_string()));
    a_label.set_margin(tree, Thickness::new(1, 1, 0, 1));
    let a = InputLine::new().window(tree, edits, None).unwrap();
    InputLine::set_value_range(tree, a, InputLineValueRange::Float(f64::MIN ..= f64::MAX));
    InputLine::value_mut(tree, a, |value| replace(value, "0".to_string()));
    a.set_margin(tree, Thickness::new(1, 1, 1, 1));

    let v_label = StaticText::new().window(tree, labels, Some(a_label)).unwrap();
    StaticText::text_mut(tree, v_label, |value| replace(value, "V:".to_string()));
    v_label.set_margin(tree, Thickness::new(1, 0, 0, 1));
    let v = InputLine::new().window(tree, edits, Some(a)).unwrap();
    InputLine::set_value_range(tree, v, InputLineValueRange::Float(f64::MIN ..= f64::MAX));
    InputLine::value_mut(tree, v, |value| replace(value, "1".to_string()));
    v.set_margin(tree, Thickness::new(1, 0, 1, 1));

    let t_label = StaticText::new().window(tree, labels, Some(v_label)).unwrap();
    StaticText::text_mut(tree, t_label, |value| replace(value, "T:".to_string()));
    t_label.set_margin(tree, Thickness::new(1, 0, 0, 1));
    let t = InputLine::new().window(tree, edits, Some(v)).unwrap();
    InputLine::set_value_range(tree, t, InputLineValueRange::Float(f64::MIN ..= f64::MAX));
    InputLine::value_mut(tree, t, |value| replace(value, "0".to_string()));
    t.set_margin(tree, Thickness::new(1, 0, 1, 1));

    let n_label = StaticText::new().window(tree, labels, Some(t_label)).unwrap();
    StaticText::text_mut(tree, n_label, |value| replace(value, "N:".to_string()));
    n_label.set_margin(tree, Thickness::new(1, 0, 0, 1));
    let n = InputLine::new().window(tree, edits, Some(t)).unwrap();
    InputLine::set_value_range(tree, n, InputLineValueRange::Integer(1 ..= i64::from(i32::MAX)));
    InputLine::value_mut(tree, n, |value| replace(value, "1".to_string()));
    n.set_margin(tree, Thickness::new(1, 0, 1, 1));

    let calc = Button::new().window(tree, controls, Some(edits_with_labels)).unwrap();
    Button::text_mut(tree, calc, |value| replace(value, "Calculate".to_string()));
    calc.set_h_align(tree, Some(HAlign::Center));

    a.set_next_focus(tree, v);
    v.set_next_focus(tree, t);
    t.set_next_focus(tree, n);
    n.set_next_focus(tree, a);
    calc.focus(tree, &mut ());
    tree.run(&mut ()).unwrap();
}
