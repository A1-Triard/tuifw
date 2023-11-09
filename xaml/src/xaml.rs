use components_arena::{Arena, Component, Id, NewtypeComponentId};
use macro_attr_2018::macro_attr;
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum XamlType {
    Struct(XamlStruct),
    Literal(XamlLiteral),
    Ref,
}

macro_attr! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, NewtypeComponentId!)]
    pub struct XamlStruct(Id<XamlStructData>);
}

impl XamlStruct {
    pub fn parent(self, xaml: &Xaml) -> Option<XamlStruct> {
        xaml.structs[self.0].parent
    }

    pub fn name(self, xaml: &Xaml) -> &str {
        &xaml.structs[self.0].name
    }

    pub fn self_property(self, xaml: &Xaml, name: &str) -> Option<XamlProperty> {
        xaml.structs[self.0].property_names.get(name).copied()
    }

    pub fn property(self, xaml: &Xaml, name: &str) -> Option<XamlProperty> {
        let mut owner = self;
        loop {
            if let Some(property) = owner.self_property(xaml, name) {
                return Some(property);
            }
            if let Some(parent) = owner.parent(xaml) {
                owner = parent;
            } else {
                break;
            }
        }
        None
    }

    pub fn self_content_property(self, xaml: &Xaml) -> Option<XamlProperty> {
        xaml.structs[self.0].content_property
    }

    pub fn self_name_property(self, xaml: &Xaml) -> Option<XamlProperty> {
        xaml.structs[self.0].name_property
    }

    pub fn content_property(self, xaml: &Xaml) -> Option<XamlProperty> {
        let mut owner = self;
        loop {
            if let Some(property) = owner.self_content_property(xaml) {
                return Some(property);
            }
            if let Some(parent) = owner.parent(xaml) {
                owner = parent;
            } else {
                break;
            }
        }
        None
    }

    pub fn name_property(self, xaml: &Xaml) -> Option<XamlProperty> {
        let mut owner = self;
        loop {
            if let Some(property) = owner.self_name_property(xaml) {
                return Some(property);
            }
            if let Some(parent) = owner.parent(xaml) {
                owner = parent;
            } else {
                break;
            }
        }
        None
    }

    pub fn new(
        xaml: &mut Xaml,
        parent: Option<XamlStruct>,
        namespace: &str,
        name: &str,
    ) -> XamlStruct {
        let name = format!("{{{namespace}}}{name}");
        let data = XamlStructData {
            name: name.clone(),
            property_names: HashMap::new(),
            content_property: None,
            name_property: None,
            parent,
            ctor: None,
        };
        let id = xaml.structs.insert(|id| (data, XamlStruct(id)));
        xaml.type_names.insert(name, XamlType::Struct(id));
        id
    }

    pub fn instance(
        self,
        xaml: &Xaml,
        name: &str,
        parent: Option<(&str, XamlProperty)>,
        prev: Option<&str>
    ) -> Option<String> {
        xaml.structs[self.0].ctor.as_ref().map(|x| x(name, parent, prev))
    }

    pub fn set_ctor(
        self,
        xaml: &mut Xaml,
        ctor: Option<Box<dyn Fn(&str, Option<(&str, XamlProperty)>, Option<&str>) -> String>>,
    ) {
        xaml.structs[self.0].ctor = ctor;
    }
}

macro_attr! {
    #[derive(Component!)]
    struct XamlStructData {
        parent: Option<XamlStruct>,
        name: String,
        property_names: HashMap<String, XamlProperty>,
        content_property: Option<XamlProperty>,
        name_property: Option<XamlProperty>,
        ctor: Option<Box<dyn Fn(&str, Option<(&str, XamlProperty)>, Option<&str>) -> String>>,
    }
}

macro_attr! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, NewtypeComponentId!)]
    pub struct XamlProperty(Id<XamlPropertyData>);
}

impl XamlProperty {
    pub fn name(self, xaml: &Xaml) -> &str {
        &xaml.properties[self.0].name
    }

    pub fn owner(self, xaml: &Xaml) -> XamlStruct {
        xaml.properties[self.0].owner
    }

    pub fn ty(self, xaml: &Xaml) -> XamlType {
        xaml.properties[self.0].ty
    }

    pub fn new(
        xaml: &mut Xaml,
        owner: XamlStruct,
        name: &str,
        ty: XamlType,
        is_content_property: bool,
        is_name_property: bool,
    ) -> XamlProperty {
        let name = name.to_string();
        let property = XamlPropertyData {
            owner,
            name: name.clone(),
            ty,
            setter: Box::new(|_, _| String::new())
        };
        let id = xaml.properties.insert(|id| (property, XamlProperty(id)));
        let owner_data = &mut xaml.structs[owner.0];
        owner_data.property_names.insert(name, id);
        if is_content_property {
            assert!(owner_data.content_property.replace(id).is_none(), "duplicate content property");
        }
        if is_name_property {
            assert!(owner_data.name_property.replace(id).is_none(), "duplicate name property");
        }
        id
    }

