use components_arena::ComponentId;
use core::any::{Any, TypeId};
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use dyn_context::state::State;
use educe::Educe;

/// A type should satisfy this trait to be a dependency property type,
/// a dependency vector item type, or a flow data type.
pub trait Convenient: PartialEq + Clone + Debug + Send + Sync + 'static { }

impl<T: PartialEq + Clone + Debug + Send + Sync + 'static> Convenient for T { }

pub struct GlobDescriptor<Id: ComponentId, Obj> {
    pub arena: TypeId,
    pub field_ref: fn(arena: &dyn Any, id: Id) -> &Obj,
    pub field_mut: fn(arena: &mut dyn Any, id: Id) -> &mut Obj
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct Glob<Id: ComponentId, Obj> {
    pub id: Id,
    pub descriptor: fn() -> GlobDescriptor<Id, Obj>,
}

pub struct GlobRef<'a, Id: ComponentId, Obj> {
    pub arena: &'a dyn Any,
    pub glob: Glob<Id, Obj>,
}

impl<'a, Id: ComponentId, Obj> Deref for GlobRef<'a, Id, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        ((self.glob.descriptor)().field_ref)(self.arena.deref(), self.glob.id)
    }
}

pub struct GlobMut<'a, Id: ComponentId, Obj> {
    pub arena: &'a mut dyn Any,
    pub glob: Glob<Id, Obj>,
}

impl<'a, Id: ComponentId, Obj> Deref for GlobMut<'a, Id, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        ((self.glob.descriptor)().field_ref)(self.arena.deref(), self.glob.id)
    }
}

impl<'a, Id: ComponentId, Obj> DerefMut for GlobMut<'a, Id, Obj> {
    fn deref_mut(&mut self) -> &mut Obj {
        ((self.glob.descriptor)().field_mut)(self.arena.deref_mut(), self.glob.id)
    }
}

impl<Id: ComponentId, Obj> Glob<Id, Obj> {
    pub fn get(self, state: &dyn State) -> GlobRef<Id, Obj> {
        let arena = (self.descriptor)().arena;
        GlobRef {
            arena: state.get_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            glob: self
        }
    }

    pub fn get_mut(self, state: &mut dyn State) -> GlobMut<Id, Obj> {
        let arena = (self.descriptor)().arena;
        GlobMut {
            arena: state.get_mut_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            glob: self
        }
    }
}
