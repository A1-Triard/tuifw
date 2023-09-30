#![deny(warnings)]
#![doc(test(attr(deny(warnings))))]
#![doc(test(attr(allow(dead_code))))]
#![doc(test(attr(allow(unused_variables))))]
#![allow(clippy::match_ref_pats)]
#![allow(clippy::type_complexity)]

use components_arena::{Arena, Component, Id};
use macro_attr_2018::macro_attr;
use std::borrow::Cow;
use std::collections::HashMap;
use std::io::{Read, Write};
use xml::EventReader;
use xml::attribute::OwnedAttribute;
use xml::common::Position;
use xml::name::OwnedName;
use xml::reader::XmlEvent;
use xml::reader::Error as xml_Error;
use xml::reader::Result as xml_Result;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum XamlType {
    Struct(Id<XamlStruct>),
    Literal(Id<XamlLiteral>),
}

macro_attr! {
    #[derive(Component!)]
    pub struct XamlStruct {
        #[allow(dead_code)]
        name: String,
        prop_names: HashMap<String, Id<XamlProp>>,
        content_prop: Option<Id<XamlProp>>,
        new: Box<dyn Fn(&str, Option<(&str, Id<XamlProp>, Option<&str>)>) -> String>,
    }
}

macro_attr! {
    #[derive(Component!)]
    pub struct XamlProp {
        owner: Id<XamlStruct>,
        #[allow(dead_code)]
        name: String,
        ty: XamlType,
        set: Box<dyn Fn(&str, &str) -> String>,
    }
}

macro_attr! {
    #[derive(Component!)]
    pub struct XamlLiteral {
        #[allow(dead_code)]
        name: String,
        new: Box<dyn Fn(&str) -> Option<String>>,
    }
}

pub struct Xaml {
    res: Box<dyn Fn(&str) -> String>,
    structs: Arena<XamlStruct>,
    literals: Arena<XamlLiteral>,
    type_names: HashMap<String, XamlType>,
    props: Arena<XamlProp>,
}

impl Xaml {
    pub fn new(
        res: Box<dyn Fn(&str) -> String>,
    ) -> Self {
        Xaml {
            res,
            structs: Arena::new(),
            literals: Arena::new(),
            type_names: HashMap::new(),
            props: Arena::new()
        }
    }

    pub fn reg_struct<'a>(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        new: Box<dyn Fn(&str, Option<(&str, Id<XamlProp>, Option<&str>)>) -> String>,
    ) -> Id<XamlStruct> {
        let name = name.into().into_owned();
        let ty = XamlStruct { name: name.clone(), prop_names: HashMap::new(), content_prop: None, new };
        let id = self.structs.insert(|id| (ty, id));
        self.type_names.insert(name, XamlType::Struct(id));
        id
    }

    pub fn reg_literal<'a>(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        new: Box<dyn Fn(&str) -> Option<String>>,
    ) -> Id<XamlLiteral> {
        let name = name.into().into_owned();
        let ty = XamlLiteral { name: name.clone(), new };
        let id = self.literals.insert(|id| (ty, id));
        self.type_names.insert(name, XamlType::Literal(id));
        id
    }

    pub fn reg_prop<'a>(
        &mut self,
        owner: Id<XamlStruct>,
        name: impl Into<Cow<'a, str>>,
        ty: XamlType,
        set: Box<dyn Fn(&str, &str) -> String>,
    ) -> Id<XamlProp> {
        let name = name.into().into_owned();
        let prop = XamlProp { owner, name: name.clone(), ty, set };
        let id = self.props.insert(|id| (prop, id));
        self.structs[owner].prop_names.insert(name, id);
        id
    }

    pub fn make_content(&mut self, prop: Id<XamlProp>) {
        let owner = self.props[prop].owner;
        assert!(self.structs[owner].content_prop.replace(prop).is_none());
    }

    pub fn process(&self, source: impl Read, dest: impl Write) -> xml_Result<()> {
        let mut source = EventReader::new(source);
        let event = source.next()?;
        XamlProcesser { xaml: self, source, dest, event, obj_n: 0 }.process()
    }
}

