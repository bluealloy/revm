use crate::{models::CallContext, ExitError};

use crate::collection::{Borrowed, Cow};
use crate::precompiles::{Precompile, PrecompileOutput, PrecompileResult};
use core::convert::TryInto;
use core::mem;

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
        Ok(u32::from_be_bytes(input[..4].try_into().unwrap()) as u64 * costs::F_ROUND)
    }

    /// reference: https://eips.ethereum.org/EIPS/eip-152
    /// input format:
    /// [4 bytes for rounds][64 bytes for h][128 bytes for m][8 bytes for t_0][8 bytes for t_1][1 byte for f]
    fn run(
        input: &[u8],
        target_gas: u64,
        _context: &CallContext,
        _is_static: bool,
    ) -> PrecompileResult {
        if input.len() != consts::INPUT_LENGTH {
            return Err(ExitError::Other(Borrowed("Invalid last flag for blake2")));
        }

        // rounds 4 bytes
        let rounds = u32::from_be_bytes(input[..4].try_into().unwrap()) as usize;
        let cost = rounds as u64 * costs::F_ROUND;
        if cost > target_gas {
            return Err(ExitError::OutOfGas);
        }
        let mut h = [0u64; 8];
        let mut m = [0u64; 16];
        let mut t = [0u64, 2];

        for (i, pos) in (4..68).step_by(8).enumerate() {
            h[i] = u64::from_le_bytes(input[pos..pos + 8].try_into().unwrap());
        }
        for (i, pos) in (68..196).step_by(8).enumerate() {
            m[i] = u64::from_le_bytes(input[pos..pos + 8].try_into().unwrap());
        }
        t = [
            u64::from_le_bytes(input[196..196 + 8].try_into().unwrap()),
            u64::from_le_bytes(input[204..204 + 8].try_into().unwrap()),
        ];

        let f = match input[212] {
            1 => true,
            0 => false,
            _ => return Err(ExitError::Other(Borrowed("Invalid last flag for blake2"))),
        };

        compress(rounds, &mut h, m, t, f);

        let mut out = [0u8; 64];
        for (i, h) in (0..64).step_by(8).zip(h.iter()) {
            out[i..i + 8].copy_from_slice(&h.to_le_bytes());
        }

        Ok(PrecompileOutput::without_logs(cost, out.to_vec()))
    }
}

/// SIGMA from spec: https://datatracker.ietf.org/doc/html/rfc7693#section-2.7
const SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

/// got IV from: https://en.wikipedia.org/wiki/BLAKE_(hash_function)
const IV: [u64; 8] = [
    0x6a09e667f3bcc908,
    0xbb67ae8584caa73b,
    0x3c6ef372fe94f82b,
    0xa54ff53a5f1d36f1,
    0x510e527fade682d1,
    0x9b05688c2b3e6c1f,
    0x1f83d9abfb41bd6b,
    0x5be0cd19137e2179,
];

#[inline(always)]
/// G function: https://tools.ietf.org/html/rfc7693#section-3.1
fn g(v: &mut [u64], a: usize, b: usize, c: usize, d: usize, x: u64, y: u64) {
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(32);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(24);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(63);
}

// Compression function F takes as an argument the state vector "h",
// message block vector "m" (last block is padded with zeros to full
// block size, if required), 2w-bit offset counter "t", and final block
// indicator flag "f".  Local vector v[0..15] is used in processing.  F
// returns a new state vector.  The number of rounds, "r", is 12 for
// BLAKE2b and 10 for BLAKE2s.  Rounds are numbered from 0 to r - 1.
fn compress(rounds: usize, h: &mut [u64; 8], m: [u64; 16], t: [u64; 2], f: bool) {
    let mut v = [0u64; 16];
    v[..h.len()].copy_from_slice(h); // First half from state.
    v[h.len()..].copy_from_slice(&IV); // Second half from IV.

    v[12] ^= t[0];
    v[13] ^= t[1];

    if f {
        v[14] = !v[14] // Invert all bits if the last-block-flag is set.
    }
    for i in 0..rounds {
        // Message word selection permutation for this round.
        let s = &SIGMA[i % 10];
        g(&mut v, 0, 4, 8, 12, m[s[0]], m[s[1]]);
        g(&mut v, 1, 5, 9, 13, m[s[2]], m[s[3]]);
        g(&mut v, 2, 6, 10, 14, m[s[4]], m[s[5]]);
        g(&mut v, 3, 7, 11, 15, m[s[6]], m[s[7]]);

        g(&mut v, 0, 5, 10, 15, m[s[8]], m[s[9]]);
        g(&mut v, 1, 6, 11, 12, m[s[10]], m[s[11]]);
        g(&mut v, 2, 7, 8, 13, m[s[12]], m[s[13]]);
        g(&mut v, 3, 4, 9, 14, m[s[14]], m[s[15]]);
    }

    for i in 0..8 {
        h[i] ^= v[i] ^ v[i + 8];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collection::Vec;

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
        Blake2F::run(&input, 11, &CallContext::default(), false)
    }

    fn test_blake2f_empty() -> PrecompileResult {
        let input = [0u8; 0];
        Blake2F::run(&input, 0, &CallContext::default(), false)
    }

    // fn test_from_eth_blake23() -> PrecompileResult {
    //     let input = hex::decode(
    //         "\
    //         00000020\
    //         48c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5\
    //         d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b\
    //         6162630000000000000000000000000000000000000000000000000000000000\
    //         0000000000000000000000000000000000000000000000000000000000000000\
    //         0000000000000000000000000000000000000000000000000000000000000000\
    //         0000000000000000000000000000000000000000000000000000000000000000\
    //         0300000000000000000000000000000001",
    //     )
    //     .unwrap();

    //     Blake2F::run(&input, 12, &CallContext::default(), false)
    // }

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
        Blake2F::run(&input, 12, &CallContext::default(), false)
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
        Blake2F::run(&input, 12, &CallContext::default(), false)
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
        Blake2F::run(&input, 12, &CallContext::default(), false)
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
        Blake2F::run(&input, 12, &CallContext::default(), false)
            .unwrap()
            .output
    }

    fn test_blake2f_r_12() -> Vec<u8> {
        let input = hex::decode(INPUT).unwrap();
        Blake2F::run(&input, 12, &CallContext::default(), false)
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
        Blake2F::run(&input, 12, &CallContext::default(), false)
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
            Err(ExitError::Other(Borrowed("Invalid last flag for blake2")))
        ));

        assert!(matches!(
            test_blake2f_invalid_len_1(),
            Err(ExitError::Other(Borrowed("Invalid last flag for blake2")))
        ));

        assert!(matches!(
            test_blake2f_invalid_len_2(),
            Err(ExitError::Other(Borrowed("Invalid last flag for blake2")))
        ));

        assert!(matches!(
            test_blake2f_invalid_flag(),
            Err(ExitError::Other(Borrowed("Invalid last flag for blake2",)))
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
