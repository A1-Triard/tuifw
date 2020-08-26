use std::mem::replace;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Property<Owner, Type, Context> {
    value: Type,
    #[derivative(Debug="ignore")]
    on_changed: Vec<fn(owner: &mut Owner, context: &mut Context, old: &Type)>,
}

pub struct PropertyOnChanged<Owner, Type, Context>(
    Vec<fn(owner: &mut Owner, context: &mut Context, old: &Type)>
);

impl<Owner, Type, Context> PropertyOnChanged<Owner, Type, Context> {
    pub fn raise(self, owner: &mut Owner, context: &mut Context, old: &Type) {
        for on_changed in self.0 {
            on_changed(owner, context, old);
        }
    }
}

impl<Owner, Type, Context> Property<Owner, Type, Context> {
    pub fn new(value: Type) -> Self {
        Property { value, on_changed: Vec::new() }
    }

    pub fn set(&mut self, value: Type) -> (Type, PropertyOnChanged<Owner, Type, Context>) {
        let old = replace(&mut self.value, value);
        (old, PropertyOnChanged(self.on_changed.clone()))
    }

    pub fn get(&self) -> &Type { &self.value }

    pub fn on_changed(
        &mut self,
        callback: fn(owner: &mut Owner, context: &mut Context, old: &Type)
    ) {
        self.on_changed.push(callback);
    }
}

#[macro_export]
macro_rules! property {
    ($type_:ty, $name:ident, $set_name:ident, $on_name_changed:ident, $context:ty) => {
        pub fn $name(&self) -> &$type_ { self.$name.get() }

        pub fn $set_name(&mut self, value: $type_, context: &mut $context) -> $type_ { 
            let (old, on_changed) = self.$name.set(value);
            on_changed.raise(self, context, &old);
            old
        }

        pub fn $on_name_changed(
            &mut self,
            callback: fn(owner: &mut Self, context: &mut $context, old: &$type_)
        ) {
            self.$name.on_changed(callback);
        }
    }
}
