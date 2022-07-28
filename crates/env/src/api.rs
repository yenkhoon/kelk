//! Defining the Kelk API trait.

use core::any::Any;

use crate::error::HostError;
use alloc::vec::Vec;

/// the storage APIs that should be provided by the host.
/// It can't be copied or cloned since it doesn't have Copy and Clone traits.
pub trait StorageAPI {
    /// This API requests the host to read the slice of `data` from the storage file
    /// at the given `offset`.
    fn read(&self, offset: u32, data: &mut [u8]) -> Result<(), HostError>;

    /// This API requests the host to write  the slice of `data` into the storage file
    /// at the given `offset`
    fn write(&self, offset: u32, data: &[u8]) -> Result<(), HostError>;

    /// It is useful for downcasting the trait to the underling struct.
    /// For example we can downcast the trait to the mocked object.
    fn as_any(&mut self) -> &mut dyn Any;
}

/// the blockchain APIs that should be provided by the host.
/// It can't be copied or cloned since it doesn't have Copy and Clone traits.
pub trait BlockchainAPI {
    /// This API requests the host to return the associated value to the given
    /// `param_id`.
    fn get_param(&self, param_id: u32) -> Result<Vec<u8>, HostError>;

    /// It is useful for downcasting the trait to the underling struct.
    /// For example we can downcast the trait to the mocked object.
    fn as_any(&mut self) -> &mut dyn Any;
}