    pub fn set(self, xaml: &Xaml, obj: &str, value: &str) -> String {
        (xaml.properties[self.0].setter)(obj, value)
    }

    pub fn set_setter(
        self,
        xaml: &mut Xaml,
        setter: Box<dyn Fn(&str, &str) -> String>,
    ) {
        xaml.properties[self.0].setter = setter;
    }
}

macro_attr! {
    #[derive(Component!)]
    pub struct XamlPropertyData {
        owner: XamlStruct,
        name: String,
        ty: XamlType,
        setter: Box<dyn Fn(&str, &str) -> String>,
    }
}

macro_attr! {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, NewtypeComponentId!)]
    pub struct XamlLiteral(Id<XamlLiteralData>);
}

impl XamlLiteral {
    pub fn name(self, xaml: &Xaml) -> &str {
        &xaml.literals[self.0].name
    }

    pub fn new(
        xaml: &mut Xaml,
        namespace: &str,
        name: &str,
    ) -> XamlLiteral {
        let name = format!("{{{namespace}}}{name}");
        let ty = XamlLiteralData { name: name.clone(), ctor: None };
        let id = xaml.literals.insert(|id| (ty, XamlLiteral(id)));
        xaml.type_names.insert(name, XamlType::Literal(id));
        id
    }

    pub fn instance(self, xaml: &Xaml, value: &str) -> Option<String> {
        xaml.literals[self.0].ctor.as_ref().and_then(|x| x(value))
    }

    pub fn set_ctor(
        self,
        xaml: &mut Xaml,
        ctor: Option<Box<dyn Fn(&str) -> Option<String>>>,
    ) {
        xaml.literals[self.0].ctor = ctor;
    }

}

macro_attr! {
    #[derive(Component!)]
    pub struct XamlLiteralData {
        name: String,
        ctor: Option<Box<dyn Fn(&str) -> Option<String>>>,
    }
}

pub struct Xaml {
    preamble: String,
    header: String,
    footer: String,
    postamble: Box<dyn Fn(&HashMap<String, String>) -> String>,
    result: Box<dyn Fn(&str, &HashMap<String, String>) -> String>,
    structs: Arena<XamlStructData>,
    literals: Arena<XamlLiteralData>,
    type_names: HashMap<String, XamlType>,
    properties: Arena<XamlPropertyData>,
}

impl Default for Xaml {
    fn default() -> Self {
        Self::new()
    }
}

impl Xaml {
    pub fn new() -> Self {
        Xaml {
            result: Box::new(|_, _| String::new()),
            structs: Arena::new(),
            literals: Arena::new(),
            type_names: HashMap::new(),
            properties: Arena::new(),
            preamble: String::new(),
            postamble: Box::new(|_| String::new()),
            header: String::new(),
            footer: String::new(),
        }
    }

    pub fn ty(&self, name: &str) -> Option<XamlType> {
        self.type_names.get(name).copied()
    }

    pub fn preamble(&self) -> &str {
        &self.preamble
    }

    pub fn header(&self) -> &str {
        &self.header
    }

    pub fn result(&self, obj: &str, names: &HashMap<String, String>) -> String {
        (self.result)(obj, names)
    }

    pub fn footer(&self) -> &str {
        &self.footer
    }

    pub fn postamble(&self, names: &HashMap<String, String>) -> String {
        (self.postamble)(names)
    }

    pub fn set_preamble<'a>(&mut self, preamble: impl Into<Cow<'a, str>>) {
        self.preamble = preamble.into().into_owned();
    }

    pub fn append_preamble(&mut self, preamble: &str) {
        self.preamble.push_str(preamble);
    }

    pub fn set_header<'a>(&mut self, header: impl Into<Cow<'a, str>>) {
        self.header = header.into().into_owned();
    }

    pub fn set_footer<'a>(&mut self, footer: impl Into<Cow<'a, str>>) {
        self.footer = footer.into().into_owned();
    }

    pub fn set_result(&mut self, result: Box<dyn Fn(&str, &HashMap<String, String>) -> String>) {
        self.result = result;
    }

    pub fn set_postamble(&mut self, postamble: Box<dyn Fn(&HashMap<String, String>) -> String>) {
        self.postamble = postamble;
    }

}
