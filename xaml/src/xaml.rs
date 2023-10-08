use components_arena::{Arena, Component, Id};
use macro_attr_2018::macro_attr;
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum XamlType {
    Struct(Id<XamlStruct>),
    Literal(Id<XamlLiteral>),
}

macro_attr! {
    #[derive(Component!)]
    pub struct XamlStruct {
        parent: Option<Id<XamlStruct>>,
        #[allow(dead_code)]
        name: String,
        prop_names: HashMap<String, Id<XamlProp>>,
        content_prop: Option<Id<XamlProp>>,
        new: Option<Box<dyn Fn(&str, Option<(&str, Id<XamlProp>, Option<&str>)>) -> String>>,
    }
}

macro_attr! {
    #[derive(Component!)]
    pub struct XamlProp {
        owner: Id<XamlStruct>,
        #[allow(dead_code)]
        name: String,
        ty: XamlType,
        set: Option<Box<dyn Fn(&str, &str) -> String>>,
    }
}

macro_attr! {
    #[derive(Component!)]
    pub struct XamlLiteral {
        #[allow(dead_code)]
        name: String,
        new: Option<Box<dyn Fn(&str) -> Option<String>>>,
    }
}

pub struct Xaml {
    preamble: String,
    header: String,
    footer: String,
    res: Option<Box<dyn Fn(&str) -> String>>,
    structs: Arena<XamlStruct>,
    literals: Arena<XamlLiteral>,
    type_names: HashMap<String, XamlType>,
    props: Arena<XamlProp>,
}

impl Default for Xaml {
    fn default() -> Self {
        Self::new()
    }
}

impl Xaml {
    pub fn new() -> Self {
        Xaml {
            res: None,
            structs: Arena::new(),
            literals: Arena::new(),
            type_names: HashMap::new(),
            props: Arena::new(),
            preamble: String::new(),
            header: String::new(),
            footer: String::new(),
        }
    }

