use core::mem::replace;
use alloc::vec::Vec;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Reactive<Type, Context> {
    value: Type,
    #[derivative(Debug="ignore")]
    on_changed: Vec<fn(context: &mut Context, old: &Type)>,
}

pub struct OnChanged<Type, Context>(
    Vec<fn(context: &mut Context, old: &Type)>
);

impl<Type, Context> OnChanged<Type, Context> {
    pub fn raise(self, context: &mut Context, old: &Type) {
        for on_changed in self.0 {
            on_changed(context, old);
        }
    }
}

impl<Type, Context> Reactive<Type, Context> {
    pub fn new(value: Type) -> Self {
        Reactive { value, on_changed: Vec::new() }
    }

    pub fn set(&mut self, value: Type) -> (Type, OnChanged<Type, Context>) {
        let old = replace(&mut self.value, value);
        (old, OnChanged(self.on_changed.clone()))
    }

    pub fn get(&self) -> &Type { &self.value }

    pub fn on_changed(
        &mut self,
        callback: fn(context: &mut Context, old: &Type)
    ) {
        self.on_changed.push(callback);
    }
}
