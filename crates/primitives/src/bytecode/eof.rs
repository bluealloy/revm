mod body;
mod decode_helpers;
mod header;
mod types_section;

pub use body::EofBody;
pub use header::EofHeader;
pub use types_section::TypesSection;

use crate::Bytes;
use core::cmp::min;
use std::{vec, vec::Vec};

/// EOF - Ethereum Object Format.
///
/// It consist of a header, body and raw original bytes Specified in EIP.
/// Most of body contain Bytes so it references to the raw bytes.
///
/// If there is a need to create new EOF from scratch, it is recommended to use `EofBody` and
/// use `encode` function to create full [`Eof`] object.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eof {
    pub header: EofHeader,
    pub body: EofBody,
    pub raw: Bytes,
}

impl Default for Eof {
    fn default() -> Self {
        let body = EofBody {
            // types section with zero inputs, zero outputs and zero max stack size.
            types_section: vec![TypesSection::default()],
            // One code section with a STOP byte.
            code_section: vec![[0x00].into()],
            container_section: vec![],
            data_section: Bytes::new(),
            is_data_filled: true,
        };
        body.into_eof()
    }
}

impl Eof {
    /// Returns len of the header and body in bytes.
    pub fn size(&self) -> usize {
        self.header.size() + self.header.body_size()
    }

    /// Return raw EOF bytes.
    pub fn raw(&self) -> &Bytes {
        &self.raw
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

    /// Returns a slice of the data section.
    pub fn data(&self) -> &[u8] {
        &self.body.data_section
    }

    /// Slow encode EOF bytes.
    pub fn encode_slow(&self) -> Bytes {
        let mut buffer: Vec<u8> = Vec::with_capacity(self.size());
        self.header.encode(&mut buffer);
        self.body.encode(&mut buffer);
        buffer.into()
    }

    /// Decode EOF that have additional dangling bytes.
    /// Assume that data section is fully filled.
    pub fn decode_dangling(mut eof: Bytes) -> Result<(Self, Bytes), EofDecodeError> {
        let (header, _) = EofHeader::decode(&eof)?;
        let eof_size = header.body_size() + header.size();
        if eof_size > eof.len() {
            return Err(EofDecodeError::MissingInput);
        }
        let dangling_data = eof.split_off(eof_size);
        let body = EofBody::decode(&eof, &header)?;
        Ok((
            Self {
                header,
                body,
                raw: eof,
            },
            dangling_data,
        ))
    }

    /// Decode EOF from raw bytes.
    pub fn decode(raw: Bytes) -> Result<Self, EofDecodeError> {
        let (header, _) = EofHeader::decode(&raw)?;
        let body = EofBody::decode(&raw, &header)?;
        Ok(Self { header, body, raw })
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
    fn decode_eof_dangling() {
        let test_cases = [
            (
                bytes!("ef000101000402000100010400000000800000fe"),
                bytes!("010203"),
                false,
            ),
            (
                bytes!("ef000101000402000100010400000000800000fe"),
                bytes!(""),
                false,
            ),
            (
                bytes!("ef000101000402000100010400000000800000"),
                bytes!(""),
                true,
            ),
        ];

        for (eof_bytes, dangling_data, is_err) in test_cases {
            let mut raw = eof_bytes.to_vec();
            raw.extend(&dangling_data);
            let raw = Bytes::from(raw);

            let result = Eof::decode_dangling(raw.clone());
            assert_eq!(result.is_err(), is_err);
            if is_err {
                continue;
            }
            let (decoded_eof, decoded_dangling) = result.unwrap();
            assert_eq!(eof_bytes, decoded_eof.encode_slow());
            assert_eq!(decoded_dangling, dangling_data);
        }
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
