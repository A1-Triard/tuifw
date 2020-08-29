use core::any::{Any, TypeId};
use core::mem::replace;
use alloc::vec::Vec;

pub trait Context {
    fn get(&self, type_: TypeId) -> Option<&dyn Any>;
    fn get_mut(&mut self, type_: TypeId) -> Option<&mut dyn Any>;
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Reactive<Type> {
    value: Type,
    #[derivative(Debug="ignore")]
    on_changed: Option<Vec<fn(context: &mut dyn Context, old: &Type)>>,
}

pub struct OnChanged<Type>(
    Option<Vec<fn(context: &mut dyn Context, old: &Type)>>
);

impl<Type> OnChanged<Type> {
    pub fn raise(self, context: &mut dyn Context, old: &Type) {
        if let Some(on_changed) = self.0 {
            for on_changed in on_changed {
                on_changed(context, old);
            }
        }
    }
}

impl<Type: Eq> Reactive<Type> {
    pub fn set_dist(&mut self, value: Type) -> (Type, OnChanged<Type>) {
        let old = replace(&mut self.value, value);
        let on_changed = if old == self.value { None } else { self.on_changed.clone() };
        (old, OnChanged(on_changed))
    }
}

impl<Type> Reactive<Type> {
    pub fn new(value: Type) -> Self {
        Reactive { value, on_changed: None }
    }

    pub fn set(&mut self, value: Type) -> (Type, OnChanged<Type>) {
        let old = replace(&mut self.value, value);
        (old, OnChanged(self.on_changed.clone()))
    }

    pub fn get(&self) -> &Type { &self.value }

    pub fn on_changed(
        &mut self,
        callback: fn(context: &mut dyn Context, old: &Type)
    ) {
        if let Some(on_changed) = self.on_changed.as_mut() {
            on_changed.push(callback);
        } else {
            self.on_changed = Some(vec![callback]);
        }
    }
}
