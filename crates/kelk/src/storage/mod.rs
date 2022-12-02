//! Modules definition for storage libraries

pub mod bst;
pub mod codec;
pub mod error;
pub mod hash_table;
pub mod linked_list;
pub mod mock;
pub mod str;
pub mod vec;

/// is an alias for representing the offset of the allocated space inside the storage file.
pub type Offset = u32;

/// is the size of offset in bytes.
const OFFSET_SIZE: u32 = 4;

use self::codec::Codec;
use self::error::Error;
use alloc::boxed::Box;
use core::result::Result;
use kelk_env::StorageAPI;

macro_rules! impl_num {
    ($ty:ty, $size:literal, $read_fn:ident, $write_fn:ident) => {
        doc_comment! {
            concat!("reads ", stringify!($size), " byte(s) from storage file at the given `offset` and converts it to ", stringify!($ty),"."
            ),
            #[inline]
            pub fn $read_fn(&self, offset: Offset) -> Result<$ty,Error> {
                self.read::<$ty>(offset)
            }
        }

        doc_comment! {
                concat!("converts ", stringify!($ty)," to ", stringify!($size), " byte(s) and writes it into storage file at the given `offset`."
                ),
            #[inline]
            pub fn $write_fn(&self, offset: Offset, value: &$ty) -> Result<(),Error> {
                self.write(offset, value)
            }
        }
    };
}

/// Storage object
pub struct Storage {
    /// Storage APIs that are provided by the host
    api: Box<dyn StorageAPI>,

    stack_size: u16,
    allocation_offset: Offset,
}

impl Storage {
    /// creates a new instance of storage
    pub fn create(api: Box<dyn StorageAPI>) -> Result<Self, Error> {
        let mut storage = Storage {
            api,
            stack_size: 0,
            allocation_offset: 0,
        };
        storage.stack_size = 32;
        storage.allocation_offset = (storage.stack_size as u32 * Offset::PACKED_LEN) + 8;

        storage.write_u16(0, &1)?; // version
        storage.write_u16(2, &storage.stack_size)?;
        storage.write(storage.allocation_offset, &(storage.allocation_offset + 4))?;

        Ok(storage)
    }

    ///
    pub fn load(api: Box<dyn StorageAPI>) -> Result<Self, Error> {
        let mut storage = Storage {
            api,
            stack_size: 0,
            allocation_offset: 0,
        };

        let ver = storage.read_u16(0)?;
        if ver != 1 {
            return Err(Error::InvalidOffset(0));
        }
        let stack_size = storage.read_u16(2)?;

        storage.stack_size = stack_size;
        storage.allocation_offset = (storage.stack_size as u32 * Offset::PACKED_LEN) + 8;

        Ok(storage)
    }

    pub(crate) fn api_mut(&mut self) -> &mut Box<dyn StorageAPI> {
        &mut self.api
    }

    /// Allocates storage space with the specific `length` and returns the
    /// offset of allocated space in the storage file.
    pub fn allocate(&self, length: u32) -> Result<Offset, Error> {
        let cur_free_pos = self.read(self.allocation_offset)?;
        let next_free_pos = cur_free_pos + length;

        // Updating allocation pos
        self.write(self.allocation_offset, &next_free_pos)?;

        Ok(cur_free_pos)
    }

    /// Deallocates the storage space at the specific `offset` and `length`
    pub fn deallocate(&self, _offset: Offset, _length: u32) -> Result<(), Error> {
        todo!()
    }

    /// Reallocates the the storage space at the given `offset` from `length` to `new_length`
    /// and returns the new offset of reallocated area.
    pub fn reallocate(
        &self,
        _offset: Offset,
        _length: u32,
        _new_length: u32,
    ) -> Result<Offset, Error> {
        todo!()
    }

    fn stack_offset(&self, stack_index: u16) -> Result<Offset, Error> {
        if stack_index > self.stack_size {
            return Err(Error::StackOverflow);
        }

        // stack_offset = (stack_index * 4) + 4
        let header_size = 4;
        Ok((stack_index as u32 * Offset::PACKED_LEN) + header_size)
    }

    ///
    pub fn fill_stack_at(&self, stack_index: u16, offset: Offset) -> Result<(), Error> {
        self.write_u32(self.stack_offset(stack_index)?, &offset)
    }

    ///
    pub fn read_stack_at(&self, stack_index: u16) -> Result<Offset, Error> {
        self.read_u32(self.stack_offset(stack_index)?)
    }

    impl_num!(u8, 1, read_u8, write_u8);
    impl_num!(u16, 2, read_u16, write_u16);
    impl_num!(u32, 4, read_u32, write_u32);
    impl_num!(u64, 8, read_u64, write_u64);
    impl_num!(u128, 8, read_u128, write_u128);

    impl_num!(i8, 1, read_i8, write_i8);
    impl_num!(i16, 2, read_i16, write_i16);
    impl_num!(i32, 4, read_i32, write_i32);
    impl_num!(i64, 8, read_i64, write_i64);
    impl_num!(i128, 16, read_i128, write_i128);

    impl_num!(bool, 1, read_bool, write_bool);

