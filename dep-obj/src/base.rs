use alloc::vec::Vec;
use components_arena::ComponentId;
use core::fmt::Debug;
use core::ops::Range;
use dyn_context::State;
use educe::Educe;

/// A type should satisfy this trait to be a dependency property type,
/// a dependency vector item type, or a flow data type.
pub trait Convenient: Clone + Debug + Send + Sync + 'static { }

impl<T: Clone + Debug + Send + Sync + 'static> Convenient for T { }

#[derive(Educe)]
#[educe(Debug, Clone, Copy)]
pub struct Glob<Id: ComponentId, Obj> {
    pub id: Id,
    #[educe(Debug(ignore))]
    pub get_obj_mut: for<'a> fn(state: &'a mut dyn State, id: Id) -> &'a mut Obj,
}

#[derive(Debug, Clone)]
pub enum VecChange<ItemType: Convenient> {
    Reset(Vec<ItemType>),
    Inserted(Range<usize>),
    Removed(usize, Vec<ItemType>),
    Swapped(Range<usize>, Range<usize>),
}
