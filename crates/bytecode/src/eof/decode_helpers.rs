use super::EofDecodeError;

/// Consumes a single byte from the input slice and returns a tuple containing the remaining input slice
/// and the consumed byte as a u8.
///
/// Returns `EofDecodeError::MissingInput` if the input slice is empty.
#[inline]
pub(crate) fn consume_u8(input: &[u8]) -> Result<(&[u8], u8), EofDecodeError> {
    if input.is_empty() {
        return Err(EofDecodeError::MissingInput);
    }
    Ok((&input[1..], input[0]))
}

/// Consumes a u16 from the input.
///
/// Returns `EofDecodeError::MissingInput` if the input slice is less than 2 bytes.
#[inline]
pub(crate) fn consume_u16(input: &[u8]) -> Result<(&[u8], u16), EofDecodeError> {
    if input.len() < 2 {
        return Err(EofDecodeError::MissingInput);
    }
    let (int_bytes, rest) = input.split_at(2);
    Ok((rest, u16::from_be_bytes([int_bytes[0], int_bytes[1]])))
}
