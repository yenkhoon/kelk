//! Storage Vector
//!
//! Storage Vector, is a Vector or Array that instead of using Random Access Memory (RAM),
//! it uses storage file. Therefore it's permanently stored inside contract's storage.
//!

mod header;

use self::header::Header;
use crate::storage::codec::Codec;
use crate::storage::error::Error;
use crate::storage::Offset;
use crate::storage::Storage;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::result::Result;

/// The instance of Storage Vector
pub struct StorageVec<'a, T: Codec> {
    storage: &'a Storage,
    header: Header,
    offset: Offset,
    _phantom: PhantomData<T>,
}

impl<'a, T: Codec> StorageVec<'a, T> {
    /// creates and store a new instance of Storage Vector at the given offset
    pub fn create(storage: &'a Storage, capacity: u32) -> Result<Self, Error> {
        let offset = storage.allocate(Header::PACKED_LEN)?;
        let data_offset = storage.allocate(T::PACKED_LEN * capacity)?;
        let header = Header::new::<T>(capacity, data_offset);
        storage.write(offset, &header)?;

        Ok(StorageVec {
            storage,
            header,
            offset,
            _phantom: PhantomData,
        })
    }

    /// load the Storage Vector
    pub fn load(storage: &'a Storage, offset: Offset) -> Result<Self, Error> {
        let header: Header = storage.read(offset)?;

        debug_assert_eq!(header.value_len, T::PACKED_LEN as u16);

        Ok(StorageVec {
            storage,
            header,
            offset,
            _phantom: PhantomData,
        })
    }

    /// Returns the offset of `StorageVector` in the storage file.
    pub fn offset(&self) -> Offset {
        self.offset
    }

    /// Returns the number of elements in the `StorageVector`, also referred to as its ‘length’.
    pub fn len(&self) -> usize {
        self.header.count as usize
    }

    /// Returns the number of elements the `StorageVector` can hold without reallocating.
    pub fn capacity(&self) -> usize {
        self.header.capacity as usize
    }

    /// Returns `true` if the `StorageVector` contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Appends an element to the back of a `StorageVector`.
    pub fn push(&mut self, value: T) -> Result<(), Error> {
        if self.header.count >= self.header.capacity {
            return Err(Error::OutOfCapacity);
        }

        let offset = self.item_offset(self.header.count)?;
        self.storage.write(offset, &value)?;

        self.header.count += 1;
        self.storage.write(self.offset, &self.header)
    }

    /// Returns an element at the given index or None if out of bounds..
    pub fn get(&self, index: u32) -> Result<Option<T>, Error> {
        if index >= self.header.count {
            return Ok(None);
        }

        let offset = self.item_offset(index)?;
        let item = self.storage.read(offset)?;
        Ok(Some(item))
    }

    ///
    pub fn set_slice(&mut self, slice: &[T]) -> Result<(), Error> {
        if slice.len() > self.capacity() {
            return Err(Error::OutOfCapacity);
        }
        let mut offset = self.header.data_offset;
        for v in slice {
            self.storage.write(offset, v)?;
            offset += T::PACKED_LEN;
        }

        // update header
        self.header.count = slice.len() as u32;
        self.storage.write(self.offset, &self.header)
    }

    ///
    pub fn set_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        if bytes.len() > (self.capacity() * self.header.value_len as usize) {
            return Err(Error::OutOfCapacity);
        }

        self.storage.write_bytes(self.header.data_offset, bytes)?;

        // update header
        self.header.count = bytes.len() as u32 / self.header.value_len as u32;
        self.storage.write(self.offset, &self.header)
    }

    ///
    pub fn get_bytes(&self) -> Result<Vec<u8>, Error> {
        let length = self.header.count as usize * self.header.value_len as usize;
        let mut bytes = alloc::vec![0; length];
        self.storage
            .read_bytes(self.header.data_offset, &mut bytes)?;
        Ok(bytes)
    }

    fn item_offset(&self, index: u32) -> Result<Offset, Error> {
        Ok(self.header.data_offset + (index * self.header.value_len as u32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::mock::mock_storage;

    #[test]
    fn test_vector() {
        let storage = mock_storage(1024);
        let mut vec_1 = StorageVec::<i32>::create(&storage, 2).unwrap();
        vec_1.push(1).unwrap();
        vec_1.push(2).unwrap();
        assert!(vec_1.push(3).is_err());

        let vec_2 = StorageVec::<i32>::load(&storage, vec_1.offset()).unwrap();
        assert_eq!(Some(1), vec_2.get(0).unwrap());
        assert_eq!(Some(2), vec_2.get(1).unwrap());
        assert_eq!(None, vec_2.get(3).unwrap());
    }
}
