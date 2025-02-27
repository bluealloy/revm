use crate::primitives::{Address, Bytes, B256, U256};
use std::vec::Vec;

macro_rules! take_bytes {
    ($data:expr, $len:expr) => {{
        let bytes: Vec<u8> = $data.drain(0..$len).collect();
        bytes
    }};
}

macro_rules! take_int {
    ($data:expr, $type:ty, $len:expr) => {{
        let bytes = take_bytes!($data, $len);
        <$type>::from_be_bytes(bytes.try_into().unwrap())
    }};
}

pub(crate) fn take_address(data: &mut Vec<u8>) -> Address {
    Address::from_slice(&take_bytes!(data, 20))
}

pub(crate) fn take_bytes32(data: &mut Vec<u8>) -> B256 {
    B256::from_slice(&take_bytes!(data, 32))
}

pub(crate) fn take_u256(data: &mut Vec<u8>) -> U256 {
    U256::from_be_slice(&take_bytes!(data, 32))
}

pub(crate) fn take_u64(data: &mut Vec<u8>) -> u64 {
    take_int!(data, u64, 8)
}

pub(crate) fn take_u32(data: &mut Vec<u8>) -> u32 {
    take_int!(data, u32, 4)
}

pub(crate) fn take_u16(data: &mut Vec<u8>) -> u16 {
    take_int!(data, u16, 2)
}

pub(crate) fn take_rest(data: &mut Vec<u8>) -> Bytes {
    let bytes = Bytes::from(data.clone());
    data.clear();
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::address;

    #[test]
    fn test_take_address() {
        let expected = address!("18B06aaF27d44B756FCF16Ca20C1f183EB49111f");
        let mut data = expected.to_vec();
        let address = take_address(&mut data);
        assert_eq!(address, expected);
        assert_eq!(data.len(), 0);
    }

    #[test]
    fn test_take_bytes32() {
        let mut data = vec![0u8; 32];
        let bytes = take_bytes32(&mut data);
        assert_eq!(bytes, B256::default());
    }

    #[test]
    fn test_take_u256() {
        let mut data = vec![0u8; 32];
        let u256 = take_u256(&mut data);
        assert_eq!(u256, U256::default());
    }

    #[test]
    fn test_take_u64() {
        let mut data = vec![0u8; 8];
        let u64 = take_u64(&mut data);
        assert_eq!(u64, 0);
    }

    #[test]
    fn test_take_u32() {
        let mut data = vec![0u8; 4];
        let u32 = take_u32(&mut data);
        assert_eq!(u32, 0);
    }

    #[test]
    fn test_take_u16() {
        let mut data = vec![0u8; 2];
        let u16 = take_u16(&mut data);
        assert_eq!(u16, 0);
    }
}
