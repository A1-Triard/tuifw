use core::fmt::Debug;

/// A type should satisfy this trait to be a dependency property type,
/// a dependency vector item type, or a flow data type.
pub trait Convenient: Clone + Debug + Send + Sync + 'static { }

impl<T: Clone + Debug + Send + Sync + 'static> Convenient for T { }
