use crate::xaml::*;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use xml::EventReader;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::reader::Error as xml_Error;
use xml::reader::Result as xml_Result;

pub fn preprocess_xaml_file(xaml: &Xaml, source: impl AsRef<Path>, dest: impl AsRef<Path>) -> xml_Result<()> {
    preprocess_xaml(xaml, || Ok(File::open(source.as_ref())?), File::create(dest.as_ref())?)
}

pub fn preprocess_xaml<R: Read, W: Write>(
    xaml: &Xaml,
    mut source: impl FnMut() -> xml_Result<R>,
    mut dest: W,
) -> xml_Result<()> {
    write!(dest, "{}", xaml.preamble())?;
    write!(dest, "{}", xaml.header())?;
    let source_file = source()?;
    let mut events = EventReader::new(source_file);
    let event = events.next()?;
    let mut processor = XamlProcessor {
        xaml,
        source: events,
        dest,
        event,
        obj_n: 0,
        names: HashMap::new(),
        first_pass: true,
    };
    processor.process()?;
    let source_file = source()?;
    let mut events = EventReader::new(source_file);
    let event = events.next()?;
    processor.source = events;
    processor.event = event;
    processor.obj_n = 0;
    processor.first_pass = false;
    processor.process()?;
    write!(processor.dest, "{}", xaml.footer())?;
    write!(processor.dest, "{}", xaml.postamble(&processor.names))?;
    Ok(())
}

struct XamlProcessor<'a, R: Read, W: Write> {
    xaml: &'a Xaml,
    source: EventReader<R>,
    dest: W,
    event: XmlEvent,
    obj_n: u16,
    names: HashMap<String, String>,
    first_pass: bool,
}

impl<'a, R: Read, W: Write> XamlProcessor<'a, R, W> {
    fn next_event(&mut self) -> xml_Result<()> {
        self.event = self.source.next()?;
        while matches!(&self.event, XmlEvent::Comment(_)) || matches!(&self.event, XmlEvent::Whitespace(_)) {
            self.event = self.source.next()?;
        }
        Ok(())
    }

    fn error<T>(&self, e: impl Into<Cow<'static, str>>) -> xml_Result<T> {
        Err(xml_Error::from((&self.source.position(), e)))
    }

    fn name(name: &OwnedName) -> String {
        if let Some(namespace) = name.namespace.as_ref() {
            format!("{{{namespace}}}{}", name.local_name)
        } else {
            name.local_name.clone()
        }
    }

    fn new_obj_name(&mut self) -> xml_Result<String> {
        self.obj_n = self.obj_n.checked_add(1).map_or_else(|| self.error("too many objects"), Ok)?;
        Ok(format!("obj_{}", self.obj_n))
    }

    fn process(&mut self) -> xml_Result<()> {
        match &self.event {
            XmlEvent::StartDocument { .. } => { },
            _ => return self.error("invalid XML document"),
        }
        self.next_event()?;
        let value = self.process_element(None, None)?;
        match &self.event {
            XmlEvent::EndDocument { .. } => { },
            _ => return self.error("miltiple root records"),
        }
        if !self.first_pass {
            write!(self.dest, "{}", self.xaml.result(&value, &self.names))?;
        }
        Ok(())
    }

    fn process_element(
        &mut self,
        parent: Option<(&str, XamlProperty)>,
        prev: Option<&str>,
    ) -> xml_Result<String> {
        let (name, attributes) = match &self.event {
            XmlEvent::StartElement { name, attributes, .. } => (Self::name(name), attributes.clone()),
            _ => return self.error("element start expected"),
        };
        let Some(ty) = self.xaml.ty(&name) else {
            return self.error(format!("unknown type '{}'", name));
        };
        match ty {
            XamlType::Ref => self.process_literal(None, attributes),
            XamlType::Literal(ty) => self.process_literal(Some(ty), attributes),
            XamlType::Struct(ty) => self.process_struct(ty, attributes, parent, prev),
        }
    }

    fn process_literal(
        &mut self,
        ty: Option<XamlLiteral>,
        attributes: Vec<OwnedAttribute>
    ) -> xml_Result<String> {
        if !attributes.is_empty() {
            return self.error(format!("unexpected attribute '{}'", Self::name(&attributes[0].name)));
        }
        self.next_event()?;
        let res = self.process_literal_value(ty)?.0;
        assert!(matches!(&self.event, XmlEvent::EndElement { .. }));
        self.next_event()?;
        Ok(res)
    }

