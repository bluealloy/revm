mod body;
mod decode_helpers;
mod header;
mod types_section;

pub use body::EofBody;
pub use header::EofHeader;
pub use types_section::TypesSection;

use core::cmp::min;
use primitives::{b256, bytes, Bytes, B256};
use std::{fmt, vec, vec::Vec};

/// Hash of EF00 bytes that is used for EXTCODEHASH when called from legacy bytecode.
pub const EOF_MAGIC_HASH: B256 =
    b256!("9dbf3648db8210552e9c4f75c6a1c3057c0ca432043bd648be15fe7be05646f5");

/// EOF Magic in u16 form.
pub const EOF_MAGIC: u16 = 0xEF00;

/// EOF magic number in array form.
pub static EOF_MAGIC_BYTES: Bytes = bytes!("ef00");

/// EVM Object Format (EOF) container.
///
/// It consists of a header, body and the raw original bytes.
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
            code_section: vec![Bytes::from_static(&[0x00])],
            container_section: vec![],
            data_section: Bytes::new(),
            is_data_filled: true,
        };
        body.into_eof()
    }
}

impl Eof {
    /// Creates a new EOF container from the given body.
    pub fn new(body: EofBody) -> Self {
        body.into_eof()
    }

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
    pub fn decode_dangling(mut raw: Bytes) -> Result<(Self, Bytes), EofDecodeError> {
        let (header, _) = EofHeader::decode(&raw)?;
        let eof_size = header.body_size() + header.size();
        if eof_size > raw.len() {
            return Err(EofDecodeError::MissingInput);
        }
        let dangling_data = raw.split_off(eof_size);
        let body = EofBody::decode(&raw, &header)?;
        Ok((Self { header, body, raw }, dangling_data))
    }

    /// Decode EOF from raw bytes.
    pub fn decode(raw: Bytes) -> Result<Self, EofDecodeError> {
        let (header, _) = EofHeader::decode(&raw)?;
        let body = EofBody::decode(&raw, &header)?;
        Ok(Self { header, body, raw })
    }
}

/// EOF decode errors.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Invalid initcode size.
    InvalidEOFSize,
}

impl fmt::Display for EofDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::MissingInput => "Short input while processing EOF",
            Self::MissingBodyWithoutData => "Short body while processing EOF",
            Self::DanglingData => "Body size is more than specified in the header",
            Self::InvalidTypesSection => "Invalid types section data",
            Self::InvalidTypesSectionSize => "Invalid types section size",
            Self::InvalidEOFMagicNumber => "Invalid EOF magic number",
            Self::InvalidEOFVersion => "Invalid EOF version",
            Self::InvalidTypesKind => "Invalid number for types kind",
            Self::InvalidCodeKind => "Invalid number for code kind",
            Self::InvalidTerminalByte => "Invalid terminal code",
            Self::InvalidDataKind => "Invalid data kind",
            Self::InvalidKindAfterCode => "Invalid kind after code",
            Self::MismatchCodeAndTypesSize => "Mismatch of code and types sizes",
            Self::NonSizes => "There should be at least one size",
            Self::ShortInputForSizes => "Missing size",
            Self::ZeroSize => "Size cant be zero",
            Self::TooManyCodeSections => "Invalid code number",
            Self::ZeroCodeSections => "Invalid number of code sections",
            Self::TooManyContainerSections => "Invalid container number",
            Self::InvalidEOFSize => "Invalid initcode size",
        };
        f.write_str(s)
    }
}

impl core::error::Error for EofDecodeError {}

#[cfg(test)]
mod test {

    use super::*;
    use primitives::bytes;

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

        const EMPTY: &[u8] = &[];
        assert_eq!(eof.data_slice(10, 2), EMPTY);
        assert_eq!(eof.data_slice(1, 0), EMPTY);
        assert_eq!(eof.data_slice(10, 0), EMPTY);
    }
}
