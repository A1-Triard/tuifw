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

pub fn set_widget_ctor(
    xaml: &mut Xaml,
    widget: XamlStruct,
    widget_name: &'static str,
    widget_children: XamlProperty,
) {
    widget.set_ctor(xaml, Some(Box::new(move |obj, parent, prev| {
        if let Some((parent, parent_property)) = parent {
            if parent_property == widget_children {
                if let Some(prev) = prev {
                    indent_all_by(4, format!(indoc! { "
                        #[allow(unused_variables)]
                        let {} = {}::new(tree, Some({}), Some({}))?;
                    " }, obj, widget_name, parent, prev))
                } else {
                    indent_all_by(4, format!(indoc! { "
                        #[allow(unused_variables)]
                        let {} = {}::new(tree, Some({}), None)?;
                    " }, obj, widget_name, parent))
                }
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = {}::new_template(tree)?;
                " }, obj, widget_name))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_variables)]
                let {} = {}::new(tree, None, None)?;
            " }, obj, widget_name))
        }
    })));
}

pub struct Registered {
    pub boolean: XamlLiteral,
    pub string: XamlLiteral,
    pub int_16: XamlLiteral,
    pub uint_16: XamlLiteral,
    pub int_32: XamlLiteral,
    pub float_64: XamlLiteral,
    pub thickness: XamlLiteral,
    pub point: XamlLiteral,
    pub h_align: XamlLiteral,
    pub v_align: XamlLiteral,
    pub dock: XamlLiteral,
    pub visibility: XamlLiteral,

    pub validator: XamlStruct,

    pub int_validator: XamlStruct,
    pub int_validator_min: XamlProperty,
    pub int_validator_max: XamlProperty,

    pub float_validator: XamlStruct,
    pub float_validator_min: XamlProperty,
    pub float_validator_max: XamlProperty,

    pub widget: XamlStruct,
    pub widget_children: XamlProperty,
    pub widget_name: XamlProperty,
    pub widget_focus_tab: XamlProperty,
    pub widget_focus_right: XamlProperty,
    pub widget_focus_left: XamlProperty,
    pub widget_focus_up: XamlProperty,
    pub widget_focus_down: XamlProperty,
    pub widget_focused_primary: XamlProperty,
    pub widget_focused_secondary: XamlProperty,
    pub widget_h_align: XamlProperty,
    pub widget_v_align: XamlProperty,
    pub widget_width: XamlProperty,
    pub widget_height: XamlProperty,
    pub widget_margin: XamlProperty,
    pub widget_min_width: XamlProperty,
    pub widget_max_width: XamlProperty,
    pub widget_min_height: XamlProperty,
    pub widget_max_height: XamlProperty,
    pub widget_is_enabled: XamlProperty,
    pub widget_visibility: XamlProperty,

    pub background: XamlStruct,
    pub background_show_pattern: XamlProperty,
    pub background_pattern_even: XamlProperty,
    pub background_pattern_odd: XamlProperty,

    pub stack_panel: XamlStruct,
    pub stack_panel_vertical: XamlProperty,

    pub dock_panel: XamlStruct,
    pub widget_dock: XamlProperty,

    pub canvas: XamlStruct,
    pub widget_tl: XamlProperty,

    pub static_text: XamlStruct,
    pub static_text_text: XamlProperty,

    pub button: XamlStruct,
    pub button_text: XamlProperty,

    pub input_line: XamlStruct,
    pub input_line_text: XamlProperty,
    pub input_line_validator: XamlProperty,

    pub frame: XamlStruct,
    pub frame_double: XamlProperty,
    pub frame_text: XamlProperty,
    pub frame_text_align: XamlProperty,

    pub label: XamlStruct,
    pub label_text: XamlProperty,
    pub label_focus: XamlProperty,

    pub check_box: XamlStruct,
    pub check_box_text: XamlProperty,
    pub check_box_is_on: XamlProperty,

    pub radio_button: XamlStruct,
    pub radio_button_text: XamlProperty,
    pub radio_button_is_on: XamlProperty,

    pub content_presenter: XamlStruct,
    pub content_presenter_content_template: XamlProperty,

    pub items_presenter: XamlStruct,
    pub items_presenter_panel_template: XamlProperty,
    pub items_presenter_item_template: XamlProperty,
    pub items_presenter_tab_navigation: XamlProperty,

    pub virt_items_presenter: XamlStruct,
    pub virt_items_presenter_item_template: XamlProperty,
    pub virt_items_presenter_tab_navigation: XamlProperty,
}

pub fn reg_widgets(xaml: &mut Xaml) -> Registered {
    let boolean = XamlLiteral::new(xaml, XMLNS, "Bool");
    let string = XamlLiteral::new(xaml, XMLNS, "String");
    let int_16 = XamlLiteral::new(xaml, XMLNS, "I16");
    let uint_16 = XamlLiteral::new(xaml, XMLNS, "U16");
    let int_32 = XamlLiteral::new(xaml, XMLNS, "I32");
    let float_64 = XamlLiteral::new(xaml, XMLNS, "F64");
    let thickness = XamlLiteral::new(xaml, XMLNS, "Thickness");
    let point = XamlLiteral::new(xaml, XMLNS, "Point");
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

    let canvas = XamlStruct::new(xaml, Some(widget), XMLNS, "Canvas");
    let widget_tl = XamlProperty::new(xaml, widget, "Tl", XamlType::Literal(point), false, false);

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

    let content_presenter = XamlStruct::new(xaml, Some(widget), XMLNS, "ContentPresenter");
    let content_presenter_content_template = XamlProperty::new(
        xaml, content_presenter, "ContentTemplate", XamlType::Struct(widget), true, false
    );

    let items_presenter = XamlStruct::new(xaml, Some(widget), XMLNS, "ItemsPresenter");
    let items_presenter_panel_template = XamlProperty::new(
        xaml, items_presenter, "PanelTemplate", XamlType::Struct(widget), false, false
    );
    let items_presenter_item_template = XamlProperty::new(
        xaml, items_presenter, "ItemTemplate", XamlType::Struct(widget), true, false
    );
    let items_presenter_tab_navigation = XamlProperty::new(
        xaml, items_presenter, "TabNavigation", XamlType::Literal(boolean), false, false
    );

    let virt_items_presenter = XamlStruct::new(xaml, Some(widget), XMLNS, "VirtItemsPresenter");
    let virt_items_presenter_item_template = XamlProperty::new(
        xaml, virt_items_presenter, "ItemTemplate", XamlType::Struct(widget), true, false
    );
    let virt_items_presenter_tab_navigation = XamlProperty::new(
        xaml, virt_items_presenter, "TabNavigation", XamlType::Literal(boolean), false, false
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
            Some(format!("tuifw_screen_base::Thickness::new({l}, {t}, {r}, {b})"))
        } else if parts.len() == 1 {
            let a = i32::from_str(parts[0]).ok()?;
            if a < -i32::from(u16::MAX) || a > i32::from(u16::MAX) { return None; }
            Some(format!("tuifw_screen_base::Thickness::all({a})"))
        } else {
            None
        }
    })));
    h_align.set_ctor(xaml, Some(Box::new(|x| match x {
        "Left" => Some("tuifw_screen_base::HAlign::Left".to_string()),
        "Center" => Some("tuifw_screen_base::HAlign::Center".to_string()),
        "Right" => Some("tuifw_screen_base::HAlign::Right".to_string()),
        _ => None,
    })));
    v_align.set_ctor(xaml, Some(Box::new(|x| match x {
        "Top" => Some("tuifw_screen_base::VAlign::Top".to_string()),
        "Center" => Some("tuifw_screen_base::VAlign::Center".to_string()),
        "Bottom" => Some("tuifw_screen_base::VAlign::Bottom".to_string()),
        _ => None,
    })));
    dock.set_ctor(xaml, Some(Box::new(|x| match x {
        "Left" => Some("tuifw::Dock::Left".to_string()),
        "Top" => Some("tuifw::Dock::Top".to_string()),
        "Right" => Some("tuifw::Dock::Right".to_string()),
        "Bottom" => Some("tuifw::Dock::Bottom".to_string()),
        _ => None,
    })));
    visibility.set_ctor(xaml, Some(Box::new(|x| match x {
        "Visible" => Some("tuifw_window::Visibility::Visible".to_string()),
        "Hidden" => Some("tuifw_window::Visibility::Hidden".to_string()),
        "Collapsed" => Some("tuifw_window::Visibility::Collapsed".to_string()),
        _ => None,
    })));
    point.set_ctor(xaml, Some(Box::new(|x| {
        let parts = x.split(',').collect::<Vec<_>>();
        if parts.len() == 2 {
            let x = i16::from_str(parts[0]).ok()?;
            let y = i16::from_str(parts[1]).ok()?;
            Some(format!("tuifw_screen_base::Point {{ x: {x}, y: {y} }}"))
        } else {
            None
        }
    })));

    xaml.append_preamble(indoc! { "
        extern crate alloc;
    " });
    xaml.set_header(indoc! { "

        pub fn build(
            tree: &mut tuifw_window::WindowTree,
        ) -> Result<Names, tuifw_screen_base::Error> {
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
            s.push_str(": tuifw_window::Window,\n");
        }
        s.push_str("}\n");
        s
    }));

    int_validator.set_ctor(xaml, Some(Box::new(|obj, _parent, _prev| {
        indent_all_by(4, format!(indoc! { "
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            let mut {} = tuifw::IntValidator {{ min: i32::MIN, max: i32::MAX }};
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
            let mut {} = tuifw::FloatValidator {{ min: f64::MIN, max: f64::MAX }};
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

    set_widget_ctor(xaml, background, "tuifw::Background", widget_children);
    background_show_pattern.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Background::set_show_pattern(tree, {}, {});
    " }, obj, value))));
    background_pattern_even.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Background::set_pattern_even(tree, {}, {});
    " }, obj, value))));
    background_pattern_odd.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Background::set_pattern_odd(tree, {}, {});
    " }, obj, value))));

    set_widget_ctor(xaml, stack_panel, "tuifw::StackPanel", widget_children);
    stack_panel_vertical.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::StackPanel::set_vertical(tree, {}, {});
    " }, obj, value))));

    set_widget_ctor(xaml, dock_panel, "tuifw::DockPanel", widget_children);
    widget_dock.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::DockPanel::set_dock(tree, {}, Some({}));
    " }, obj, value))));

    set_widget_ctor(xaml, canvas, "tuifw::Canvas", widget_children);
    widget_tl.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Canvas::set_tl(tree, {}, {});
    " }, obj, value))));

    set_widget_ctor(xaml, static_text, "tuifw::StaticText", widget_children);
    static_text_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::StaticText::set_text(tree, {}, {});
    " }, obj, value))));

    set_widget_ctor(xaml, button, "tuifw::Button", widget_children);
    button_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Button::set_text(tree, {}, {});
    " }, obj, value))));

    set_widget_ctor(xaml, input_line, "tuifw::InputLine", widget_children);
    input_line_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::InputLine::set_text(tree, {}, {});
    " }, obj, value))));
    input_line_validator.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::InputLine::set_validator(tree, {}, Some(alloc::boxed::Box::new({})));
    " }, obj, value))));

    set_widget_ctor(xaml, frame, "tuifw::Frame", widget_children);
    frame_double.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Frame::set_double(tree, {}, {});
    " }, obj, value))));
    frame_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Frame::set_text(tree, {}, {});
    " }, obj, value))));
    frame_text_align.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Frame::set_text_align(tree, {}, {});
    " }, obj, value))));

    set_widget_ctor(xaml, label, "tuifw::Label", widget_children);
    label_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Label::set_text(tree, {}, {});
    " }, obj, value))));
    label_focus.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::Label::set_focus(tree, {}, Some({}));
    " }, obj, value))));

    set_widget_ctor(xaml, check_box, "tuifw::CheckBox", widget_children);
    check_box_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::CheckBox::set_text(tree, {}, {});
    " }, obj, value))));
    check_box_is_on.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::CheckBox::set_is_on(tree, {}, {});
    " }, obj, value))));

    set_widget_ctor(xaml, radio_button, "tuifw::RadioButton", widget_children);
    radio_button_text.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::RadioButton::set_text(tree, {}, {});
    " }, obj, value))));
    radio_button_is_on.set_setter(xaml, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        tuifw::RadioButton::set_is_on(tree, {}, {});
    " }, obj, value))));

    set_widget_ctor(xaml, content_presenter, "tuifw::ContentPresenter", widget_children);
    content_presenter_content_template.set_setter(
        xaml,
        Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
            tuifw::ContentPresenter::set_content_template(tree, {}, Some({}));
        " }, obj, value)))
    );

    set_widget_ctor(xaml, items_presenter, "tuifw::ItemsPresenter", widget_children);
    items_presenter_panel_template.set_setter(
        xaml,
        Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
            tuifw::ItemsPresenter::set_panel_template(tree, {}, Some({}));
        " }, obj, value)))
    );
    items_presenter_item_template.set_setter(
        xaml,
        Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
            tuifw::ItemsPresenter::set_item_template(tree, {}, Some({}));
        " }, obj, value)))
    );
    items_presenter_tab_navigation.set_setter(
        xaml,
        Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
            tuifw::ItemsPresenter::set_tab_navigation(tree, {}, {});
        " }, obj, value)))
    );

    set_widget_ctor(xaml, virt_items_presenter, "tuifw::VirtItemsPresenter", widget_children);
    virt_items_presenter_item_template.set_setter(
        xaml,
        Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
            tuifw::VirtItemsPresenter::set_item_template(tree, {}, Some({}));
        " }, obj, value)))
    );
    virt_items_presenter_tab_navigation.set_setter(
        xaml,
        Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
            tuifw::VirtItemsPresenter::set_tab_navigation(tree, {}, {});
        " }, obj, value)))
    );

    Registered {
        boolean,
        string,
        int_16,
        uint_16,
        int_32,
        float_64,
        thickness,
        point,
        h_align,
        v_align,
        dock,
        visibility,

        validator,

        int_validator,
        int_validator_min,
        int_validator_max,

        float_validator,
        float_validator_min,
        float_validator_max,

        widget,
        widget_children,
        widget_name,
        widget_focus_tab,
        widget_focus_right,
        widget_focus_left,
        widget_focus_up,
        widget_focus_down,
        widget_focused_primary,
        widget_focused_secondary,
        widget_h_align,
        widget_v_align,
        widget_width,
        widget_height,
        widget_margin,
        widget_min_width,
        widget_max_width,
        widget_min_height,
        widget_max_height,
        widget_is_enabled,
        widget_visibility,

        background,
        background_show_pattern,
        background_pattern_even,
        background_pattern_odd,

        stack_panel,
        stack_panel_vertical,

        dock_panel,
        widget_dock,

        canvas,
        widget_tl,

        static_text,
        static_text_text,

        button,
        button_text,

        input_line,
        input_line_text,
        input_line_validator,

        frame,
        frame_double,
        frame_text,
        frame_text_align,

        label,
        label_text,
        label_focus,

        check_box,
        check_box_text,
        check_box_is_on,

        radio_button,
        radio_button_text,
        radio_button_is_on,

        content_presenter,
        content_presenter_content_template,

        items_presenter,
        items_presenter_panel_template,
        items_presenter_item_template,
        items_presenter_tab_navigation,

        virt_items_presenter,
        virt_items_presenter_item_template,
        virt_items_presenter_tab_navigation,
    }
}
