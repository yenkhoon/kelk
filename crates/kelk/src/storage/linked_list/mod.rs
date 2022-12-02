//! Storage Linked List
//!
//! Storage Linked List, is an implementation of singly linked list that instead
//! of using Random Access Memory (RAM), it uses storage file. Therefore it's
//! permanently stored inside contract's storage.
//!

mod header;

use self::header::Header;
use crate::storage::codec::Codec;
use crate::storage::error::Error;
use crate::storage::Offset;
use crate::storage::Storage;
use crate::Codec;
use core::iter::IntoIterator;
use core::marker::PhantomData;
use core::result::Result;

/// The instance of `StorageLinkedList`
pub struct StorageLinkedList<'a, T: Codec> {
    storage: &'a Storage,
    // Offset of the header in the storage file.
    header_offset: Offset,
    // In memory instance of the header.
    // Any change in the header should be flushed into the storage file
    header: Header,
    _phantom: PhantomData<T>,
}

#[derive(Codec)]
pub(self) struct Node<T: Codec> {
    pub item: T,
    pub next: Offset,
}

impl<T: Codec> Node<T> {
    pub fn new(item: T) -> Self {
        Self { item, next: 0 }
    }
}

impl<'a, T: Codec> StorageLinkedList<'a, T> {
    /// Creates a new instance of `StorageLinkedList`.
    pub fn create(storage: &'a Storage) -> Result<Self, Error> {
        let header_offset = storage.allocate(Header::PACKED_LEN)?;
        let header = Header::new::<T>();
        storage.write(header_offset, &header)?;

        Ok(StorageLinkedList {
            storage,
            header_offset,
            header,
            _phantom: PhantomData,
        })
    }

    /// Try to load the `StorageLinkedList` at the given offset in the storage file.
    pub fn load(storage: &'a Storage, offset: Offset) -> Result<Self, Error> {
        let header: Header = storage.read(offset)?;
        debug_assert_eq!(header.item_len, T::PACKED_LEN as u16);

        Ok(StorageLinkedList {
            storage,
            header_offset: offset,
            header,
            _phantom: PhantomData,
        })
    }

    /// Returns the offset of `StorageLinkedList` in the storage file.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn offset(&self) -> Offset {
        self.header_offset
    }

    /// Returns the number of elements in the `StorageLinkedList`.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn len(&self) -> u32 {
        self.header.items
    }

    /// Returns `true` if the `StorageLinkedList` contains no elements.
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes an item at the end of linked list.
    pub fn push_back(&mut self, item: T) -> Result<(), Error> {
        let offset = self.storage.allocate(Node::<T>::PACKED_LEN)?;
        let node = Node::new(item);

        if self.header.items == 0 {
            self.header.head_offset = offset;
        } else {
            let mut tail: Node<T> = self.storage.read(self.header.tail_offset)?;
            tail.next = offset;
            self.storage.write(self.header.tail_offset, &tail)?;
        }
        self.storage.write(offset, &node)?;

        self.header.items += 1;
        self.header.tail_offset = offset;
        self.storage.write(self.header_offset, &self.header)
    }
}

/// An iterator over the elements of a `StorageLinkedList`.
pub struct StorageLinkedListIter<'a, T> {
    storage: &'a Storage,
    cur_offset: Offset,
    _phantom: PhantomData<T>,
}

impl<'a, T: Codec + 'a> Iterator for StorageLinkedListIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_offset == 0 {
            None
        } else {
            let node: Node<T> = self.storage.read(self.cur_offset).unwrap();
            self.cur_offset = node.next;
            Some(node.item)
        }
    }
}

impl<'a, T: Codec> IntoIterator for &'a mut StorageLinkedList<'a, T> {
    type Item = T;
    type IntoIter = StorageLinkedListIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            storage: self.storage,
            cur_offset: self.header.head_offset,
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StorageLinkedList;
    use crate::storage::mock::mock_storage;
    use alloc::vec::Vec;

    #[test]
    fn test_linked_list() {
        let storage = mock_storage(4 * 1024);
        let mut list_1 = StorageLinkedList::<i32>::create(&storage).unwrap();
        assert!(list_1.is_empty());
        list_1.push_back(1).unwrap();
        list_1.push_back(2).unwrap();
        list_1.push_back(3).unwrap();

        let mut list_2 = StorageLinkedList::<i32>::load(&storage, list_1.offset()).unwrap();
        assert_eq!(list_2.len(), 3);
        let iter = list_2.into_iter();
        let all_items: Vec<i32> = iter.collect();
        assert!(all_items.eq(&[1, 2, 3]));
    }
}
