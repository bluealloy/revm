use super::{
    decode_helpers::{consume_u16, consume_u8},
    EofDecodeError,
};
use std::vec::Vec;

/// EOF header structure that contains section sizes and metadata
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EofHeader {
    /// Size of EOF types section
    ///
    /// Types section includes num of input and outputs and max stack size.
    pub types_size: u16,
    /// Sizes of EOF code section
    ///
    /// Code size can't be zero.
    pub code_sizes: Vec<u16>,
    /// EOF Container size
    ///
    /// Container size can be zero.
    pub container_sizes: Vec<u32>,
    /// EOF data size
    pub data_size: u16,
    /// Sum of code sizes
    pub sum_code_sizes: usize,
    /// Sum of container sizes
    pub sum_container_sizes: usize,
}

/// EOF header terminal kind, marking end of header.
pub const KIND_TERMINAL: u8 = 0;
/// EOF header code info kind, marking code info section.
pub const KIND_CODE_INFO: u8 = 1;
/// EOF header code kind, marking code section.
pub const KIND_CODE: u8 = 2;
/// EOF header container kind, marking container section.
pub const KIND_CONTAINER: u8 = 3;
/// EOF header data kind, marking data section.
pub const KIND_DATA: u8 = 0xff;
/// EOF header code section size length.
pub const CODE_SECTION_SIZE: usize = 2;
/// EOF header container section size length.
pub const CONTAINER_SECTION_SIZE: usize = 4;

/// Consumes code section from Header.
///
/// It returns rest of the input, list of sizes and sum of all sizes.
#[inline]
fn consume_header_code_section(input: &[u8]) -> Result<(&[u8], Vec<u16>, usize), EofDecodeError> {
    // `num_sections`   2 bytes 0x0001-0xFFFF
    // 16-bit unsigned big-endian integer denoting the number of the sections
    let (input, num_sections) = consume_u16(input)?;
    if num_sections == 0 {
        return Err(EofDecodeError::NonSizes);
    }
    let num_sections = num_sections as usize;
    let byte_size = num_sections * CODE_SECTION_SIZE;
    if input.len() < byte_size {
        return Err(EofDecodeError::ShortInputForSizes);
    }
    let mut sizes = Vec::with_capacity(num_sections);
    let mut sum = 0;
    for i in 0..num_sections {
        // `code_size`  2 bytes 0x0001-0xFFFF
        // 16-bit unsigned big-endian integer denoting the length of the section content
        let code_size = u16::from_be_bytes([
            input[i * CODE_SECTION_SIZE],
            input[i * CODE_SECTION_SIZE + 1],
        ]);
        if code_size == 0 {
            return Err(EofDecodeError::ZeroSize);
        }
        sum += code_size as usize;
        sizes.push(code_size);
    }

    Ok((&input[byte_size..], sizes, sum))
}

/// Consumes container section from Header.
///
/// Similar to [`consume_header_code_section`] but it takes u32 bytes for containers size.
#[inline]
fn consume_header_container_section(
    input: &[u8],
) -> Result<(&[u8], Vec<u32>, usize), EofDecodeError> {
    // `num_sections`   2 bytes 0x0001-0xFFFF
    // 16-bit unsigned big-endian integer denoting the number of the sections
    let (input, num_sections) = consume_u16(input)?;
    if num_sections == 0 {
        return Err(EofDecodeError::NonSizes);
    }
    let num_sections = num_sections as usize;
    let byte_size = num_sections * CONTAINER_SECTION_SIZE;
    if input.len() < byte_size {
        return Err(EofDecodeError::ShortInputForSizes);
    }
    let mut sizes = Vec::with_capacity(num_sections);
    let mut sum = 0;
    for i in 0..num_sections {
        // `section_size` is 4 bytes 0x00000001-0xFFFFFFFF
        // 32-bit unsigned big-endian integer denoting the length of the section content
        let container_size = u32::from_be_bytes(
            input[i * CONTAINER_SECTION_SIZE..(i + 1) * CONTAINER_SECTION_SIZE]
                .try_into()
                .unwrap(),
        );

        if container_size == 0 {
            return Err(EofDecodeError::ZeroSize);
        }
        sum += container_size as usize;
        sizes.push(container_size);
    }

    Ok((&input[byte_size..], sizes, sum))
}

