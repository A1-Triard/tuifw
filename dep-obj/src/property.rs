use std::mem::replace;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Property<Type, Context> {
    value: Type,
    #[derivative(Debug="ignore")]
    on_changed: Vec<fn(context: &mut Context, old: &Type)>,
}

pub struct PropertyOnChanged<Type, Context>(
    Vec<fn(context: &mut Context, old: &Type)>
);

impl<Type, Context> PropertyOnChanged<Type, Context> {
    pub fn raise(self, context: &mut Context, old: &Type) {
        for on_changed in self.0 {
            on_changed(context, old);
        }
    }
}

impl<Type, Context> Property<Type, Context> {
    pub fn new(value: Type) -> Self {
        Property { value, on_changed: Vec::new() }
    }

    pub fn set(&mut self, value: Type) -> (Type, PropertyOnChanged<Type, Context>) {
        let old = replace(&mut self.value, value);
        (old, PropertyOnChanged(self.on_changed.clone()))
    }

    pub fn get(&self) -> &Type { &self.value }

    pub fn on_changed(
        &mut self,
        callback: fn(context: &mut Context, old: &Type)
    ) {
        self.on_changed.push(callback);
    }
}