    pub fn reg_struct<'a>(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        parent: Option<Id<XamlStruct>>,
    ) -> Id<XamlStruct> {
        let name = name.into().into_owned();
        let ty = XamlStruct {
            name: name.clone(),
            prop_names: HashMap::new(),
            content_prop: None,
            parent,
            new: None,
        };
        let id = self.structs.insert(|id| (ty, id));
        self.type_names.insert(name, XamlType::Struct(id));
        id
    }

    pub fn reg_literal<'a>(
        &mut self,
        name: impl Into<Cow<'a, str>>,
    ) -> Id<XamlLiteral> {
        let name = name.into().into_owned();
        let ty = XamlLiteral { name: name.clone(), new: None };
        let id = self.literals.insert(|id| (ty, id));
        self.type_names.insert(name, XamlType::Literal(id));
        id
    }

    pub fn reg_prop<'a>(
        &mut self,
        owner: Id<XamlStruct>,
        name: impl Into<Cow<'a, str>>,
        ty: XamlType,
    ) -> Id<XamlProp> {
        let name = name.into().into_owned();
        let prop = XamlProp { owner, name: name.clone(), ty, set: None };
        let id = self.props.insert(|id| (prop, id));
        self.structs[owner].prop_names.insert(name, id);
        id
    }

    pub fn set_preamble<'a>(&mut self, preamble: impl Into<Cow<'a, str>>) {
        self.preamble = preamble.into().into_owned();
    }

    pub fn set_header<'a>(&mut self, header: impl Into<Cow<'a, str>>) {
        self.header = header.into().into_owned();
    }

    pub fn set_footer<'a>(&mut self, footer: impl Into<Cow<'a, str>>) {
        self.footer = footer.into().into_owned();
    }

    pub fn set_res(&mut self, res: Box<dyn Fn(&str) -> String>) {
        self.res = Some(res);
    }

    pub fn set_struct_new(
        &mut self,
        ty: Id<XamlStruct>,
        new: Option<Box<dyn Fn(&str, Option<(&str, Id<XamlProp>, Option<&str>)>) -> String>>,
    ) {
        self.structs[ty].new = new;
    }

    pub fn set_literal_new(
        &mut self,
        ty: Id<XamlLiteral>,
        new: Box<dyn Fn(&str) -> Option<String>>,
    ) {
        self.literals[ty].new = Some(new);
    }

    pub fn set_prop_set(
        &mut self,
        prop: Id<XamlProp>,
        set: Box<dyn Fn(&str, &str) -> String>,
    ) {
        self.props[prop].set = Some(set);
    }

    pub fn reset_content_prop(&mut self, ty: Id<XamlStruct>) {
        self.structs[ty].content_prop = None;
    }

    pub fn set_as_content_prop(&mut self, prop: Id<XamlProp>) {
        let owner = self.props[prop].owner;
        self.structs[owner].content_prop = Some(prop);
    }

    fn find_prop(&self, mut owner: Id<XamlStruct>, name: impl AsRef<str>) -> Option<Id<XamlProp>> {
        let name = name.as_ref();
        loop {
            let owner_data = &self.structs[owner];
            if let Some(&prop) = owner_data.prop_names.get(name) {
                return Some(prop);
            }
            if let Some(parent) = owner_data.parent {
                owner = parent;
            } else {
                break;
            }
        }
        None
    }

    fn find_content_prop(&self, mut owner: Id<XamlStruct>) -> Option<Id<XamlProp>> {
        loop {
            let owner_data = &self.structs[owner];
            if let Some(prop) = owner_data.content_prop {
                return Some(prop);
            }
            if let Some(parent) = owner_data.parent {
                owner = parent;
            } else {
                break;
            }
        }
        None
    }

    pub fn process_file(&self, source: impl AsRef<Path>, dest: impl AsRef<Path>) -> xml_Result<()> {
        let source = File::open(source.as_ref())?;
        let dest = File::create(dest.as_ref())?;
        self.process(source, dest)
    }

    pub fn process(&self, source: impl Read, mut dest: impl Write) -> xml_Result<()> {
        let mut source = EventReader::new(source);
        let event = source.next()?;
        write!(dest, "{}", self.preamble)?;
        write!(dest, "{}", self.header)?;
        let mut processor = XamlProcessor { xaml: self, source, dest, event, obj_n: 0 };
        processor.process()?;
        write!(processor.dest, "{}", self.footer)?;
        Ok(())
    }
}