    /// Reads `T` from the storage file at the given `offset`.
    /// Note that `T` should be `Codec`.
    #[inline]
    pub(crate) fn read<T: Codec>(&self, offset: Offset) -> Result<T, Error> {
        let mut bytes = alloc::vec![0; T::PACKED_LEN as usize];
        self.api.read(offset, &mut bytes)?;
        let value = T::from_bytes(&bytes);
        Ok(value)
    }

    /// Writes `T` to the storage file at the given `offset`.
    /// Note that `T` should be `Codec`.
    #[inline]
    pub(crate) fn write<T: Codec>(&self, offset: Offset, value: &T) -> Result<(), Error> {
        let mut bytes = alloc::vec![0; T::PACKED_LEN as usize];
        value.to_bytes(&mut bytes);
        Ok(self.api.write(offset, &bytes)?)
    }

    /// Reads slice of bytes of size `length` from the storage file at the given `offset`.
    #[inline]
    pub(crate) fn read_bytes(&self, offset: Offset, data: &mut [u8]) -> Result<(), Error> {
        Ok(self.api.read(offset, data)?)
    }

    /// Writes bytes slice to the storage file at the given `offset`.
    #[inline]
    pub(crate) fn write_bytes(&self, offset: Offset, data: &[u8]) -> Result<(), Error> {
        Ok(self.api.write(offset, data)?)
    }
}

#[cfg(test)]
pub mod tests {
    use super::Storage;
    use crate::storage::codec::Codec;
    use crate::storage::mock::mock_storage;
    use crate::Codec;

    #[test]
    fn test_storage_load() {
        let storage_1 = mock_storage(1024);
        assert!(Storage::load(storage_1.api).is_ok());
    }

    #[test]
    fn test_unsigned_integers() {
        let storage = mock_storage(1024);

        let offset1 = storage.allocate(u8::PACKED_LEN).unwrap();
        let offset2 = storage.allocate(u16::PACKED_LEN).unwrap();
        let offset3 = storage.allocate(u32::PACKED_LEN).unwrap();
        let offset4 = storage.allocate(u64::PACKED_LEN).unwrap();
        let offset5 = storage.allocate(u128::PACKED_LEN).unwrap();

        storage.write_u8(offset1, &1).unwrap();
        storage.write_u16(offset2, &2).unwrap();
        storage.write_u32(offset3, &3).unwrap();
        storage.write_u64(offset4, &4).unwrap();
        storage.write_u128(offset5, &5).unwrap();

        assert_eq!(storage.read_u8(offset1).unwrap(), 1);
        assert_eq!(storage.read_u16(offset2).unwrap(), 2);
        assert_eq!(storage.read_u32(offset3).unwrap(), 3);
        assert_eq!(storage.read_u64(offset4).unwrap(), 4);
        assert_eq!(storage.read_u128(offset5).unwrap(), 5);
    }

    #[test]
    fn test_signed_integers() {
        let storage = mock_storage(1024);

        let offset1 = storage.allocate(i8::PACKED_LEN).unwrap();
        let offset2 = storage.allocate(i16::PACKED_LEN).unwrap();
        let offset3 = storage.allocate(i32::PACKED_LEN).unwrap();
        let offset4 = storage.allocate(i64::PACKED_LEN).unwrap();
        let offset5 = storage.allocate(i128::PACKED_LEN).unwrap();

        storage.write_i8(offset1, &-1).unwrap();
        storage.write_i16(offset2, &-2).unwrap();
        storage.write_i32(offset3, &-3).unwrap();
        storage.write_i64(offset4, &-4).unwrap();
        storage.write_i128(offset5, &-5).unwrap();

        assert_eq!(storage.read_i8(offset1).unwrap(), -1);
        assert_eq!(storage.read_i16(offset2).unwrap(), -2);
        assert_eq!(storage.read_i32(offset3).unwrap(), -3);
        assert_eq!(storage.read_i64(offset4).unwrap(), -4);
        assert_eq!(storage.read_i128(offset5).unwrap(), -5);
    }

    #[test]
    fn test_bool() {
        let storage = mock_storage(1024);

        let offset1 = storage.allocate(bool::PACKED_LEN).unwrap();
        let offset2 = storage.allocate(bool::PACKED_LEN).unwrap();

        storage.write_bool(offset1, &true).unwrap();
        storage.write_bool(offset2, &false).unwrap();

        assert!(storage.read_bool(offset1).unwrap());
        assert!(!storage.read_bool(offset2).unwrap());
    }

    #[test]
    fn test_struct() {
        use self::Codec;

        #[derive(Debug, PartialEq, Codec, Clone)]
        struct Test {
            foo: i16,
            bar: i8,
            zoo: i32,
        }

        let storage = mock_storage(1024);
        let foo_1 = Test {
            foo: 123,
            bar: 7,
            zoo: 1024,
        };

        let offset = storage.allocate(Test::PACKED_LEN).unwrap();

        storage.write(offset, &foo_1).unwrap();
        let foo_2 = storage.read::<Test>(offset).unwrap();
        assert_eq!(foo_1, foo_2);
    }
}
