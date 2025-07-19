//! Blake2 precompile. More details in [`run`]

use crate::{
    crypto_provider::get_provider, PrecompileError, PrecompileOutput, PrecompileResult,
    PrecompileWithAddress,
};

const F_ROUND: u64 = 1;
const INPUT_LENGTH: usize = 213;

/// Blake2 precompile
pub const FUN: PrecompileWithAddress = PrecompileWithAddress(crate::u64_to_address(9), run);

/// reference: <https://eips.ethereum.org/EIPS/eip-152>
/// input format:
/// [4 bytes for rounds][64 bytes for h][128 bytes for m][8 bytes for t_0][8 bytes for t_1][1 byte for f]
pub fn run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Blake2WrongLength);
    }

    // Rounds 4 bytes
    let rounds = u32::from_be_bytes(input[..4].try_into().unwrap()) as usize;
    let gas_used = rounds as u64 * F_ROUND;
    if gas_used > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let f = match input[212] {
        1 => true,
        0 => false,
        _ => return Err(PrecompileError::Blake2WrongFinalIndicatorFlag),
    };

    let mut h = [0u64; 8];
    //let mut m = [0u64; 16];

    let t;
    // Optimized parsing using ptr::read_unaligned for potentially better performance

    let m;
    unsafe {
        let ptr = input.as_ptr();

        // Read h values
        for (i, item) in h.iter_mut().enumerate() {
            *item = u64::from_le_bytes(core::ptr::read_unaligned(
                ptr.add(4 + i * 8) as *const [u8; 8]
            ));
        }

        m = input[68..68 + 16 * size_of::<u64>()].try_into().unwrap();

        t = [
            u64::from_le_bytes(core::ptr::read_unaligned(ptr.add(196) as *const [u8; 8])),
            u64::from_le_bytes(core::ptr::read_unaligned(ptr.add(204) as *const [u8; 8])),
        ];
    }

    let h = get_provider().blake2_compress(rounds, h, m, t, f);

    let mut out = [0u8; 64];
    for (i, h) in (0..64).step_by(8).zip(h.iter()) {
        out[i..i + 8].copy_from_slice(&h.to_le_bytes());
    }

    Ok(PrecompileOutput::new(gas_used, out.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::hex;
    use std::time::Instant;

    #[test]
    fn perfblake2() {
        let input = [hex!("0000040048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b616162636465666768696a6b6c6d6e6f700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001")
        ,hex!("0000020048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001")
        ,hex!("0000004048c9bdf267e6096a3ba7ca8485ae67bb2bf894fe72f36e3cf1361d5f3af54fa5d182e6ad7f520e511f6c3e2b8c68059b6bbd41fbabd9831f79217e1319cde05b61626300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000001")];

        let time = Instant::now();
        for i in 0..3000 {
            let _ = run(&input[i % 3], u64::MAX).unwrap();
        }
        println!("duration: {:?}", time.elapsed());
    }
}
