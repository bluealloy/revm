//! EOF bytecode.
//!
//! Contains body, header and raw bytes.
//!
//! Also contains verification logic and pretty printer.
mod body;
mod code_info;
mod decode_helpers;
mod header;
/// Pritty printer for the EOF bytecode. Enabled by `std` feature.
pub mod printer;
/// Verification logic for the EOF bytecode.
pub mod verification;

pub use body::EofBody;
pub use code_info::CodeInfo;
pub use header::{
    EofHeader, CODE_SECTION_SIZE, CONTAINER_SECTION_SIZE, KIND_CODE, KIND_CODE_INFO,
    KIND_CONTAINER, KIND_DATA, KIND_TERMINAL,
};
pub use verification::*;

use core::cmp::min;
use primitives::{b256, bytes, Bytes, B256};
use std::{fmt, vec, vec::Vec};

/// Hash of EF00 bytes that is used for EXTCODEHASH when called from legacy bytecode
pub const EOF_MAGIC_HASH: B256 =
    b256!("0x9dbf3648db8210552e9c4f75c6a1c3057c0ca432043bd648be15fe7be05646f5");

/// EOF Magic in [u16] form
pub const EOF_MAGIC: u16 = 0xEF00;

/// EOF magic number in array form
pub static EOF_MAGIC_BYTES: Bytes = bytes!("ef00");

/// EVM Object Format (EOF) container
///
/// It consists of a header, body and the raw original bytes.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eof {
    /// Header of the EOF container
    pub header: EofHeader,
    /// Body of the EOF container
    pub body: EofBody,
    /// Raw bytes of the EOF container. Chunks of raw Bytes are used in Body to reference
    /// parts of code, data and container sections.
    pub raw: Bytes,
}