impl EofHeader {
    /// Gets the length of the header in bytes.
    ///
    /// It is minimum 15 bytes (there is at least one code section).
    pub fn size(&self) -> usize {
        2 + // Magic
        1 + // Version
        3 + // Types section
        3 + // Code section
        CODE_SECTION_SIZE * self.code_sizes.len() + // `num_code_sections`
        if self.container_sizes.is_empty() { 0 } else { 3 + CONTAINER_SECTION_SIZE * self.container_sizes.len() } + // Container
        3 + // Data section.
        1 // Terminator
    }

    /// Returns index where data size starts.
    ///
    /// Data size is two bytes long.
    pub fn data_size_raw_i(&self) -> usize {
        // termination(1byte) + code size(2) bytes.
        self.size() - 3
    }

    /// Returns number of types.
    pub fn types_count(&self) -> usize {
        self.types_size as usize / 4
    }

    /// Returns body size.
    ///
    /// It is sum of code sizes, container sizes and data size.
    pub fn body_size(&self) -> usize {
        self.types_size as usize
            + self.sum_code_sizes
            + self.sum_container_sizes
            + self.data_size as usize
    }

    /// Returns raw size of the EOF.
    pub fn eof_size(&self) -> usize {
        self.size() + self.body_size()
    }

    /// Encodes EOF header into binary form.
    pub fn encode(&self, buffer: &mut Vec<u8>) {
        // `magic`	2 bytes	0xEF00	EOF prefix
        buffer.extend_from_slice(&0xEF00u16.to_be_bytes());
        // `version`	1 byte	0x01	EOF version
        buffer.push(0x01);
        // `kind_types`	1 byte	0x01	kind marker for types size section
        buffer.push(KIND_CODE_INFO);
        // `types_size`	2 bytes	0x0004-0xFFFF
        buffer.extend_from_slice(&self.types_size.to_be_bytes());
        // `kind_code`	1 byte	0x02	kind marker for code size section
        buffer.push(KIND_CODE);
        // `code_sections_sizes`
        buffer.extend_from_slice(&(self.code_sizes.len() as u16).to_be_bytes());
        for size in &self.code_sizes {
            buffer.extend_from_slice(&size.to_be_bytes());
        }
        // `kind_container_or_data`	1 byte	0x03 or 0xff kind marker for container size section or data size section
        if !self.container_sizes.is_empty() {
            buffer.push(KIND_CONTAINER);
            // `container_sections_sizes`
            buffer.extend_from_slice(&(self.container_sizes.len() as u16).to_be_bytes());
            for size in &self.container_sizes {
                buffer.extend_from_slice(&size.to_be_bytes());
            }
        }

        // `kind_data`	1 byte	0xff	kind marker for data size section
        buffer.push(KIND_DATA);
        // `data_size`	2 bytes	0x0000-0xFFFF	16-bit unsigned big-endian integer denoting the length of the data section content
        buffer.extend_from_slice(&self.data_size.to_be_bytes());
        // `terminator`	1 byte	0x00	marks the end of the EofHeader
        buffer.push(KIND_TERMINAL);
    }

