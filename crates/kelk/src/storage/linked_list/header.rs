use crate::storage::codec::Codec;
use crate::storage::Offset;
use crate::Codec;

#[derive(Codec)]

pub(super) struct Header {
    // Number of items in the `StorageLinkedList`, only really used by len().
    pub items: u32,
    // How many bytes is the item when it is packed with the `Codec`.
    pub item_len: u16,
    // Offset of the head item in the storage file.
    // It set to zero when the `StorageLinkedList` is empty.
    pub head_offset: Offset,
    // Offset of the tail item in the storage file.
    // It set to zero when the `StorageLinkedList` is empty
    pub tail_offset: Offset,
}

impl Header {
    pub fn new<T: Codec>() -> Self {
        Self {
            items: 0,
            item_len: T::PACKED_LEN as u16,
            head_offset: 0,
            tail_offset: 0,
        }
    }
}
