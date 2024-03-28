#[inline]
pub(crate) fn consume_u8(input: &[u8]) -> Result<(&[u8], u8), ()> {
    if input.is_empty() {
        return Err(());
    }
    Ok((&input[1..], input[0]))
}

/// Consumes a u16 from the input.
#[inline]
pub(crate) fn consume_u16(input: &[u8]) -> Result<(&[u8], u16), ()> {
    if input.len() < 2 {
        return Err(());
    }
    let (int_bytes, rest) = input.split_at(2);
    Ok((rest, u16::from_be_bytes([int_bytes[0], int_bytes[1]])))
}
