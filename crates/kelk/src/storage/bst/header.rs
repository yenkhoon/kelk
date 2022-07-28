use crate::storage::codec::Codec;
use crate::storage::Offset;
use crate::Codec;

#[derive(Codec)]
pub(super) struct Header {
    pub key_len: u16,
    pub value_len: u16,
    pub count: u32,
    pub root_offset: Offset,
}

impl Header {
    pub fn new<K: Codec, V: Codec>() -> Self {
        Self {
            key_len: K::PACKED_LEN as u16,
            value_len: V::PACKED_LEN as u16,
            count: 0,
            root_offset: 0,
        }
    }
}
