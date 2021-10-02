use crate::{models::CallContext, ExitError};

use crate::precompiles::{Precompile, PrecompileOutput, PrecompileResult};
use core::mem;
use core::convert::TryInto;
use crate::collection::{Cow,Borrowed};

use primitive_types::H160 as Address;

/// Blake2 costs.
mod costs {
    /// Cost per round of Blake2 F.
    pub(super) const F_ROUND: u64 = 1;
}

/// Blake2 constants.
mod consts {
    pub(super) const INPUT_LENGTH: usize = 213;
}

pub(super) struct Blake2F;

impl Blake2F {
    pub(super) const ADDRESS: Address = super::make_address(0, 9);
}

impl Precompile for Blake2F {
    fn required_gas(input: &[u8]) -> Result<u64, ExitError> {
        let (int_bytes, _) = input.split_at(mem::size_of::<u32>());
        Ok(u64::from(u32::from_be_bytes(
            int_bytes.try_into().expect("cannot fail"),
        )) * costs::F_ROUND)
    }

    /// The compression function of the blake2 algorithm.
    ///
    /// Takes as an argument the state vector `h`, message block vector `m` (the last block is padded
    /// with zeros to full block size, if required), 2w-bit offset counter `t`, and final block
    /// indicator flag `f`. Local vector v[0..15] is used in processing. F returns a new state vector.
    /// The number of rounds, `r`, is 12 for BLAKE2b and 10 for BLAKE2s. Rounds are numbered from 0 to
    /// r - 1.
    ///
    /// See: https://eips.ethereum.org/EIPS/eip-152
    /// See: https://etherscan.io/address/0000000000000000000000000000000000000009
    fn run(
        input: &[u8],
        target_gas: u64,
        _context: &CallContext,
        _is_static: bool,
    ) -> PrecompileResult {
        if input.len() != consts::INPUT_LENGTH {
            return Err(ExitError::Other(Borrowed("ERR_BLAKE2F_INVALID_LEN")));
        }

        let cost = Self::required_gas(input)?;
        if cost > target_gas {
            return Err(ExitError::OutOfGas);
        }

        let mut rounds_bytes = [0u8; 4];
        rounds_bytes.copy_from_slice(&input[0..4]);
        let rounds = u32::from_be_bytes(rounds_bytes);

        let mut h = [0u64; 8];
        for (mut x, value) in h.iter_mut().enumerate() {
            let mut word: [u8; 8] = [0u8; 8];
            x = x * 8 + 4;
            word.copy_from_slice(&input[x..(x + 8)]);
            *value = u64::from_le_bytes(word);
        }

        let mut m = [0u64; 16];
        for (mut x, value) in m.iter_mut().enumerate() {
            let mut word: [u8; 8] = [0u8; 8];
            x = x * 8 + 68;
            word.copy_from_slice(&input[x..(x + 8)]);
            *value = u64::from_le_bytes(word);
        }

        let mut t: [u64; 2] = [0u64; 2];
        for (mut x, value) in t.iter_mut().enumerate() {
            let mut word: [u8; 8] = [0u8; 8];
            x = x * 8 + 196;
            word.copy_from_slice(&input[x..(x + 8)]);
            *value = u64::from_le_bytes(word);
        }

        if input[212] != 0 && input[212] != 1 {
            return Err(ExitError::Other(Borrowed("ERR_BLAKE2F_FINAL_FLAG")));
        }
        let finished = input[212] != 0;

        let output = blake2::blake2b::Blake2b(rounds, h, m, t, finished).to_vec();
        Ok(PrecompileOutput::without_logs(cost, output))
    }
}
/*
#[cfg(test)]
mod tests {
    use crate::collection::Vec;
    use crate::test_utils::new_context;

    use super::*;

    // [4 bytes for rounds]
    // [64 bytes for h]
    // [128 bytes for m]
    // [8 bytes for t_0]
    // [8 bytes for t_1]
    // [1 byte for f]
    const INPUT: &str = "\
            0000000c\
            48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5\
            d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b\
            6162630000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0300000000000000\
            0000000000000000\
            01";

    fn test_blake2f_out_of_gas() -> PrecompileResult {
        let input = hex::decode(INPUT).unwrap();
        Blake2F::run(&input, 11, &new_context(), false)
    }

    fn test_blake2f_empty() -> PrecompileResult {
        let input = [0u8; 0];
        Blake2F::run(&input, 0, &new_context(), false)
    }

    fn test_blake2f_invalid_len_1() -> PrecompileResult {
        let input = hex::decode(
            "\
            00000c\
            48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5\
            d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b\
            6162630000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0300000000000000\
            0000000000000000\
            01",
        )
        .unwrap();
        Blake2F::run(&input, 12, &new_context(), false)
    }

    fn test_blake2f_invalid_len_2() -> PrecompileResult {
        let input = hex::decode(
            "\
            000000000c\
            48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5\
            d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b\
            6162630000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0300000000000000\
            0000000000000000\
            01",
        )
        .unwrap();
        Blake2F::run(&input, 12, &new_context(), false)
    }

    fn test_blake2f_invalid_flag() -> PrecompileResult {
        let input = hex::decode(
            "\
            0000000c\
            48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5\
            d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b\
            6162630000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0300000000000000\
            0000000000000000\
            02",
        )
        .unwrap();
        Blake2F::run(&input, 12, &new_context(), false)
    }

    fn test_blake2f_r_0() -> Vec<u8> {
        let input = hex::decode(
            "\
            00000000\
            48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5\
            d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b\
            6162630000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0300000000000000\
            0000000000000000\
            01",
        )
        .unwrap();
        Blake2F::run(&input, 12, &new_context(), false)
            .unwrap()
            .output
    }

    fn test_blake2f_r_12() -> Vec<u8> {
        let input = hex::decode(INPUT).unwrap();
        Blake2F::run(&input, 12, &new_context(), false)
            .unwrap()
            .output
    }

    fn test_blake2f_final_block_false() -> Vec<u8> {
        let input = hex::decode(
            "\
            0000000c\
            48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5\
            d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b\
            6162630000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0300000000000000\
            0000000000000000\
            00",
        )
        .unwrap();
        Blake2F::run(&input, 12, &new_context(), false)
            .unwrap()
            .output
    }

    #[test]
    fn test_blake2f() {
        assert!(matches!(
            test_blake2f_out_of_gas(),
            Err(ExitError::OutOfGas)
        ));

        assert!(matches!(
            test_blake2f_empty(),
            Err(ExitError::Other(Borrowed("ERR_BLAKE2F_INVALID_LEN")))
        ));

        assert!(matches!(
            test_blake2f_invalid_len_1(),
            Err(ExitError::Other(Borrowed("ERR_BLAKE2F_INVALID_LEN")))
        ));

        assert!(matches!(
            test_blake2f_invalid_len_2(),
            Err(ExitError::Other(Borrowed("ERR_BLAKE2F_INVALID_LEN")))
        ));

        assert!(matches!(
            test_blake2f_invalid_flag(),
            Err(ExitError::Other(Borrowed("ERR_BLAKE2F_FINAL_FLAG",)))
        ));

        let expected = hex::decode(
            "08c9bcf367e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d\
            282e6ad7f520e511f6c3e2b8c68059b9442be0454267ce079217e1319cde05b",
        )
        .unwrap();
        assert_eq!(test_blake2f_r_0(), expected);

        let expected = hex::decode(
            "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d1\
                7d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923",
        )
        .unwrap();
        assert_eq!(test_blake2f_r_12(), expected);

        let expected = hex::decode(
            "75ab69d3190a562c51aef8d88f1c2775876944407270c42c9844252c26d28752\
            98743e7f6d5ea2f2d3e8d226039cd31b4e426ac4f2d3d666a610c2116fde4735",
        )
        .unwrap();
        assert_eq!(test_blake2f_final_block_false(), expected);
    }
}
*/