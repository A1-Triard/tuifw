use core::mem::replace;
use alloc::vec::Vec;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Reactive<Type, Context> {
    value: Type,
    #[derivative(Debug="ignore")]
    on_changed: Option<Vec<fn(context: &mut Context, old: &Type)>>,
}

pub struct OnChanged<Type, Context>(
    Option<Vec<fn(context: &mut Context, old: &Type)>>
);

impl<Type, Context> OnChanged<Type, Context> {
    pub fn raise(self, context: &mut Context, old: &Type) {
        if let Some(on_changed) = self.0 {
            for on_changed in on_changed {
                on_changed(context, old);
            }
        }
    }
}

impl<Type: Eq, Context> Reactive<Type, Context> {
    pub fn set_dist(&mut self, value: Type) -> (Type, OnChanged<Type, Context>) {
        let old = replace(&mut self.value, value);
        let on_changed = if old == self.value { None } else { self.on_changed.clone() };
        (old, OnChanged(on_changed))
    }
}

impl<Type, Context> Reactive<Type, Context> {
    pub fn new(value: Type) -> Self {
        Reactive { value, on_changed: None }
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
        if let Some(on_changed) = self.on_changed.as_mut() {
            on_changed.push(callback);
        } else {
            self.on_changed = Some(vec![callback]);
        }
    }
}
