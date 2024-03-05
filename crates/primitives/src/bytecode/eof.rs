mod body;
mod decode_helpers;
mod header;
mod types_section;

pub use body::EofBody;
pub use header::EofHeader;
pub use types_section::TypesSection;

use crate::Bytes;
use core::cmp::min;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eof {
    pub header: EofHeader,
    pub body: EofBody,
    pub raw: Option<Bytes>,
}

impl Default for Eof {
    fn default() -> Self {
        // TODO(EOF) make proper minimal EOF.
        Eof::decode("ef000101000402000100010400000000800000fe".into()).unwrap()
    }
}

impl Eof {
    /// Returns len of the header and body in bytes.
    pub fn len(&self) -> usize {
        self.header.len() + self.header.body_len()
    }

    pub fn raw(&self) -> Option<Bytes> {
        self.raw.clone()
    }

    /// Returns a slice of the raw bytes.
    /// If offset is greater than the length of the raw bytes, an empty slice is returned.
    /// If len is greater than the length of the raw bytes, the slice is truncated to the length of the raw bytes.
    pub fn data_slice(&self, offset: usize, len: usize) -> &[u8] {
        self.body
            .data_section
            .get(offset..)
            .and_then(|bytes| bytes.get(..min(len, bytes.len())))
            .unwrap_or(&[])
    }

    pub fn data(&self) -> &[u8] {
        &self.body.data_section
    }

    pub fn decode(raw: Bytes) -> Result<Self, ()> {
        let (header, _) = EofHeader::decode(&raw)?;
        let body = EofBody::decode(&raw, &header)?;
        Ok(Self {
            header,
            body,
            raw: Some(raw),
        })
    }

    /// TODO implement it.
    pub fn push_aux_data(&mut self, _aux_data: Bytes) {
        // Need to modify/replace raw Bytes, and recalculate body sections.
        // We can be little wasteful here and just replace the raw Bytes and
        // data section in the body. Other sections would still pin old raw Bytes.
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::bytes;

    #[test]
    fn decode_eof() {
        let bytes = alloy_primitives::bytes!("ef000101000402000100010400000000800000fe");
        Eof::decode(bytes).unwrap();
    }

    #[test]
    fn data_slice() {
        let bytes = alloy_primitives::bytes!("ef000101000402000100010400000000800000fe");
        let mut eof = Eof::decode(bytes).unwrap();
        eof.body.data_section = bytes!("01020304");
        assert_eq!(eof.data_slice(0, 1), &[0x01]);
        assert_eq!(eof.data_slice(0, 4), &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(eof.data_slice(0, 5), &[0x01, 0x02, 0x03, 0x04]);
        assert_eq!(eof.data_slice(1, 2), &[0x02, 0x03]);
        assert_eq!(eof.data_slice(10, 2), &[]);
        assert_eq!(eof.data_slice(1, 0), &[]);
        assert_eq!(eof.data_slice(10, 0), &[]);
    }
}
