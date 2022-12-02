//! Mocking the storage for testing purpose

use super::{Offset, Storage};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::{any::Any, cell::RefCell, result::Result};
use kelk_env::{HostError, StorageAPI};

/// mocks the storage for testing purpose.
pub struct MockStorage {
    storage: RefCell<Vec<u8>>,
}

impl MockStorage {
    /// instantiates a new storage mock
    pub fn new(size: usize) -> Self {
        let storage = RefCell::new(alloc::vec![0; size].to_vec());
        Self { storage }
    }
}

impl StorageAPI for MockStorage {
    fn read(&self, offset: Offset, data: &mut [u8]) -> Result<(), HostError> {
        if offset as usize + data.len() > self.storage.borrow().len() {
            return Err(HostError { code: -1 });
        }

        unsafe {
            data.as_mut_ptr().copy_from(
                self.storage.borrow().as_ptr().offset(offset as isize),
                data.len(),
            )
        };
        Ok(())
    }

    fn write(&self, offset: Offset, data: &[u8]) -> Result<(), HostError> {
        if offset as usize + data.len() > self.storage.borrow().len() {
            return Err(HostError { code: -1 });
        }

        unsafe {
            data.as_ptr().copy_to(
                self.storage
                    .borrow_mut()
                    .as_mut_ptr()
                    .offset(offset as isize),
                data.len(),
            )
        };

        Ok(())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

/// mocks the storage for testing
pub fn mock_storage(storage_size: usize) -> Storage {
    Storage::create(Box::new(MockStorage::new(storage_size))).unwrap()
}