impl Default for Eof {
    fn default() -> Self {
        let body = EofBody {
            // Types section with zero inputs, zero outputs and zero max stack size.
            code_info: vec![CodeInfo::default()],
            code_section: vec![1],
            // One code section with a STOP byte.
            code: Bytes::from_static(&[0x00]),
            code_offset: 0,
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

    /// Validates the EOF container.
    pub fn validate(&self) -> Result<(), EofError> {
        validate_eof(self)
    }

    /// Validates the raw EOF bytes.
    pub fn validate_raw(bytes: Bytes) -> Result<Eof, EofError> {
        validate_raw_eof(bytes)
    }

    /// Validates the EOF container with the given code type.   
    pub fn validate_mode(&self, mode: CodeType) -> Result<(), EofError> {
        validate_eof_inner(self, Some(mode))
    }

    /// Returns len of the header and body in bytes.
    pub fn size(&self) -> usize {
        self.header.size() + self.header.body_size()
    }

    /// Returns raw EOF bytes.
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

    /// Slow encodes EOF bytes.
    pub fn encode_slow(&self) -> Bytes {
        let mut buffer: Vec<u8> = Vec::with_capacity(self.size());
        self.header.encode(&mut buffer);
        self.body.encode(&mut buffer);
        buffer.into()
    }

    /// Decodes EOF that have additional dangling bytes.
    ///
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

    /// Decodes EOF from raw bytes.
    pub fn decode(raw: Bytes) -> Result<Self, EofDecodeError> {
        let (header, _) = EofHeader::decode(&raw)?;
        let body = EofBody::decode(&raw, &header)?;
        Ok(Self { header, body, raw })
    }
}

/// EOF decode errors
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EofDecodeError {
    /// Short input while processing EOF
    MissingInput,
    /// Short body while processing EOF
    MissingBodyWithoutData,
    /// Body size is more than specified in the header
    DanglingData,
    /// Invalid code info data
    InvalidCodeInfo,
    /// Invalid code info input value
    InvalidCodeInfoInputValue {
        /// Number of inputs
        value: u8,
    },
    /// Invalid code info input value
    InvalidCodeInfoOutputValue {
        /// Number of outputs
        value: u8,
    },
    /// Invalid code info input value
    InvalidCodeInfoMaxIncrementValue {
        /// MaxIncrementValue
        value: u16,
    },
    /// Invalid code info input value can't be greater than [`primitives::STACK_LIMIT`]
    InvalidCodeInfoStackOverflow {
        /// Number of inputs
        inputs: u8,
        /// Max stack increment
        max_stack_increment: u16,
    },
    /// Invalid code info size
    InvalidCodeInfoSize,
    /// Invalid EOF magic number
    InvalidEOFMagicNumber,
    /// Invalid EOF version
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
    InvalidKindAfterCode {
        /// Invalid unexpected kind type.
        invalid_kind: u8,
    },
    /// Mismatch of code and info sizes
    MismatchCodeAndInfoSize,
    /// There should be at least one size
    NonSizes,
    /// Missing size
    ShortInputForSizes,
    /// Size cant be zero
    ZeroSize,
    /// Invalid code number
    TooManyCodeSections,
    /// Invalid number of code sections
    ZeroCodeSections,
    /// Invalid container number
    TooManyContainerSections,
    /// Invalid initcode size
    InvalidEOFSize,
}

impl fmt::Display for EofDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::MissingInput => "Short input while processing EOF",
            Self::MissingBodyWithoutData => "Short body while processing EOF",
            Self::DanglingData => "Body size is more than specified in the header",
            Self::InvalidCodeInfo => "Invalid types section data",
            Self::InvalidCodeInfoInputValue { value } => {
                return write!(f, "Invalid code info input value: {}", value);
            }
            Self::InvalidCodeInfoOutputValue { value } => {
                return write!(f, "Invalid code info output value: {}", value);
            }
            Self::InvalidCodeInfoMaxIncrementValue { value } => {
                return write!(f, "Invalid code info max increment value: {}", value);
            }
            Self::InvalidCodeInfoStackOverflow {
                inputs,
                max_stack_increment,
            } => {
                return write!(
                    f,
                    "Invalid code info stack overflow: inputs: {}, max_stack_increment: {}",
                    inputs, max_stack_increment
                );
            }
            Self::InvalidCodeInfoSize => "Invalid types section size",
            Self::InvalidEOFMagicNumber => "Invalid EOF magic number",
            Self::InvalidEOFVersion => "Invalid EOF version",
            Self::InvalidTypesKind => "Invalid number for types kind",
            Self::InvalidCodeKind => "Invalid number for code kind",
            Self::InvalidTerminalByte => "Invalid terminal code",
            Self::InvalidDataKind => "Invalid data kind",
            Self::InvalidKindAfterCode { invalid_kind } => {
                return write!(f, "Invalid kind after code: {}", invalid_kind);
            }
            Self::MismatchCodeAndInfoSize => "Mismatch of code and types sizes",
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
        let bytes = bytes!("ef00010100040200010001ff00000000800000fe");
        let eof = Eof::decode(bytes.clone()).unwrap();
        assert_eq!(bytes, eof.encode_slow());
    }

    #[test]
    fn decode_eof_dangling() {
        //0xEF000101 | u16  | 0x02 | u16 | u16 * cnum | 0x03 | u16 | cnum* u32 | 0xff | u16 | 0x00
        let test_cases = [
            (
                bytes!("ef00010100040200010001ff00000000800000fe"),
                bytes!("010203"),
                false,
            ),
            (
                bytes!("ef00010100040200010001ff00000000800000fe"),
                bytes!(""),
                false,
            ),
            (
                bytes!("ef00010100040200010001ff00000000800000"),
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
        //0xEF000101 | u16  | 0x02 | u16 | u16 * cnum | 0x03 | u16 | cnum* u32 | 0xff | u16 | 0x00
        let bytes = bytes!("ef00010100040200010001ff00000000800000fe");
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