    fn process_literal_value(&mut self, ty: Option<XamlLiteral>) -> xml_Result<(String, String)> {
        let value = match self.event.clone() {
            XmlEvent::Characters(s) => {
                self.next_event()?;
                s
            },
            XmlEvent::Whitespace(s) => {
                self.next_event()?;
                s
            },
            XmlEvent::EndElement { .. } => String::new(),
            _ => return self.error("unsupported XML feature"),
        };
        if let Some(ty) = ty {
            if let Some(processed_value) = ty.instance(self.xaml, &value) {
                Ok((processed_value, value))
            } else {
                self.error(format!("invalid literal '{value}'"))
            }
        } else {
            Ok((String::new(), value))
        }
    }

    fn process_struct(
        &mut self,
        ty: XamlStruct,
        attributes: Vec<OwnedAttribute>,
        parent: Option<(&str, XamlProperty)>,
        prev: Option<&str>,
    ) -> xml_Result<String> {
        let obj = self.new_obj_name()?;
        if self.first_pass {
            if let Some(instance) = ty.instance(self.xaml, &obj, parent, prev) {
                write!(self.dest, "{}", instance)?;
            } else {
                return self.error("cannot create abstract type");
            }
        }
        for attr in attributes {
            let attr_name = Self::name(&attr.name);
            let Some(property) = ty.property(self.xaml, &attr_name) else {
                return self.error(format!("unknown property '{attr_name}'"));
            };
            let is_name_property = Some(property) == ty.name_property(self.xaml);
            if !self.first_pass {
                let value = match property.ty(self.xaml) {
                    XamlType::Struct(_) => return self.error(format!("invalid '{attr_name}' property value")),
                    XamlType::Literal(property_ty) => {
                        let Some(value) = property_ty.instance(self.xaml, &attr.value) else {
                            return self.error(format!("invalid '{attr_name}' property value"));
                        };
                        value
                    },
                    XamlType::Ref => self.names[&attr.value].clone(),
                };
                write!(self.dest, "{}", property.set(self.xaml, &obj, &value))?;
            }
            if is_name_property {
                if attr.value.is_empty() {
                    return self.error("name property value should be a non-empty string");
                }
                self.names.insert(attr.value, obj.clone());
            }
        }
        self.next_event()?;
        loop {
            match &self.event {
                XmlEvent::EndElement { .. } => { self.next_event()?; break; },
                XmlEvent::StartElement { .. } => self.process_property(obj.clone(), ty)?,
                XmlEvent::Whitespace(_) => { self.next_event()? },
                _ => return self.error("unsupported XML feature"),
            }
        }
        Ok(obj)
    }

    fn process_property(&mut self, obj: String, ty: XamlStruct) -> xml_Result<()> {
        let (name, attributes) = match &self.event {
            XmlEvent::StartElement { name, attributes, .. } => (Self::name(name), attributes.clone()),
            _ => unreachable!(),
        };
        let ty_name = ty.name(self.xaml);
        let (property, skip_end_element) = if
            name.starts_with(ty_name) &&
            name.len() > ty_name.len() &&
            name.len() - ty_name.len() >= 2 &&
            name.as_bytes()[ty_name.len()] == b'.'
        {
            self.next_event()?;
            let property_name = &name[ty_name.len() + 1 ..];
            let Some(property) = ty.property(self.xaml, property_name) else {
                return self.error(format!("unknown property '{property_name}'"));
            };
            if !attributes.is_empty() {
                return self.error(format!("unexpected attribute '{}'", Self::name(&attributes[0].name)));
            }
            (property, true)
        } else if let Some(content_property) = ty.content_property(self.xaml) {
            (content_property, false)
        } else {
            return self.error("type does not have content property");
        };
        let mut prev_value = None;
        loop {
            let (value, raw_value) = match &self.event {
                XmlEvent::EndElement { .. } => { break; },
                XmlEvent::StartElement { .. } =>
                    (self.process_element(Some((&obj, property)), prev_value.as_deref())?, String::new()),
                XmlEvent::Characters(_) | XmlEvent::Whitespace(_) => {
                    let XamlType::Literal(property_ty) = property.ty(self.xaml) else {
                        return self.error(format!(
                            "invalid '{}' property value 2",
                            property.name(self.xaml)
                        ));
                    };
                    self.process_literal_value(Some(property_ty))?
                },
                _ => return self.error("unsupported XML feature"),
            };
            if !self.first_pass {
                write!(self.dest, "{}", property.set(self.xaml, &obj, &value))?;
            }
            let is_name_property = Some(property) == ty.name_property(self.xaml);
            if is_name_property {
                if raw_value.is_empty() {
                    return self.error("name property value should be a non-empty string");
                }
                self.names.insert(raw_value, obj.clone());
            }
            prev_value = Some(value);
        }
        if skip_end_element {
            self.next_event()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::{self};

    #[test]
    fn process_literal() {
        let mut xaml = Xaml::new();
        let t = XamlLiteral::new(&mut xaml, "https://a1-triard.github.io/tuifw/2023/xaml", "Bool");
        xaml.set_result(Box::new(|x, _| x.to_string()));
        t.set_ctor(&mut xaml, Some(Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        })));
        let source = "<Bool xmlns='https://a1-triard.github.io/tuifw/2023/xaml'>True</Bool>";
        let mut dest = Vec::new();
        preprocess_xaml(&xaml, || Ok(source.as_bytes()), &mut dest).unwrap();
        assert_eq!(&dest[..], b"true");
    }

