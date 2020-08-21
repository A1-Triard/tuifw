#[macro_use]
extern crate tylift;

pub mod context;

use std::mem::replace;
use context::{Context, ContextToken};

pub struct Property<Owner, Type, Token: ContextToken> {
    value: Type,
    on_changed: Vec<fn(owner: &mut Owner, context: &mut Context<Token>, old: &Type)>,
}

pub struct PropertyOnChanged<Owner, Type, Token: ContextToken>(
    Vec<fn(owner: &mut Owner, context: &mut Context<Token>, old: &Type)>
);

impl<Owner, Type, Token: ContextToken> PropertyOnChanged<Owner, Type, Token> {
    pub fn raise(self, owner: &mut Owner, context: &mut Context<Token>, old: &Type) {
        for on_changed in self.0 {
            on_changed(owner, context, old);
        }
    }
}

impl<Owner, Type, Token: ContextToken> Property<Owner, Type, Token> {
    pub fn new(value: Type) -> Self {
        Property { value, on_changed: Vec::new() }
    }

    pub fn set(&mut self, value: Type) -> (Type, PropertyOnChanged<Owner, Type, Token>) {
        let old = replace(&mut self.value, value);
        (old, PropertyOnChanged(self.on_changed.clone()))
    }

    pub fn get(&self) -> &Type { &self.value }

    pub fn on_changed(
        &mut self,
        callback: fn(owner: &mut Owner, context: &mut Context<Token>, old: &Type)
    ) {
        self.on_changed.push(callback);
    }
}

#[macro_export]
macro_rules! property {
    ($type_:ty, $name:ident, $set_name:ident, $on_changed_name:ident) => {
        pub fn $name(&self) -> &$type_ { self.$name.get() }

        pub fn $set_name(&mut self, value: $type_) -> $type_ { 
            let (old, on_changed) = self.$name.set(value);
            on_changed.raise(self, &old);
            old
        }

        pub fn $on_changed_name(
            &mut self,
            callback: fn(owner: &mut Self, context: &mut Context<$context>, old: &$type_)
        ) {
            self.$name.on_changed(callback);
        }
    }
}