    /// Decodes EOF header from binary form.
    /// Format of the code section is:
    /// 0xEF000101 | u16  | 0x02 | u16 | u16 * cnum | 0x03 | u16 | cnum* u32 | 0xff | u16 | 0x00
    pub fn decode(input: &[u8]) -> Result<(Self, &[u8]), EofDecodeError> {
        let mut header = EofHeader::default();

        // `magic`	2 bytes	0xEF00	EOF prefix
        let (input, kind) = consume_u16(input)?;
        if kind != 0xEF00 {
            return Err(EofDecodeError::InvalidEOFMagicNumber);
        }

        // `version`	1 byte	0x01	EOF version
        let (input, version) = consume_u8(input)?;
        if version != 0x01 {
            return Err(EofDecodeError::InvalidEOFVersion);
        }

        // `kind_types`	1 byte	0x01	kind marker for types size section
        let (input, kind_code_info) = consume_u8(input)?;
        if kind_code_info != KIND_CODE_INFO {
            return Err(EofDecodeError::InvalidTypesKind);
        }

        // `types_size`	2 bytes	0x0004-0xFFFF
        // 16-bit unsigned big-endian integer denoting the length of the type section content
        let (input, types_size) = consume_u16(input)?;
        header.types_size = types_size;

        // types size
        if header.types_size % CODE_SECTION_SIZE as u16 != 0 {
            return Err(EofDecodeError::InvalidCodeInfo);
        }

        // `kind_code`	1 byte	0x02	kind marker for code size section
        let (input, kind_code) = consume_u8(input)?;
        if kind_code != KIND_CODE {
            return Err(EofDecodeError::InvalidCodeKind);
        }

        // `code_sections_sizes`
        let (input, sizes, sum) = consume_header_code_section(input)?;

        // more than 1024 code sections are not allowed
        if sizes.len() > 0x0400 {
            return Err(EofDecodeError::TooManyCodeSections);
        }

        if sizes.is_empty() {
            return Err(EofDecodeError::ZeroCodeSections);
        }

        if sizes.len() != (types_size / 4) as usize {
            return Err(EofDecodeError::MismatchCodeAndInfoSize);
        }

        header.code_sizes = sizes;
        header.sum_code_sizes = sum;

        let (input, kind_container_or_data) = consume_u8(input)?;

        let input = match kind_container_or_data {
            KIND_CONTAINER => {
                // container_sections_sizes
                let (input, sizes, sum) = consume_header_container_section(input)?;
                // the number of container sections may not exceed 256
                if sizes.len() > 0x0100 {
                    return Err(EofDecodeError::TooManyContainerSections);
                }
                header.container_sizes = sizes;
                header.sum_container_sizes = sum;
                let (input, kind_data) = consume_u8(input)?;
                if kind_data != KIND_DATA {
                    return Err(EofDecodeError::InvalidDataKind);
                }
                input
            }
            KIND_DATA => input,
            invalid_kind => return Err(EofDecodeError::InvalidKindAfterCode { invalid_kind }),
        };

        // `data_size`	2 bytes	0x0000-0xFFFF	16-bit
        // unsigned big-endian integer denoting the length
        // of the data section content (for not yet deployed
        // containers this can be more than the actual content, see Data Section Lifecycle)
        let (input, data_size) = consume_u16(input)?;
        header.data_size = data_size;

        // `terminator`	1 byte	0x00	marks the end of the EofHeader
        let (input, terminator) = consume_u8(input)?;
        if terminator != KIND_TERMINAL {
            return Err(EofDecodeError::InvalidTerminalByte);
        }

        Ok((header, input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::hex;
    use std::vec;

    #[test]
    fn sanity_header_decode() {
        let input = hex!("ef00010100040200010001ff00000000800000fe");
        let (header, _) = EofHeader::decode(&input).unwrap();
        assert_eq!(header.types_size, 4);
        assert_eq!(header.code_sizes, vec![1]);
        assert_eq!(header.container_sizes, Vec::new());
        assert_eq!(header.data_size, 0);
    }

    #[test]
    fn decode_header_not_terminated() {
        let input = hex!("ef0001010004");
        assert_eq!(EofHeader::decode(&input), Err(EofDecodeError::MissingInput));
    }

    #[test]
    fn failing_test() {
        let input = hex!("ef0001010004020001000603000100000014ff000200008000016000e0000000ef000101000402000100010400000000800000fe");
        let _ = EofHeader::decode(&input).unwrap();
    }

    #[test]
    fn cut_header() {
        let input = hex!("ef0001010000028000");
        assert_eq!(
            EofHeader::decode(&input),
            Err(EofDecodeError::ShortInputForSizes)
        );
    }

    #[test]
    fn short_input() {
        let input = hex!("ef0001010000028000");
        assert_eq!(
            EofHeader::decode(&input),
            Err(EofDecodeError::ShortInputForSizes)
        );
    }
}