    #[test]
    fn process_struct_with_property() {
        let mut xaml = Xaml::new();
        let b = XamlLiteral::new(&mut xaml, "https://a1-triard.github.io/tuifw/2023/xaml", "Bool");
        let bg = XamlStruct::new(&mut xaml, None, "https://a1-triard.github.io/tuifw/2023/xaml", "Background");
        let bg_sp = XamlProperty::new(&mut xaml, bg, "ShowPattern", XamlType::Literal(b), false, false);
        xaml.set_result(Box::new(|x, _| x.to_string()));
        b.set_ctor(&mut xaml, Some(Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        })));
        bg.set_ctor(&mut xaml, Some(Box::new(|x, _, _|
            format!("let mut {x} = Background::new();\n")
        )));
        bg_sp.set_setter(&mut xaml, Box::new(|o, x|
            format!("Background::set_show_pattern({o}, {x});\n")
        ));
        let source = "
            <Background
                xmlns='https://a1-triard.github.io/tuifw/2023/xaml'
                ShowPattern='True'
            />
        ";
        let mut dest = Vec::new();
        preprocess_xaml(&xaml, || Ok(source.as_bytes()), &mut dest).unwrap();
        assert_eq!(str::from_utf8(&dest[..]).unwrap(), "\
            let mut obj_1 = Background::new();\n\
            Background::set_show_pattern(obj_1, true);\n\
            obj_1\
        ");
    }

    #[test]
    fn process_struct_with_expanded_property() {
        let mut xaml = Xaml::new();
        let b = XamlLiteral::new(&mut xaml, "https://a1-triard.github.io/tuifw/2023/xaml", "Bool");
        let bg = XamlStruct::new(&mut xaml, None, "https://a1-triard.github.io/tuifw/2023/xaml", "Background");
        let bg_sp = XamlProperty::new(&mut xaml, bg, "ShowPattern", XamlType::Literal(b), false, false);
        xaml.set_result(Box::new(|x, _| x.to_string()));
        b.set_ctor(&mut xaml, Some(Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        })));
        bg.set_ctor(&mut xaml, Some(Box::new(|x, _, _|
            format!("let mut {x} = Background::new();\n")
        )));
        bg_sp.set_setter(&mut xaml, Box::new(|o, x|
            format!("Background::set_show_pattern({o}, {x});\n")
        ));
        let source = "
            <Background xmlns='https://a1-triard.github.io/tuifw/2023/xaml'>
                <Background.ShowPattern><Bool>True</Bool></Background.ShowPattern>
            </Background>
        ";
        let mut dest = Vec::new();
        preprocess_xaml(&xaml, || Ok(source.as_bytes()), &mut dest).unwrap();
        assert_eq!(str::from_utf8(&dest[..]).unwrap(), "\
            let mut obj_1 = Background::new();\n\
            Background::set_show_pattern(obj_1, true);\n\
            obj_1\
        ");
    }

    #[test]
    fn process_struct_with_expanded_property_2() {
        let mut xaml = Xaml::new();
        let b = XamlLiteral::new(&mut xaml, "https://a1-triard.github.io/tuifw/2023/xaml", "Bool");
        let bg = XamlStruct::new(&mut xaml, None, "https://a1-triard.github.io/tuifw/2023/xaml", "Background");
        let bg_sp = XamlProperty::new(&mut xaml, bg, "ShowPattern", XamlType::Literal(b), false, false);
        xaml.set_result(Box::new(|x, _| x.to_string()));
        b.set_ctor(&mut xaml, Some(Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        })));
        bg.set_ctor(&mut xaml, Some(Box::new(|x, _, _|
            format!("let mut {x} = Background::new();\n")
        )));
        bg_sp.set_setter(&mut xaml, Box::new(|o, x|
            format!("Background::set_show_pattern({o}, {x});\n")
        ));
        let source = "
            <Background xmlns='https://a1-triard.github.io/tuifw/2023/xaml'>
                <Background.ShowPattern>True</Background.ShowPattern>
            </Background>
        ";
        let mut dest = Vec::new();
        preprocess_xaml(&xaml, || Ok(source.as_bytes()), &mut dest).unwrap();
        assert_eq!(str::from_utf8(&dest[..]).unwrap(), "\
            let mut obj_1 = Background::new();\n\
            Background::set_show_pattern(obj_1, true);\n\
            obj_1\
        ");
    }
}
