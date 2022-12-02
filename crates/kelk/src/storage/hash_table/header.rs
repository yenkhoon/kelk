use crate::storage::codec::Codec;
use crate::storage::Offset;
use crate::Codec;

#[derive(Codec)]
pub(super) struct Header {
    // Number of items in the `StorageHashTable`, only really used by len().
    pub items: u32,
    // How many bytes is the key when it is packed with the `Codec`.
    pub key_len: u16,
    // How many bytes is the value when it is packed with the `Codec`.
    pub value_len: u16,
    // The size of the hash table.
    pub table_offset: Offset,
    // The size of the hash table.
    pub table_size: u32,
}

impl Header {
    pub fn new<K: Codec, V: Codec>(table_size: u32, table_offset: Offset) -> Self {
        Self {
            items: 0,
            key_len: K::PACKED_LEN as u16,
            value_len: V::PACKED_LEN as u16,
            table_offset,
            table_size,
        }
    }
}
