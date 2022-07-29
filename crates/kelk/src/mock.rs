//! Mocking the `Context` for testing purpose

use crate::{
    blockchain::{mock::MockBlockchain, Blockchain},
    context::Context,
    storage::{mock::MockStorage, Storage},
};

/// `MockContext` owns the mocked instances.
pub struct MockContext {
    /// The instance of Storage
    pub storage: Storage,
    /// The instance of Blockchain
    pub blockchain: Blockchain,
}

impl MockContext {
    /// returns the context as reference
    pub fn as_ref(&self) -> Context<'_> {
        Context {
            storage: &self.storage,
            blockchain: &self.blockchain,
        }
    }

    /// returns a reference to the mocked storage
    pub fn mocked_storage(&mut self) -> &mut MockStorage {
        self.storage
            .api_mut()
            .as_any()
            .downcast_mut::<MockStorage>()
            .expect("Wasn't a trusty printer!")
    }

    /// returns a reference to the mocked blockchain
    pub fn mocked_blockchain(&mut self) -> &mut MockBlockchain {
        self.blockchain
            .api_mut()
            .as_any()
            .downcast_mut::<MockBlockchain>()
            .expect("Wasn't a trusty printer!")
    }
}

/// mocks the context for testing
pub fn mock_context(storage_size: usize) -> MockContext {
    use crate::{blockchain::mock::mock_blockchain, storage::mock::mock_storage};

    MockContext {
        blockchain: mock_blockchain(),
        storage: mock_storage(storage_size),
    }
}
