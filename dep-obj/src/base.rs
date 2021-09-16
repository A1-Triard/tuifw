use components_arena::{RawId};
use core::any::{Any, TypeId};
use core::fmt::Debug;
use core::ops::{Deref, DerefMut};
use dyn_context::state::State;
use educe::Educe;

/// A type should satisfy this trait to be a dependency property type,
/// a dependency vector item type, or a flow data type.
pub trait Convenient: PartialEq + Clone + Debug + Send + Sync + 'static { }

impl<T: PartialEq + Clone + Debug + Send + Sync + 'static> Convenient for T { }

pub struct GlobDescriptor<Obj> {
    pub arena: TypeId,
    pub field_ref: fn(arena: &dyn Any, id: RawId) -> &Obj,
    pub field_mut: fn(arena: &mut dyn Any, id: RawId) -> &mut Obj
}

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct Glob<Obj> {
    pub id: RawId,
    pub descriptor: fn() -> GlobDescriptor<Obj>,
}

pub struct GlobRef<'a, Obj> {
    pub arena: &'a dyn Any,
    pub glob: Glob<Obj>,
}

impl<'a, Obj> Deref for GlobRef<'a, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        ((self.glob.descriptor)().field_ref)(self.arena.deref(), self.glob.id)
    }
}

pub struct GlobMut<'a, Obj> {
    pub arena: &'a mut dyn Any,
    pub glob: Glob<Obj>,
}

impl<'a, Obj> Deref for GlobMut<'a, Obj> {
    type Target = Obj;

    fn deref(&self) -> &Obj {
        ((self.glob.descriptor)().field_ref)(self.arena.deref(), self.glob.id)
    }
}

impl<'a, Obj> DerefMut for GlobMut<'a, Obj> {
    fn deref_mut(&mut self) -> &mut Obj {
        ((self.glob.descriptor)().field_mut)(self.arena.deref_mut(), self.glob.id)
    }
}

impl<Obj> Glob<Obj> {
    pub fn get(self, state: &dyn State) -> GlobRef<Obj> {
        let arena = (self.descriptor)().arena;
        GlobRef {
            arena: state.get_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            glob: self
        }
    }

    pub fn get_mut(self, state: &mut dyn State) -> GlobMut<Obj> {
        let arena = (self.descriptor)().arena;
        GlobMut {
            arena: state.get_mut_raw(arena).unwrap_or_else(|| panic!("{:?} required", arena)),
            glob: self
        }
    }
}
