/// EOF Header containing
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Header {
    /// Size of EOF types section.
    /// types section includes num of input and outputs and max stack size.
    pub types_size: u16,
    /// Sizes of EOF code section.
    /// Code size can't be zero.
    pub code_sizes: Vec<u16>,
    /// EOF Container size.
    /// Container size can be zero.
    pub container_sizes: Vec<u16>,
    /// EOF data size.
    pub data_size: u16,
}

const KIND_TERMINAL: u8 = 0;
const KIND_TYPES: u8 = 1;
const KIND_CODE: u8 = 2;
const KIND_CONTAINER: u8 = 3;
const KIND_DATA: u8 = 4;

#[inline]
fn consume_u8(input: &[u8]) -> Result<(&[u8], u8), ()> {
    if input.is_empty() {
        return Err(());
    }
    Ok((&input[1..], input[0]))
}

#[inline]
fn consume_u16(input: &[u8]) -> Result<(&[u8], u16), ()> {
    if input.len() < 2 {
        return Err(());
    }
    let (int_bytes, rest) = input.split_at(2);
    Ok((rest, u16::from_be_bytes([int_bytes[0], int_bytes[1]])))
}

#[inline]
fn consume_header_section_size(input: &[u8]) -> Result<(&[u8], Vec<u16>), ()> {
    // num_sections	2 bytes	0x0001-0xFFFF
    // 16-bit unsigned big-endian integer denoting the number of the sections
    let (input, num_sections) = consume_u16(input)?;
    if num_sections == 0 {
        return Err(());
    }
    let byte_size = (num_sections * 2) as usize;
    if input.len() < byte_size {
        return Err(());
    }
    let mut sizes = Vec::with_capacity(num_sections as usize);
    for i in 0..num_sections as usize {
        // size	2 bytes	0x0001-0xFFFF
        // 16-bit unsigned big-endian integer denoting the length of the section content
        let code_size = u16::from_be_bytes([input[i * 2], input[i * 2 + 1]]);
        if code_size == 0 {
            return Err(());
        }
        sizes.push(code_size);
    }

    Ok((&input[byte_size..], sizes))
}

impl Header {
    /// Create new EOF Header.
    pub fn new(
        types_size: u16,
        code_sizes: Vec<u16>,
        container_sizes: Vec<u16>,
        data_size: u16,
    ) -> Self {
        Self {
            types_size,
            code_sizes,
            container_sizes,
            data_size,
        }
    }

    pub fn decode(input: &mut [u8]) -> Result<Self, ()> {
        let mut header = Header::default();

        // magic	2 bytes	0xEF00	EOF prefix
        let (input, kind) = consume_u16(input)?;
        if kind != 0xEF00 {
            return Err(());
        }

        // version	1 byte	0x01	EOF version
        let (input, version) = consume_u8(input)?;
        if version != 0x01 {
            return Err(());
        }

        // kind_types	1 byte	0x01	kind marker for types size section
        let (input, kind_types) = consume_u8(input)?;
        if kind_types != KIND_TYPES {
            return Err(());
        }

        // types_size	2 bytes	0x0004-0xFFFF
        // 16-bit unsigned big-endian integer denoting the length of the type section content
        let (input, types_size) = consume_u16(input)?;
        header.types_size = types_size;

        // kind_code	1 byte	0x02	kind marker for code size section
        let (input, kind_types) = consume_u8(input)?;
        if kind_types != KIND_CODE {
            return Err(());
        }

        // code_sections_sizes
        let (input, code_sizes) = consume_header_section_size(input)?;
        header.code_sizes = code_sizes;

        let (input, kind_container_or_data) = consume_u8(input)?;

        let input = match kind_container_or_data {
            KIND_CONTAINER => {
                // container_sections_sizes
                let (input, container_sizes) = consume_header_section_size(input)?;
                header.container_sizes = container_sizes;
                input
            }
            KIND_DATA => input,
            _ => return Err(()),
        };

        // data_size	2 bytes	0x0000-0xFFFF	16-bit
        // unsigned big-endian integer denoting the length
        // of the data section content (for not yet deployed
        // containers this can be more than the actual content, see Data Section Lifecycle)
        let (input, data_size) = consume_u16(input)?;
        header.data_size = data_size;

        // terminator	1 byte	0x00	marks the end of the header
        let (_, terminator) = consume_u8(input)?;
        if terminator != KIND_TERMINAL {
            return Err(());
        }

        Ok(header)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex;

    #[test]
    fn sanity_header_decode() {
        let mut input = hex!("ef000101000402000100010400000000800000fe");
        let header = Header::decode(&mut input).unwrap();
        assert_eq!(header.types_size, 4);
        assert_eq!(header.code_sizes, vec![1]);
        assert_eq!(header.container_sizes, vec![]);
        assert_eq!(header.data_size, 0);
    }

    #[test]
    fn decode_header_not_terminated() {
        let mut input = hex!("ef0001010004");
        assert_eq!(Header::decode(&mut input), Err(()));
    }
}
