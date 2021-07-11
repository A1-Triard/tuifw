use core::fmt::Debug;
use core::ops::Range;

/// A type should satisfy this trait to be a dependency property type,
/// a dependency vector item type, or a flow data type.
pub trait Convenient: Clone + Debug + Send + Sync + 'static { }

impl<T: Clone + Debug + Send + Sync + 'static> Convenient for T { }

#[derive(Debug, Clone)]
pub enum VecChange<ItemType: Convenient> {
    Reset(Vec<ItemType>),
    Inserted(Range<usize>),
    Removed(usize, Vec<ItemType>),
    Swapped(Range<usize>, Range<usize>),
}