struct XamlProcesser<'a, R: Read, W: Write> {
    xaml: &'a Xaml,
    source: EventReader<R>,
    dest: W,
    event: XmlEvent,
    obj_n: u16,
}

impl<'a, R: Read, W: Write> XamlProcesser<'a, R, W> {
    fn next_event(&mut self) -> xml_Result<()> {
        self.event = self.source.next()?;
        while matches!(&self.event, XmlEvent::Comment(_)) {
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
        let value = self.process_element(None)?;
        match &self.event {
            XmlEvent::EndDocument { .. } => { },
            _ => return self.error("miltiple root records"),
        }
        write!(self.dest, "{}", (self.xaml.res)(&value))?;
        Ok(())
    }

    fn process_element(
        &mut self,
        parent_prop: Option<(&str, Id<XamlProp>, Option<&str>)>,
    ) -> xml_Result<String> {
        let (name, attributes) = match &self.event {
            XmlEvent::StartElement { name, attributes, .. } => (Self::name(name), attributes.clone()),
            _ => return self.error("element start expected"),
        };
        let Some(ty) = self.xaml.type_names.get(&name) else {
            return self.error(format!("unknown type '{}'", name));
        };
        match ty {
            &XamlType::Literal(ty) => self.process_literal(ty, attributes),
            &XamlType::Struct(ty) => self.process_struct(ty, attributes, parent_prop),
        }
    }

    fn process_literal(&mut self, ty: Id<XamlLiteral>, attributes: Vec<OwnedAttribute>) -> xml_Result<String> {
        if !attributes.is_empty() {
            return self.error(format!("unexpected attribute '{}'", Self::name(&attributes[0].name)));
        }
        self.next_event()?;
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
        assert!(matches!(&self.event, XmlEvent::EndElement { .. }));
        self.next_event()?;
        let ty = &self.xaml.literals[ty];
        if let Some(value) = (ty.new)(&value) {
            Ok(value)
        } else {
            self.error(format!("invalid literal '{value}'"))
        }
    }

    fn process_struct(
        &mut self,
        ty: Id<XamlStruct>,
        attributes: Vec<OwnedAttribute>,
        parent_prop: Option<(&str, Id<XamlProp>, Option<&str>)>,
    ) -> xml_Result<String> {
        let obj = self.new_obj_name()?;
        {
            let ty = &self.xaml.structs[ty];
            write!(self.dest, "{}", (ty.new)(&obj, parent_prop))?;
            for attr in attributes {
                let attr_name = Self::name(&attr.name);
                let Some(&prop) = ty.prop_names.get(&attr_name) else {
                    return self.error(format!("unknown property '{attr_name}'"));
                };
                let prop = &self.xaml.props[prop];
                let XamlType::Literal(prop_ty) = prop.ty else {
                    return self.error(format!("invalid '{attr_name}' property value"));
                };
                let Some(value) = (self.xaml.literals[prop_ty].new)(&attr.value) else {
                    return self.error(format!("invalid '{attr_name}' property value"));
                };
                write!(self.dest, "{}", (prop.set)(&obj, &value))?;
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

    fn process_property(&mut self, obj: String, ty: Id<XamlStruct>) -> xml_Result<()> {
        let (name, attributes) = match &self.event {
            XmlEvent::StartElement { name, attributes, .. } => (Self::name(name), attributes.clone()),
            _ => unreachable!(),
        };
        if !attributes.is_empty() {
            return self.error(format!("unexpected attribute '{}'", Self::name(&attributes[0].name)));
        }
        let ty = &self.xaml.structs[ty];
        let (prop, skip_end_element) = if
            name.starts_with(&ty.name) &&
            name.len() > ty.name.len() &&
            name.len() - ty.name.len() >= 2 &&
            name.as_bytes()[ty.name.len()] == b'.'
        {
            self.next_event()?;
            let prop_name = &name[ty.name.len() + 1 ..];
            let Some(&prop) = ty.prop_names.get(prop_name) else {
                return self.error(format!("unknown property '{prop_name}'"));
            };
            (prop, true)
        } else if let Some(content_prop) = ty.content_prop {
            (content_prop, false)
        } else {
            return self.error("type does not have content property");
        };
        let mut prev_value = None;
        loop {
            let value = match &self.event {
                XmlEvent::EndElement { .. } => { break; },
                XmlEvent::StartElement { .. } =>
                    self.process_element(Some((&obj, prop, prev_value.as_deref())))?,
                XmlEvent::Characters(_) | XmlEvent::Whitespace(_) => {
                    let XamlType::Literal(prop_ty) = self.xaml.props[prop].ty else {
                        return self.error(format!("invalid '{}' property value", self.xaml.props[prop].name));
                    };
                    self.process_literal(prop_ty, Vec::new())?
                },
                x => return self.error(format!("unsupported XML feature 2 {x:?}")),
            };
            write!(self.dest, "{}", (self.xaml.props[prop].set)(&obj, &value))?;
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
    use crate::*;
    use std::str::{self};

    #[test]
    fn process_literal() {
        let mut xaml = Xaml::new(Box::new(|x| x.to_string()));
        xaml.reg_literal("{https://a1-triard.github.io/tuifw/2023/xaml}Bool", Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        }));
        let source = "<Bool xmlns='https://a1-triard.github.io/tuifw/2023/xaml'>True</Bool>";
        let mut dest = Vec::new();
        xaml.process(source.as_bytes(), &mut dest).unwrap();
        assert_eq!(&dest[..], b"true");
    }

    #[test]
    fn process_struct_with_property() {
        let mut xaml = Xaml::new(Box::new(|x| x.to_string()));
        let b = xaml.reg_literal("{https://a1-triard.github.io/tuifw/2023/xaml}Bool", Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        }));
        let bg = xaml.reg_struct("{https://a1-triard.github.io/tuifw/2023/xaml}Background", Box::new(|x, _|
            format!("let mut {x} = Background::new();\n")
        ));
        xaml.reg_prop(bg, "ShowPattern", XamlType::Literal(b), Box::new(|o, x|
            format!("Background::set_show_pattern({o}, {x});\n")
        ));
        let source = "
            <Background
                xmlns='https://a1-triard.github.io/tuifw/2023/xaml'
                ShowPattern='True'
            />
        ";
        let mut dest = Vec::new();
        xaml.process(source.as_bytes(), &mut dest).unwrap();
        assert_eq!(str::from_utf8(&dest[..]).unwrap(), "\
            let mut obj_1 = Background::new();\n\
            Background::set_show_pattern(obj_1, true);\n\
            obj_1\
        ");
    }

    #[test]
    fn process_struct_with_expanded_property() {
        let mut xaml = Xaml::new(Box::new(|x| x.to_string()));
        let b = xaml.reg_literal("{https://a1-triard.github.io/tuifw/2023/xaml}Bool", Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        }));
        let bg = xaml.reg_struct("{https://a1-triard.github.io/tuifw/2023/xaml}Background", Box::new(|x, _|
            format!("let mut {x} = Background::new();\n")
        ));
        xaml.reg_prop(bg, "ShowPattern", XamlType::Literal(b), Box::new(|o, x|
            format!("Background::set_show_pattern({o}, {x});\n")
        ));
        let source = "
            <Background xmlns='https://a1-triard.github.io/tuifw/2023/xaml'>
                <Background.ShowPattern><Bool>True</Bool></Background.ShowPattern>
            </Background>
        ";
        let mut dest = Vec::new();
        xaml.process(source.as_bytes(), &mut dest).unwrap();
        assert_eq!(str::from_utf8(&dest[..]).unwrap(), "\
            let mut obj_1 = Background::new();\n\
            Background::set_show_pattern(obj_1, true);\n\
            obj_1\
        ");
    }
}
