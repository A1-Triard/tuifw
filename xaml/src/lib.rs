#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::match_ref_pats)]
#![allow(clippy::type_complexity)]

pub mod xaml;

pub mod preprocessor;

use indent::indent_all_by;
use indoc::indoc;
use std::str::FromStr;
use xaml::*;

pub const XMLNS: &str = "https://a1-triard.github.io/tuifw/2023/xaml";

pub fn reg_widgets(xaml: &mut Xaml) {
    let boolean = XamlLiteral::new(xaml, XMLNS, "Bool");
    let string = XamlLiteral::new(xaml, XMLNS, "String");
    let int_16 = XamlLiteral::new(xaml, XMLNS, "I16");
    let uint_16 = XamlLiteral::new(xaml, XMLNS, "U16");
    let int_32 = XamlLiteral::new(xaml, XMLNS, "I32");
    let float_64 = XamlLiteral::new(xaml, XMLNS, "F64");
    let thickness = XamlLiteral::new(xaml, XMLNS, "Thickness");
    let h_align = XamlLiteral::new(xaml, XMLNS, "HAlign");
    let v_align = XamlLiteral::new(xaml, XMLNS, "VAlign");
    let dock = XamlLiteral::new(xaml, XMLNS, "Dock");
    let visibility = XamlLiteral::new(xaml, XMLNS, "Visibility");

    let validator = XamlStruct::new(xaml, None, XMLNS, "Validator");

    let int_validator = XamlStruct::new(xaml, Some(validator), XMLNS, "IntValidator");
    let int_validator_min = XamlProperty::new(
        xaml, int_validator, "Min", XamlType::Literal(int_32), false, false
    );
    let int_validator_max = XamlProperty::new(
        xaml, int_validator, "Max", XamlType::Literal(int_32), false, false
    );

    let float_validator = XamlStruct::new(xaml, Some(validator), XMLNS, "FloatValidator");
    let float_validator_min = XamlProperty::new(
        xaml, float_validator, "Min", XamlType::Literal(float_64), false, false
    );
    let float_validator_max = XamlProperty::new(
        xaml, float_validator, "Max", XamlType::Literal(float_64), false, false
    );

    let widget = XamlStruct::new(xaml, None, XMLNS, "Widget");
    let widget_children = XamlProperty::new(xaml, widget, "Children", XamlType::Struct(widget), true, false);
    let widget_name = XamlProperty::new(xaml, widget, "Name", XamlType::Literal(string), false, true);
    let widget_focus_tab = XamlProperty::new(xaml, widget, "FocusTab", XamlType::Ref, false, false);
    let widget_focus_right = XamlProperty::new(xaml, widget, "FocusRight", XamlType::Ref, false, false);
    let widget_focus_left = XamlProperty::new(xaml, widget, "FocusLeft", XamlType::Ref, false, false);
    let widget_focus_up = XamlProperty::new(xaml, widget, "FocusUp", XamlType::Ref, false, false);
    let widget_focus_down = XamlProperty::new(xaml, widget, "FocusDown", XamlType::Ref, false, false);
    let widget_focused_primary = XamlProperty::new(
        xaml, widget, "FocusedPrimary", XamlType::Literal(boolean), false, false
    );
    let widget_focused_secondary = XamlProperty::new(
        xaml, widget, "FocusedSecondary", XamlType::Literal(boolean), false, false
    );
    let widget_h_align = XamlProperty::new(xaml, widget, "HAlign", XamlType::Literal(h_align), false, false);
    let widget_v_align = XamlProperty::new(xaml, widget, "VAlign", XamlType::Literal(v_align), false, false);
    let widget_width = XamlProperty::new(xaml, widget, "Width", XamlType::Literal(int_16), false, false);
    let widget_height = XamlProperty::new(xaml, widget, "Height", XamlType::Literal(int_16), false, false);
    let widget_margin = XamlProperty::new(xaml, widget, "Margin", XamlType::Literal(thickness), false, false);
    let widget_min_width = XamlProperty::new(xaml, widget, "MinWidth", XamlType::Literal(int_16), false, false);
    let widget_max_width = XamlProperty::new(xaml, widget, "MaxWidth", XamlType::Literal(int_16), false, false);
    let widget_min_height = XamlProperty::new(
        xaml, widget, "MinHeight", XamlType::Literal(int_16), false, false
    );
    let widget_max_height = XamlProperty::new(
        xaml, widget, "MaxHeight", XamlType::Literal(int_16), false, false
    );
    let widget_is_enabled = XamlProperty::new(
        xaml, widget, "IsEnabled", XamlType::Literal(boolean), false, false
    );
    let widget_visibility = XamlProperty::new(
        xaml, widget, "Visibility", XamlType::Literal(visibility), false, false
    );

    let background = XamlStruct::new(xaml, Some(widget), XMLNS, "Background");
    let background_show_pattern = XamlProperty::new(
        xaml, background, "ShowPattern", XamlType::Literal(boolean), false, false
    );
    let background_pattern_even = XamlProperty::new(
        xaml, background, "PatternEven", XamlType::Literal(string), false, false
    );
    let background_pattern_odd = XamlProperty::new(
        xaml, background, "PatternOdd", XamlType::Literal(string), false, false
    );

    let stack_panel = XamlStruct::new(xaml, Some(widget), XMLNS, "StackPanel");
    let stack_panel_vertical = XamlProperty::new(
        xaml, stack_panel, "Vertical", XamlType::Literal(boolean), false, false
    );

    let dock_panel = XamlStruct::new(xaml, Some(widget), XMLNS, "DockPanel");
    let widget_dock = XamlProperty::new(xaml, widget, "Dock", XamlType::Literal(dock), false, false);

    let static_text = XamlStruct::new(xaml, Some(widget), XMLNS, "StaticText");
    let static_text_text = XamlProperty::new(
        xaml, static_text, "Text", XamlType::Literal(string), false, false
    );

    let button = XamlStruct::new(xaml, Some(widget), XMLNS, "Button");
    let button_text = XamlProperty::new(xaml, button, "Text", XamlType::Literal(string), false, false);

    let input_line = XamlStruct::new(xaml, Some(widget), XMLNS, "InputLine");
    let input_line_text = XamlProperty::new(xaml, input_line, "Text", XamlType::Literal(string), false, false);
    let input_line_validator = XamlProperty::new(
        xaml, input_line, "Validator", XamlType::Struct(validator), false, false
    );

    let frame = XamlStruct::new(xaml, Some(widget), XMLNS, "Frame");
    let frame_double = XamlProperty::new(xaml, frame, "Double", XamlType::Literal(boolean), false, false);
    let frame_text = XamlProperty::new(xaml, frame, "Text", XamlType::Literal(string), false, false);
    let frame_text_align = XamlProperty::new(
        xaml, frame, "TextAlign", XamlType::Literal(h_align), false, false
    );

    let label = XamlStruct::new(xaml, Some(widget), XMLNS, "Label");
    let label_text = XamlProperty::new(xaml, label, "Text", XamlType::Literal(string), false, false);
    let label_focus = XamlProperty::new(xaml, label, "Focus", XamlType::Ref, false, false);

    let check_box = XamlStruct::new(xaml, Some(widget), XMLNS, "CheckBox");
    let check_box_text = XamlProperty::new(xaml, check_box, "Text", XamlType::Literal(string), false, false);
    let check_box_is_on = XamlProperty::new(xaml, check_box, "IsOn", XamlType::Literal(boolean), false, false);

    let radio_button = XamlStruct::new(xaml, Some(widget), XMLNS, "RadioButton");
    let radio_button_text = XamlProperty::new(
        xaml, radio_button, "Text", XamlType::Literal(string), false, false
    );
    let radio_button_is_on = XamlProperty::new(
        xaml, radio_button, "IsOn", XamlType::Literal(boolean), false, false
    );

    boolean.set_ctor(xaml, Some(Box::new(|x| match x {
        "True" => Some("true".to_string()),
        "False" => Some("false".to_string()),
        _ => None,
    })));
    string.set_ctor(xaml, Some(Box::new(|x| Some(format!("\"{}\"", x.escape_debug())))));
    int_16.set_ctor(xaml, Some(Box::new(|x| i16::from_str(x).ok().map(|x| x.to_string()))));
    uint_16.set_ctor(xaml, Some(Box::new(|x| u16::from_str(x).ok().map(|x| x.to_string()))));
    int_32.set_ctor(xaml, Some(Box::new(|x| i32::from_str(x).ok().map(|x| x.to_string()))));
    float_64.set_ctor(xaml, Some(Box::new(|x| f64::from_str(x).ok().map(|x| x.to_string()))));
    thickness.set_ctor(xaml, Some(Box::new(|x| {
        let parts = x.split(',').collect::<Vec<_>>();
        if parts.len() == 4 {
            let l = i32::from_str(parts[0]).ok()?;
            let t = i32::from_str(parts[1]).ok()?;
            let r = i32::from_str(parts[2]).ok()?;
            let b = i32::from_str(parts[3]).ok()?;
            if l < -i32::from(u16::MAX) || l > i32::from(u16::MAX) { return None; }
            if t < -i32::from(u16::MAX) || t > i32::from(u16::MAX) { return None; }
            if r < -i32::from(u16::MAX) || r > i32::from(u16::MAX) { return None; }
            if b < -i32::from(u16::MAX) || b > i32::from(u16::MAX) { return None; }
            Some(format!("Thickness::new({l}, {t}, {r}, {b})"))
        } else if parts.len() == 1 {
            let a = i32::from_str(parts[0]).ok()?;
            if a < -i32::from(u16::MAX) || a > i32::from(u16::MAX) { return None; }
            Some(format!("Thickness::all({a})"))
        } else {
            None
        }
    })));
    h_align.set_ctor(xaml, Some(Box::new(|x| match x {
        "Left" => Some("HAlign::Left".to_string()),
        "Center" => Some("HAlign::Center".to_string()),
        "Right" => Some("HAlign::Right".to_string()),
        _ => None,
    })));
    v_align.set_ctor(xaml, Some(Box::new(|x| match x {
        "Top" => Some("VAlign::Top".to_string()),
        "Center" => Some("VAlign::Center".to_string()),
        "Bottom" => Some("VAlign::Bottom".to_string()),
        _ => None,
    })));
    dock.set_ctor(xaml, Some(Box::new(|x| match x {
        "Left" => Some("Dock::Left".to_string()),
        "Top" => Some("Dock::Top".to_string()),
        "Right" => Some("Dock::Right".to_string()),
        "Bottom" => Some("Dock::Bottom".to_string()),
        _ => None,
    })));
    visibility.set_ctor(xaml, Some(Box::new(|x| match x {
        "Visible" => Some("Visibility::Visible".to_string()),
        "Hidden" => Some("Visibility::Hidden".to_string()),
        "Collapsed" => Some("Visibility::Collapsed".to_string()),
        _ => None,
    })));

    xaml.set_preamble(indoc! { "
        extern crate alloc;

        #[allow(unused_imports)]
        use alloc::boxed::Box;
        use tuifw::*;
        use tuifw_screen::*;
        use tuifw_window::*;
    " });
    xaml.set_header(indoc! { "

        pub fn build(
            tree: &mut WindowTree,
        ) -> Result<Names, Error> {
    " });
    xaml.set_result(Box::new(|_, names| {
        let mut s = "    let names = Names {\n".to_string();
        for (name, obj) in names {
            s.push_str("        ");
            s.push_str(name);
            s.push_str(": ");
            s.push_str(obj);
            s.push_str(",\n");
        }
        s.push_str("    };\n    Ok(names)\n");
        s
    }));
    xaml.set_footer(indoc! {"
        }
    " });
    xaml.set_postamble(Box::new(|names| {
        let mut s = "\npub struct Names {\n".to_string();
        for name in names.keys() {
            s.push_str("    #[allow(dead_code)]\n    pub ");
            s.push_str(name);
            s.push_str(": Window,\n");
        }
        s.push_str("}\n");
        s
    }));

    int_validator.set_ctor(xaml, Some(Box::new(|obj, _parent, _prev| {
        indent_all_by(4, format!(indoc! { "
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            let mut {} = IntValidator {{ min: i32::MIN, max: i32::MAX }};
        " }, obj))
    })));
    int_validator_min.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.min = {};
    " }, obj, value))));
    int_validator_max.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.max = {};
    " }, obj, value))));

    float_validator.set_ctor(xaml, Some(Box::new(|obj, _parent, _prev| {
        indent_all_by(4, format!(indoc! { "
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            let mut {} = FloatValidator {{ min: f64::MIN, max: f64::MAX }};
        " }, obj))
    })));
    float_validator_min.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.min = {};
    " }, obj, value))));
    float_validator_max.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.max = {};
    " }, obj, value))));

    widget_children.set_setter(xaml, Box::new(|_obj, _value| String::new()));
    widget_is_enabled.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_is_enabled(tree, {});
    " }, obj, value))));
    widget_visibility.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_visibility(tree, {});
    " }, obj, value))));
    widget_name.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_name(tree, {});
    " }, obj, value))));
    widget_focus_tab.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_focus_tab(tree, {});
    " }, obj, value))));
    widget_focus_right.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_focus_right(tree, {});
    " }, obj, value))));
    widget_focus_left.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_focus_left(tree, {});
    " }, obj, value))));
    widget_focus_up.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_focus_up(tree, {});
    " }, obj, value))));
    widget_focus_down.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_focus_down(tree, {});
    " }, obj, value))));
    widget_focused_primary.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_focused_primary(tree, {});
    " }, obj, value))));
    widget_focused_secondary.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_focused_secondary(tree, {});
    " }, obj, value))));
    widget_h_align.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_h_align(tree, Some({}));
    " }, obj, value))));
    widget_v_align.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_v_align(tree, Some({}));
    " }, obj, value))));
    widget_width.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_width(tree, Some({}));
    " }, obj, value))));
    widget_height.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_height(tree, Some({}));
    " }, obj, value))));
    widget_margin.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_margin(tree, {});
    " }, obj, value))));
    widget_min_width.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_min_width(tree, {});
    " }, obj, value))));
    widget_min_height.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_min_height(tree, {});
    " }, obj, value))));
    widget_max_width.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_max_width(tree, {});
    " }, obj, value))));
    widget_max_height.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_max_height(tree, {});
    " }, obj, value))));

    background.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Background::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Background::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = Background::new(tree, None, None)?;
            " }, obj))
        }
    })));
    background_show_pattern.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Background::set_show_pattern(tree, {}, {});
    " }, obj, value))));
    background_pattern_even.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Background::set_pattern_even(tree, {}, {});
    " }, obj, value))));
    background_pattern_odd.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Background::set_pattern_odd(tree, {}, {});
    " }, obj, value))));

    stack_panel.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = StackPanel::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = StackPanel::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = StackPanel::new(tree, None, None)?;
            " }, obj))
        }
    })));
    stack_panel_vertical.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        StackPanel::set_vertical(tree, {}, {});
    " }, obj, value))));

    dock_panel.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = DockPanel::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = DockPanel::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = DockPanel::new(tree, None, None)?;
            " }, obj))
        }
    })));
    widget_dock.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        DockPanel::set_dock(tree, {}, Some({}));
    " }, obj, value))));

    static_text.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = StaticText::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = StaticText::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = StaticText::new(tree, None, None)?;
            " }, obj))
        }
    })));
    static_text_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        StaticText::set_text(tree, {}, {});
    " }, obj, value))));

    button.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Button::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Button::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = Button::new(tree, None, None)?;
            " }, obj))
        }
    })));
    button_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Button::set_text(tree, {}, {});
    " }, obj, value))));

    input_line.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = InputLine::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = InputLine::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = InputLine::new(tree, None, None)?;
            " }, obj))
        }
    })));
    input_line_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        InputLine::set_text(tree, {}, {});
    " }, obj, value))));
    input_line_validator.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        InputLine::set_validator(tree, {}, Some(Box::new({})));
    " }, obj, value))));

    frame.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Frame::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Frame::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = Frame::new(tree, None, None)?;
            " }, obj))
        }
    })));
    frame_double.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Frame::set_double(tree, {}, {});
    " }, obj, value))));
    frame_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Frame::set_text(tree, {}, {});
    " }, obj, value))));
    frame_text_align.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Frame::set_text_align(tree, {}, {});
    " }, obj, value))));

    label.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Label::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Label::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = Label::new(tree, None, None)?;
            " }, obj))
        }
    })));
    label_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Label::set_text(tree, {}, {});
    " }, obj, value))));
    label_focus.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Label::set_focus(tree, {}, Some({}));
    " }, obj, value))));

    check_box.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = CheckBox::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = CheckBox::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = CheckBox::new(tree, None, None)?;
            " }, obj))
        }
    })));
    check_box_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        CheckBox::set_text(tree, {}, {});
    " }, obj, value))));
    check_box_is_on.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        CheckBox::set_is_on(tree, {}, {});
    " }, obj, value))));

    radio_button.set_ctor(xaml, Some(Box::new(|obj, parent, prev| {
        if let Some((parent, _parent_property)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = RadioButton::new(tree, Some({}), Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = RadioButton::new(tree, Some({}), None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = RadioButton::new(tree, None, None)?;
            " }, obj))
        }
    })));
    radio_button_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        RadioButton::set_text(tree, {}, {});
    " }, obj, value))));
    radio_button_is_on.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        RadioButton::set_is_on(tree, {}, {});
    " }, obj, value))));
}
