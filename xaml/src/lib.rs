#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::match_ref_pats)]
#![allow(clippy::type_complexity)]

pub mod xaml;

use indent::indent_all_by;
use indoc::indoc;
use std::str::FromStr;
use xaml::*;

macro_rules! xmlns {
    (
        $s:literal
    ) => {
        concat!("{https://a1-triard.github.io/tuifw/2023/xaml}", $s)
    };
}

pub fn reg_widgets(xaml: &mut Xaml) {
    let boolean = xaml.reg_literal(xmlns!("Bool"));
    let string = xaml.reg_literal(xmlns!("String"));
    let int_16 = xaml.reg_literal(xmlns!("I16"));
    let uint_16 = xaml.reg_literal(xmlns!("U16"));
    let int_32 = xaml.reg_literal(xmlns!("I32"));
    let float_64 = xaml.reg_literal(xmlns!("F64"));
    let thickness = xaml.reg_literal(xmlns!("Thickness"));
    let h_align = xaml.reg_literal(xmlns!("HAlign"));
    let v_align = xaml.reg_literal(xmlns!("VAlign"));
    let dock = xaml.reg_literal(xmlns!("Dock"));
    let validator = xaml.reg_struct(xmlns!("Validator"), None);
    let int_validator = xaml.reg_struct(xmlns!("IntValidator"), Some(validator));
    let int_validator_min = xaml.reg_prop(int_validator, "Min", XamlType::Literal(int_32));
    let int_validator_max = xaml.reg_prop(int_validator, "Max", XamlType::Literal(int_32));
    let float_validator = xaml.reg_struct(xmlns!("FloatValidator"), Some(validator));
    let float_validator_min = xaml.reg_prop(float_validator, "Min", XamlType::Literal(float_64));
    let float_validator_max = xaml.reg_prop(float_validator, "Max", XamlType::Literal(float_64));
    let widget = xaml.reg_struct(xmlns!("Widget"), None);
    let widget_children = xaml.reg_prop(widget, "Children", XamlType::Struct(widget));
    xaml.set_as_content_prop(widget_children);
    let widget_tag = xaml.reg_prop(widget, "Tag", XamlType::Literal(uint_16));
    let widget_h_align = xaml.reg_prop(widget, "HAlign", XamlType::Literal(h_align));
    let widget_v_align = xaml.reg_prop(widget, "VAlign", XamlType::Literal(v_align));
    let widget_width = xaml.reg_prop(widget, "Width", XamlType::Literal(int_16));
    let widget_margin = xaml.reg_prop(widget, "Margin", XamlType::Literal(thickness));
    let background = xaml.reg_struct(xmlns!("Background"), Some(widget));
    let background_show_pattern = xaml.reg_prop(background, "ShowPattern", XamlType::Literal(boolean));
    let background_pattern_even = xaml.reg_prop(background, "PatternEven", XamlType::Literal(string));
    let background_pattern_odd = xaml.reg_prop(background, "PatternOdd", XamlType::Literal(string));
    let stack_panel = xaml.reg_struct(xmlns!("StackPanel"), Some(widget));
    let dock_panel = xaml.reg_struct(xmlns!("DockPanel"), Some(widget));
    let widget_dock = xaml.reg_prop(widget, "Dock", XamlType::Literal(dock));
    let static_text = xaml.reg_struct(xmlns!("StaticText"), Some(widget));
    let static_text_text = xaml.reg_prop(static_text, "Text", XamlType::Literal(string));
    let input_line = xaml.reg_struct(xmlns!("InputLine"), Some(widget));
    let input_line_text = xaml.reg_prop(input_line, "Text", XamlType::Literal(string));
    let input_line_validator = xaml.reg_prop(input_line, "Validator", XamlType::Struct(validator));
    xaml.set_literal_new(boolean, Box::new(|x| match x {
        "True" => Some("true".to_string()),
        "False" => Some("false".to_string()),
        _ => None,
    }));
    xaml.set_literal_new(string, Box::new(|x| Some(format!("\"{}\"", x.escape_debug()))));
    xaml.set_literal_new(int_16, Box::new(|x| i16::from_str(x).ok().map(|x| x.to_string())));
    xaml.set_literal_new(uint_16, Box::new(|x| u16::from_str(x).ok().map(|x| x.to_string())));
    xaml.set_literal_new(int_32, Box::new(|x| i32::from_str(x).ok().map(|x| x.to_string())));
    xaml.set_literal_new(float_64, Box::new(|x| f64::from_str(x).ok().map(|x| x.to_string())));
    xaml.set_literal_new(thickness, Box::new(|x| {
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
    }));
    xaml.set_literal_new(h_align, Box::new(|x| match x {
        "Left" => Some("Some(HAlign::Left)".to_string()),
        "Center" => Some("Some(HAlign::Center)".to_string()),
        "Right" => Some("Some(HAlign::Right)".to_string()),
        "Stretch" => Some("None".to_string()),
        _ => None,
    }));
    xaml.set_literal_new(v_align, Box::new(|x| match x {
        "Top" => Some("Some(VAlign::Top)".to_string()),
        "Center" => Some("Some(VAlign::Center)".to_string()),
        "Bottom" => Some("Some(VAlign::Bottom)".to_string()),
        "Stretch" => Some("None".to_string()),
        _ => None,
    }));
    xaml.set_literal_new(dock, Box::new(|x| match x {
        "Left" => Some("Some(Dock::Left)".to_string()),
        "Top" => Some("Some(Dock::Top)".to_string()),
        "Right" => Some("Some(Dock::Right)".to_string()),
        "Bottom" => Some("Some(Dock::Bottom)".to_string()),
        "None" => Some("None".to_string()),
        _ => None,
    }));
    xaml.set_preamble(indoc! { "
        extern crate alloc;

        use alloc::boxed::Box;
        #[allow(unused_imports)]
        use alloc::string::ToString;
        #[allow(unused_imports)]
        use core::mem::replace;
        use timer_no_std::MonoClock;
        use tuifw::*;
        use tuifw_screen::*;
        use tuifw_window::*;

    " });
    xaml.set_header(indoc! { "
        pub fn build_tree<State: ?Sized + 'static>(
            screen: Box<dyn Screen>,
            clock: &MonoClock
        ) -> Result<WindowTree<State>, Error> {
    " });
    xaml.set_footer(indoc! {"
        }
    " });
    xaml.set_res(Box::new(|_| indent_all_by(4, format!(indoc! { "
        Ok(tree)
    " }))));
    xaml.set_struct_new(int_validator, Some(Box::new(|obj, _parent| {
        indent_all_by(4, format!(indoc! { "
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            let mut {} = IntValidator {{ min: i32::MIN, max: i32::MAX }};
        " }, obj))
    })));
    xaml.set_prop_set(int_validator_min, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.min = {};
    " }, obj, value))));
    xaml.set_prop_set(int_validator_max, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.max = {};
    " }, obj, value))));
    xaml.set_struct_new(float_validator, Some(Box::new(|obj, _parent| {
        indent_all_by(4, format!(indoc! { "
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            let mut {} = FloatValidator {{ min: f64::MIN, max: f64::MAX }};
        " }, obj))
    })));
    xaml.set_prop_set(float_validator_min, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.min = {};
    " }, obj, value))));
    xaml.set_prop_set(float_validator_max, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.max = {};
    " }, obj, value))));
    xaml.set_prop_set(widget_children, Box::new(|_obj, _value| String::new()));
    xaml.set_prop_set(widget_tag, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_tag(&mut tree, {});
    " }, obj, value))));
    xaml.set_prop_set(widget_h_align, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_h_align(&mut tree, {});
    " }, obj, value))));
    xaml.set_prop_set(widget_v_align, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_v_align(&mut tree, {});
    " }, obj, value))));
    xaml.set_prop_set(widget_width, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_width(&mut tree, {});
    " }, obj, value))));
    xaml.set_prop_set(widget_margin, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        {}.set_margin(&mut tree, {});
    " }, obj, value))));
    xaml.set_struct_new(background, Some(Box::new(|obj, parent| {
        if let Some((parent, _parent_prop, prev)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Background::new().window(&mut tree, {}, Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = Background::new().window(&mut tree, {}, None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_mut)]
                let mut tree = Background::new().window_tree(screen, clock)?;
                #[allow(unused_variables)]
                let {} = tree.root();
            " }, obj))
        }
    })));
    xaml.set_prop_set(background_show_pattern, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Background::set_show_pattern(&mut tree, {}, {});
    " }, obj, value))));
    xaml.set_prop_set(background_pattern_even, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Background::pattern_even_mut(&mut tree, {}, |value| replace(value, {}.to_string()));
    " }, obj, value))));
    xaml.set_prop_set(background_pattern_odd, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        Background::pattern_odd_mut(&mut tree, {}, |value| replace(value, {}.to_string()));
    " }, obj, value))));
    xaml.set_struct_new(stack_panel, Some(Box::new(|obj, parent| {
        if let Some((parent, _parent_prop, prev)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = StackPanel::new().window(&mut tree, {}, Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = StackPanel::new().window(&mut tree, {}, None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_mut)]
                let mut tree = StackPanel::new().window_tree(screen, clock)?;
                #[allow(unused_variables)]
                let {} = tree.root();
            " }, obj))
        }
    })));
    xaml.set_struct_new(dock_panel, Some(Box::new(|obj, parent| {
        if let Some((parent, _parent_prop, prev)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = DockPanel::new().window(&mut tree, {}, Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = DockPanel::new().window(&mut tree, {}, None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_mut)]
                let mut tree = DockPanel::new().window_tree(screen, clock)?;
                #[allow(unused_variables)]
                let {} = tree.root();
            " }, obj))
        }
    })));
    xaml.set_prop_set(widget_dock, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        DockPanel::set_layout(&mut tree, {}, {});
    " }, obj, value))));
    xaml.set_struct_new(static_text, Some(Box::new(|obj, parent| {
        if let Some((parent, _parent_prop, prev)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = StaticText::new().window(&mut tree, {}, Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = StaticText::new().window(&mut tree, {}, None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_mut)]
                let mut tree = StaticText::new().window_tree(screen, clock)?;
                #[allow(unused_variables)]
                let {} = tree.root();
            " }, obj))
        }
    })));
    xaml.set_prop_set(static_text_text, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        StaticText::text_mut(&mut tree, {}, |value| replace(value, {}.to_string()));
    " }, obj, value))));
    xaml.set_struct_new(input_line, Some(Box::new(|obj, parent| {
        if let Some((parent, _parent_prop, prev)) = parent {
            if let Some(prev) = prev {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = InputLine::new().window(&mut tree, {}, Some({}))?;
                " }, obj, parent, prev))
            } else {
                indent_all_by(4, format!(indoc! { "
                    #[allow(unused_variables)]
                    let {} = InputLine::new().window(&mut tree, {}, None)?;
                " }, obj, parent))
            }
        } else {
            indent_all_by(4, format!(indoc! { "
                #[allow(unused_mut)]
                let mut tree = InputLine::new().window_tree(screen, clock)?;
                #[allow(unused_variables)]
                let {} = tree.root();
            " }, obj))
        }
    })));
    xaml.set_prop_set(input_line_text, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        InputLine::text_mut(&mut tree, {}, |value| replace(value, {}.to_string()));
    " }, obj, value))));
    xaml.set_prop_set(input_line_validator, Box::new(|obj, value| indent_all_by(4, format!(indoc! { "
        InputLine::validator_mut(&mut tree, {}, |value| value.replace(Box::new({})));
    " }, obj, value))));
}
