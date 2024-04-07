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
    pub fn size(&self) -> usize {
        self.header.size() + self.header.body_size()
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

    /// Re-encode the raw EOF bytes.
    pub fn reencode_inner(&mut self) {
        self.raw = Some(self.encode_slow())
    }

    /// Slow encode EOF bytes.
    pub fn encode_slow(&self) -> Bytes {
        let mut buffer: Vec<u8> = Vec::with_capacity(self.size());
        self.header.encode(&mut buffer);
        self.body.encode(&mut buffer);
        buffer.into()
    }

    /// Encode the EOF into bytes.
    pub fn encode(&self) -> Bytes {
        if let Some(raw) = &self.raw {
            raw.clone()
        } else {
            self.encode_slow()
        }
    }

    pub fn decode(raw: Bytes) -> Result<Self, EofDecodeError> {
        let (header, _) = EofHeader::decode(&raw)?;
        let body = EofBody::decode(&raw, &header)?;
        Ok(Self {
            header,
            body,
            raw: Some(raw),
        })
    }
}

/// EOF decode errors.
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EofDecodeError {
    /// Short input while processing EOF.
    MissingInput,
    /// Short body while processing EOF.
    MissingBodyWithoutData,
    /// Body size is more than specified in the header.
    DanglingData,
    /// Invalid types section data.
    InvalidTypesSection,
    /// Invalid types section size.
    InvalidTypesSectionSize,
    /// Invalid EOF magic number.
    InvalidEOFMagicNumber,
    /// Invalid EOF version.
    InvalidEOFVersion,
    /// Invalid number for types kind
    InvalidTypesKind,
    /// Invalid number for code kind
    InvalidCodeKind,
    /// Invalid terminal code
    InvalidTerminalByte,
    /// Invalid data kind
    InvalidDataKind,
    /// Invalid kind after code
    InvalidKindAfterCode,
    /// Mismatch of code and types sizes.
    MismatchCodeAndTypesSize,
    /// There should be at least one size.
    NonSizes,
    /// Missing size.
    ShortInputForSizes,
    /// Size cant be zero
    ZeroSize,
    /// Invalid code number.
    TooManyCodeSections,
    /// Invalid number of code sections.
    ZeroCodeSections,
    /// Invalid container number.
    TooManyContainerSections,
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::bytes;

    #[test]
    fn decode_eof() {
        let bytes = bytes!("ef000101000402000100010400000000800000fe");
        let eof = Eof::decode(bytes.clone()).unwrap();
        assert_eq!(bytes, eof.encode_slow());
    }

    #[test]
    fn data_slice() {
        let bytes = bytes!("ef000101000402000100010400000000800000fe");
        let mut eof = Eof::decode(bytes.clone()).unwrap();
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