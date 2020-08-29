use core::any::{Any, TypeId};
use core::mem::replace;
use alloc::vec::Vec;

pub trait Context {
    fn get_raw(&self, type_: TypeId) -> Option<&dyn Any>;
    fn get_mut_raw(&mut self, type_: TypeId) -> Option<&mut dyn Any>;
}

pub trait ContextExt: Context {
    fn get<T: 'static>(&self) -> Option<&T> {
        self.get_raw(TypeId::of::<T>()).map(|x| x.downcast_ref::<T>().expect("invalid cast"))
    }

    fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.get_mut_raw(TypeId::of::<T>()).map(|x| x.downcast_mut::<T>().expect("invalid cast"))
    }
}

impl<T: Context + ?Sized> ContextExt for T { }

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Reactive<Owner: Copy, Type> {
    value: Type,
    #[derivative(Debug="ignore")]
    on_changed: Option<Vec<fn(owner: Owner, context: &mut dyn Context, old: &Type)>>,
}

pub struct OnChanged<Owner: Copy, Type>(
    Option<Vec<fn(owner: Owner, context: &mut dyn Context, old: &Type)>>
);

impl<Owner: Copy, Type> OnChanged<Owner, Type> {
    pub fn raise(self, owner: Owner, context: &mut dyn Context, old: &Type) {
        if let Some(on_changed) = self.0 {
            for on_changed in on_changed {
                on_changed(owner, context, old);
            }
        }
    }
}

impl<Owner: Copy, Type: Eq> Reactive<Owner, Type> {
    pub fn set_dist(&mut self, value: Type) -> (Type, OnChanged<Owner, Type>) {
        let old = replace(&mut self.value, value);
        let on_changed = if old == self.value { None } else { self.on_changed.clone() };
        (old, OnChanged(on_changed))
    }
}

impl<Owner: Copy, Type> Reactive<Owner, Type> {
    pub fn new(value: Type) -> Self {
        Reactive { value, on_changed: None }
    }

    pub fn set(&mut self, value: Type) -> (Type, OnChanged<Owner, Type>) {
        let old = replace(&mut self.value, value);
        (old, OnChanged(self.on_changed.clone()))
    }

    pub fn get(&self) -> &Type { &self.value }

    pub fn on_changed(
        &mut self,
        callback: fn(owner: Owner, context: &mut dyn Context, old: &Type)
    ) {
        if let Some(on_changed) = self.on_changed.as_mut() {
            on_changed.push(callback);
        } else {
            self.on_changed = Some(vec![callback]);
        }
    }
}
