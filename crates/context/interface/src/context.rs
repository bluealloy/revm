use auto_impl::auto_impl;

/// Some actions on top of context with just Getter traits would require borrowing the context
/// with a both mutable and immutable reference.
///
/// To allow doing this action more efficiently, we introduce a new trait that does this directly.
///
/// Used for loading access list and applying EIP-7702 authorization list.
#[auto_impl(&mut,Box)]
pub trait PerformantContextAccess {
    type Error;

    /// Load access list
    fn load_access_list(&mut self) -> Result<(), Self::Error>;
}
