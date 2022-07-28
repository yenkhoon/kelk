//! Storage Binary Search Tree
//!
//! Storage Binary Search Tree, is a Binary Search Tree or BST that instead of using Random Access Memory (RAM),
//! it uses storage file. Therefore it's permanently stored inside contract's storage.

mod header;
mod node;

use self::header::Header;
use self::node::Node;
use crate::storage::codec::Codec;
use crate::storage::error::Error;
use crate::storage::{Offset, Storage};
use core::marker::PhantomData;
use core::result::Result;

/// The instance of Storage Binary Search Tree
pub struct StorageBST<'a, K, V>
where
    K: Codec + Ord,
    V: Codec,
{
    storage: &'a Storage,
    offset: Offset,
    header: Header,
    _phantom: PhantomData<(K, V)>,
}

impl<'a, K, V> StorageBST<'a, K, V>
where
    K: Codec + Ord,
    V: Codec,
{
    /// Creates a new instance of `StorageBST`.
    pub fn create(storage: &'a Storage) -> Result<Self, Error> {
        let offset = storage.allocate(Header::PACKED_LEN)?;
        let header = Header::new::<K, V>();
        storage.write(offset, &header)?;

        Ok(StorageBST {
            storage,
            offset,
            header,
            _phantom: PhantomData,
        })
    }

    /// Loads the Storage Binary Search Tree
    pub fn load(storage: &'a Storage, offset: u32) -> Result<Self, Error> {
        let header: Header = storage.read(offset)?;

        debug_assert_eq!(header.key_len, K::PACKED_LEN as u16);
        debug_assert_eq!(header.value_len, V::PACKED_LEN as u16);

        Ok(StorageBST {
            storage,
            offset,
            header,
            _phantom: PhantomData,
        })
    }
    /// Returns the offset of `StorageLinkedList` in the storage file.
    pub fn offset(&self) -> Offset {
        self.offset
    }

    /// Inserts a key-value pair into the tree.
    /// If the map did not have this key present, None is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, Error> {
        if self.header.count == 0 {
            // create a root node
            let offset = self.storage.allocate(Node::<K, V>::PACKED_LEN)?;
            let root = Node::new(key, value);

            self.header.count = 1;
            self.header.root_offset = offset;

            self.storage.write(self.offset, &self.header)?;
            self.storage.write(offset, &root)?;
            Ok(None)
        } else {
            let mut offset = self.header.root_offset;
            let mut node: Node<K, V> = self.storage.read(offset)?;

            loop {
                if node.key.eq(&key) {
                    let old_value = node.value;
                    node.value = value;

                    // node exists, update value
                    self.storage.write(offset, &node)?;
                    return Ok(Some(old_value));
                } else if node.key.le(&key) {
                    if node.left.eq(&0) {
                        let new_offset = self.storage.allocate(Node::<K, V>::PACKED_LEN)?;
                        let new_node = Node::new(key, value);

                        // update header
                        self.header.count += 1;
                        self.storage.write(self.offset, &self.header)?;

                        // update parent node
                        node.left = new_offset;
                        self.storage.write(offset, &node)?;

                        // write new node
                        self.storage.write(new_offset, &new_node)?;
                        return Ok(None);
                    }
                    offset = node.left;
                } else {
                    if node.right.eq(&0) {
                        let new_offset = self.storage.allocate(Node::<K, V>::PACKED_LEN)?;
                        let new_node = Node::new(key, value);

                        // update header
                        self.header.count += 1;
                        self.storage.write(self.offset, &self.header)?;

                        // update parent node
                        node.right = new_offset;
                        self.storage.write(offset, &node)?;

                        // write new node
                        self.storage.write(new_offset, &new_node)?;
                        return Ok(None);
                    }
                    offset = node.right;
                }
                node = self.storage.read(offset)?;
            }
        }
    }

    /// Returns the value corresponding to the key. If the key doesn't exists, it returns None.
    pub fn find(&self, key: &K) -> Result<Option<V>, Error> {
        if self.header.count == 0 {
            return Ok(None);
        }

        let mut offset = self.header.root_offset;
        let mut node: Node<K, V> = self.storage.read(offset)?;

        loop {
            if node.key.eq(key) {
                return Ok(Some(node.value));
            } else if node.key.le(key) {
                if node.left.eq(&0) {
                    return Ok(None);
                }
                offset = node.left;
            } else {
                if node.right.eq(&0) {
                    return Ok(None);
                }
                offset = node.right;
            }
            node = self.storage.read(offset)?;
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
    fn test_bst() {
        let storage = mock_storage(1024);
        let mut bst_1 = StorageBST::<i32, i64>::create(&storage).unwrap();

        assert_eq!(None, bst_1.insert(1, 10).unwrap());
        assert_eq!(None, bst_1.insert(3, 30).unwrap());
        assert_eq!(None, bst_1.insert(2, 20).unwrap());
        assert_eq!(Some(10), bst_1.insert(1, 100).unwrap());

        let bst_2 = StorageBST::<i32, i64>::load(&storage, bst_1.offset()).unwrap();
        assert_eq!(Some(20), bst_2.find(&2).unwrap());
        assert_eq!(None, bst_2.find(&4).unwrap());
        assert_eq!(Some(30), bst_2.find(&3).unwrap());
        assert_eq!(Some(100), bst_2.find(&1).unwrap());
        assert!(!bst_2.contains_key(&-1).unwrap());
        assert!(bst_2.contains_key(&2).unwrap());
        assert!(!bst_2.contains_key(&4).unwrap());
    }
}