struct XamlProcessor<'a, R: Read, W: Write> {
    xaml: &'a Xaml,
    source: EventReader<R>,
    dest: W,
    event: XmlEvent,
    obj_n: u16,
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
        let value = self.process_element(None)?;
        match &self.event {
            XmlEvent::EndDocument { .. } => { },
            _ => return self.error("miltiple root records"),
        }
        let Some(res) = self.xaml.res.as_ref() else {
            return self.error("XAML result processing function is not set");
        };
        write!(self.dest, "{}", res(&value))?;
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
        let res = self.process_literal_value(ty)?;
        assert!(matches!(&self.event, XmlEvent::EndElement { .. }));
        self.next_event()?;
        Ok(res)
    }

    fn process_literal_value(&mut self, ty: Id<XamlLiteral>) -> xml_Result<String> {
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
        let Some(new) = self.xaml.literals[ty].new.as_ref() else {
            return self.error("literal creation function is not set");
        };
        if let Some(value) = new(&value) {
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
        if let Some(new) = self.xaml.structs[ty].new.as_deref() {
            write!(self.dest, "{}", new(&obj, parent_prop))?;
        } else {
            return self.error("cannot create abstract type");
        }
        for attr in attributes {
            let attr_name = Self::name(&attr.name);
            let Some(prop) = self.xaml.find_prop(ty, &attr_name) else {
                return self.error(format!("unknown property '{attr_name}'"));
            };
            let prop = &self.xaml.props[prop];
            let XamlType::Literal(prop_ty) = prop.ty else {
                return self.error(format!("invalid '{attr_name}' property value"));
            };
            let Some(new) = self.xaml.literals[prop_ty].new.as_ref() else {
                return self.error("literal creation function is not set");
            };
            let Some(value) = new(&attr.value) else {
                return self.error(format!("invalid '{attr_name}' property value"));
            };
            let Some(set) = prop.set.as_ref() else {
                return self.error("property set function is not set");
            };
            write!(self.dest, "{}", set(&obj, &value))?;
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
        let ty_name = &self.xaml.structs[ty].name;
        let (prop, skip_end_element) = if
            name.starts_with(ty_name) &&
            name.len() > ty_name.len() &&
            name.len() - ty_name.len() >= 2 &&
            name.as_bytes()[ty_name.len()] == b'.'
        {
            self.next_event()?;
            let prop_name = &name[ty_name.len() + 1 ..];
            let Some(prop) = self.xaml.find_prop(ty, prop_name) else {
                return self.error(format!("unknown property '{prop_name}'"));
            };
            if !attributes.is_empty() {
                return self.error(format!("unexpected attribute '{}'", Self::name(&attributes[0].name)));
            }
            (prop, true)
        } else if let Some(content_prop) = self.xaml.find_content_prop(ty) {
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
                        return self.error(format!("invalid '{}' property value 2", self.xaml.props[prop].name));
                    };
                    self.process_literal_value(prop_ty)?
                },
                _ => return self.error("unsupported XML feature"),
            };
            let Some(set) = self.xaml.props[prop].set.as_ref() else {
                return self.error("property set function is not set");
            };
            write!(self.dest, "{}", set(&obj, &value))?;
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
        let t = xaml.reg_literal("{https://a1-triard.github.io/tuifw/2023/xaml}Bool");
        xaml.set_res(Box::new(|x| x.to_string()));
        xaml.set_literal_new(t, Box::new(|x| match x {
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
        let mut xaml = Xaml::new();
        let b = xaml.reg_literal("{https://a1-triard.github.io/tuifw/2023/xaml}Bool");
        let bg = xaml.reg_struct("{https://a1-triard.github.io/tuifw/2023/xaml}Background", None);
        let bg_sp = xaml.reg_prop(bg, "ShowPattern", XamlType::Literal(b));
        xaml.set_res(Box::new(|x| x.to_string()));
        xaml.set_literal_new(b, Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        }));
        xaml.set_struct_new(bg, Some(Box::new(|x, _|
            format!("let mut {x} = Background::new();\n")
        )));
        xaml.set_prop_set(bg_sp, Box::new(|o, x|
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
        let mut xaml = Xaml::new();
        let b = xaml.reg_literal("{https://a1-triard.github.io/tuifw/2023/xaml}Bool");
        let bg = xaml.reg_struct("{https://a1-triard.github.io/tuifw/2023/xaml}Background", None);
        let bg_sp = xaml.reg_prop(bg, "ShowPattern", XamlType::Literal(b));
        xaml.set_res(Box::new(|x| x.to_string()));
        xaml.set_literal_new(b, Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        }));
        xaml.set_struct_new(bg, Some(Box::new(|x, _|
            format!("let mut {x} = Background::new();\n")
        )));
        xaml.set_prop_set(bg_sp, Box::new(|o, x|
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

    #[test]
    fn process_struct_with_expanded_property_2() {
        let mut xaml = Xaml::new();
        let b = xaml.reg_literal("{https://a1-triard.github.io/tuifw/2023/xaml}Bool");
        let bg = xaml.reg_struct("{https://a1-triard.github.io/tuifw/2023/xaml}Background", None);
        let bg_sp = xaml.reg_prop(bg, "ShowPattern", XamlType::Literal(b));
        xaml.set_res(Box::new(|x| x.to_string()));
        xaml.set_literal_new(b, Box::new(|x| match x {
            "True" => Some("true".to_string()),
            "False" => Some("false".to_string()),
            _ => None,
        }));
        xaml.set_struct_new(bg, Some(Box::new(|x, _|
            format!("let mut {x} = Background::new();\n")
        )));
        xaml.set_prop_set(bg_sp, Box::new(|o, x|
            format!("Background::set_show_pattern({o}, {x});\n")
        ));
        let source = "
            <Background xmlns='https://a1-triard.github.io/tuifw/2023/xaml'>
                <Background.ShowPattern>True</Background.ShowPattern>
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
