//! Storage Hash Table
//!
//! `StorageHashTable` is an implementation of hash table that instead of using
//! Random Access Memory (RAM), it uses storage file. Therefore it's permanently
//!  stored inside contract's storage.
//!

mod header;

use self::header::Header;
use super::{bst, OFFSET_SIZE};
use crate::storage::codec::Codec;
use crate::storage::error::Error;
use crate::storage::{Offset, Storage};
use core::hash::{Hash, Hasher};
use core::marker::PhantomData;

fn compute_hash<K: Hash>(key: &K) -> u32 {
    let mut state = fnv::FnvHasher::default();
    key.hash(&mut state);
    state.finish() as u32
}
/// The instance of `StorageLinkedList`
pub struct StorageHashTable<'a, K, V>
where
    K: Codec + Ord + Hash + Eq,
    V: Codec,
{
    storage: &'a Storage,
    // Offset of the header in the storage file.
    header_offset: Offset,
    // In memory instance of the header.
    // Any change in the header should be flushed into the storage file
    header: Header,
    _phantom: PhantomData<(K, V)>,
}

impl<'a, K, V> StorageHashTable<'a, K, V>
where
    K: Codec + Ord + Hash + Eq,
    V: Codec,
{
    /// Creates a new instance of `StorageHashTable`.
    pub fn create(storage: &'a Storage, table_size: u32) -> Result<Self, Error> {
        let header_offset = storage.allocate(Header::PACKED_LEN)?;
        let table_offset = storage.allocate(table_size * OFFSET_SIZE)?;
        let header = Header::new::<K, V>(table_size, table_offset);
        storage.write(header_offset, &header)?;

        Ok(StorageHashTable {
            storage,
            header_offset,
            header,
            _phantom: PhantomData,
        })
    }

    /// Try to load the `StorageHashTable` at the given offset in the storage file.
    pub fn load(storage: &'a Storage, offset: Offset) -> Result<Self, Error> {
        let header: Header = storage.read(offset)?;

        debug_assert_eq!(header.key_len, K::PACKED_LEN as u16);
        debug_assert_eq!(header.value_len, V::PACKED_LEN as u16);

        Ok(StorageHashTable {
            storage,
            header_offset: offset,
            header,
            _phantom: PhantomData,
        })
    }

    /// Returns the offset of `StorageHashTable` in the storage file.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn offset(&self) -> Offset {
        self.header_offset
    }

    /// Returns the number of elements in the `StorageHashTable`.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> u32 {
        self.header.items
    }

    /// Returns `true` if the `StorageHashTable` contains no elements.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Inserts a key-value pair into the tree.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, Error> {
        let hash = compute_hash(&key);
        let bucket = hash % self.header.table_size;
        let bucket_offset = self.header.table_offset + (bucket * OFFSET_SIZE);
        let bst_offset = self.storage.read_u32(bucket_offset)?;
        if bst_offset == 0 {
            let mut bst = bst::StorageBST::create(self.storage)?;
            bst.insert(key, value)?;
            self.storage.write(bucket_offset, &bst.offset())?;
            self.header.items += 1;
            self.storage.write(self.header_offset, &self.header)?;

            Ok(None)
        } else {
            let mut bst = bst::StorageBST::load(self.storage, bst_offset)?;
            let item = bst.insert(key, value)?;
            if item.is_none() {
                self.header.items += 1;
                self.storage.write(self.header_offset, &self.header)?;
            }

            Ok(item)
        }
    }

    /// Returns the value corresponding to the key. If the key doesn't exists, it returns None.
    pub fn find(&self, key: &K) -> Result<Option<V>, Error> {
        if self.header.items == 0 {
            return Ok(None);
        }
        let hash = compute_hash(key);
        let bucket = hash % self.header.table_size;
        let bucket_offset = self.header.table_offset + (bucket * OFFSET_SIZE);
        let bst_offset = self.storage.read_u32(bucket_offset)?;
        if bst_offset == 0 {
            Ok(None)
        } else {
            let bst = bst::StorageBST::load(self.storage, bst_offset)?;
            bst.find(key)
        }
    }

    /// Returns true if the tree contains a value for the specified key.
    pub fn contains_key(&self, key: &K) -> Result<bool, Error> {
        Ok(self.find(key)?.is_some())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::storage::mock::mock_storage;

    #[test]
    fn test_hash_table() {
        let storage = mock_storage(1024);
        let mut ht_1 = StorageHashTable::<i32, i64>::create(&storage, 64).unwrap();

        assert!(ht_1.is_empty());
        assert_eq!(None, ht_1.insert(1, 10).unwrap());
        assert_eq!(None, ht_1.insert(3, 30).unwrap());
        assert_eq!(None, ht_1.insert(2, 20).unwrap());
        assert_eq!(Some(10), ht_1.insert(1, 100).unwrap());

        let ht_2 = StorageHashTable::<i32, i64>::load(&storage, ht_1.offset()).unwrap();
        assert_eq!(3, ht_2.len());
        assert_eq!(Some(20), ht_2.find(&2).unwrap());
        assert_eq!(None, ht_2.find(&4).unwrap());
        assert_eq!(Some(30), ht_2.find(&3).unwrap());
        assert_eq!(Some(100), ht_2.find(&1).unwrap());
        assert!(!ht_2.contains_key(&-1).unwrap());
        assert!(ht_2.contains_key(&2).unwrap());
        assert!(!ht_2.contains_key(&4).unwrap());
    }
}
