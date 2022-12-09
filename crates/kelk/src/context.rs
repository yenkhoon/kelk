//! The context for running contract actor

use crate::{blockchain::Blockchain, storage::Storage};

/// `Context` holds the references to the storage and blockchain objects.
/// This can be easily mocked for the testing environment.
pub struct Context<'a> {
    /// A reference to the instance Storage
    pub storage: &'a Storage,
    /// A reference to the instance Blockchain
    pub blockchain: &'a Blockchain,
}

/// `OwnedContext` owns the instances.
/// It designed to allow easy dependency injection at runtime.
pub struct OwnedContext {
    /// The instance of Storage
    pub storage: Storage,
    /// The instance of Blockchain
    pub blockchain: Blockchain,
}

impl OwnedContext {
    /// returns the context as reference
    pub fn as_ref(&self) -> Context<'_> {
        Context {
            storage: &self.storage,
            blockchain: &self.blockchain,
        }
    }
}
