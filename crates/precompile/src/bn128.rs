use crate::{
    utilities::{bool_to_bytes32, right_pad},
    Address, Error, Precompile, PrecompileResult, PrecompileWithAddress,
};
use bn::{AffineG1, AffineG2, Fq, Fq2, Group, Gt, G1, G2};
use revm_primitives::Bytes;

pub mod add {
    use super::*;

    const ADDRESS: Address = crate::u64_to_address(6);

    pub const ISTANBUL: PrecompileWithAddress = PrecompileWithAddress(
        ADDRESS,
        Precompile::Standard(|input, gas_limit| {
            if 150 > gas_limit {
                return Err(Error::OutOfGas);
            }
            Ok((150, super::run_add(input)?))
        }),
    );

    pub const BYZANTIUM: PrecompileWithAddress = PrecompileWithAddress(
        ADDRESS,
        Precompile::Standard(|input, gas_limit| {
            if 500 > gas_limit {
                return Err(Error::OutOfGas);
            }
            Ok((500, super::run_add(input)?))
        }),
    );
}

pub mod mul {
    use super::*;

    const ADDRESS: Address = crate::u64_to_address(7);

    pub const ISTANBUL: PrecompileWithAddress = PrecompileWithAddress(
        ADDRESS,
        Precompile::Standard(|input, gas_limit| {
            if 6_000 > gas_limit {
                return Err(Error::OutOfGas);
            }
            Ok((6_000, super::run_mul(input)?))
        }),
    );

    pub const BYZANTIUM: PrecompileWithAddress = PrecompileWithAddress(
        ADDRESS,
        Precompile::Standard(|input, gas_limit| {
            if 40_000 > gas_limit {
                return Err(Error::OutOfGas);
            }
            Ok((40_000, super::run_mul(input)?))
        }),
    );
}

pub mod pair {
    use super::*;

    const ADDRESS: Address = crate::u64_to_address(8);

    pub const ISTANBUL_PAIR_PER_POINT: u64 = 34_000;
    pub const ISTANBUL_PAIR_BASE: u64 = 45_000;
    pub const ISTANBUL: PrecompileWithAddress = PrecompileWithAddress(
        ADDRESS,
        Precompile::Standard(|input, gas_limit| {
            super::run_pair(
                input,
                ISTANBUL_PAIR_PER_POINT,
                ISTANBUL_PAIR_BASE,
                gas_limit,
            )
        }),
    );

    pub const BYZANTIUM_PAIR_PER_POINT: u64 = 80_000;
    pub const BYZANTIUM_PAIR_BASE: u64 = 100_000;
    pub const BYZANTIUM: PrecompileWithAddress = PrecompileWithAddress(
        ADDRESS,
        Precompile::Standard(|input, gas_limit| {
            super::run_pair(
                input,
                BYZANTIUM_PAIR_PER_POINT,
                BYZANTIUM_PAIR_BASE,
                gas_limit,
            )
        }),
    );
}

/// Input length for the add operation.
/// `ADD` takes two uncompressed G1 points (64 bytes each).
pub const ADD_INPUT_LEN: usize = 64 + 64;

/// Input length for the multiplication operation.
/// `MUL` takes an uncompressed G1 point (64 bytes) and scalar (32 bytes).
pub const MUL_INPUT_LEN: usize = 64 + 32;

/// Pair element length.
/// `PAIR` elements are composed of an uncompressed G1 point (64 bytes) and an uncompressed G2 point
/// (128 bytes).
pub const PAIR_ELEMENT_LEN: usize = 64 + 128;

/// Reads a single `Fq` from the input slice.
///
/// # Panics
///
/// Panics if the input is not at least 32 bytes long.
#[inline]
pub fn read_fq(input: &[u8]) -> Result<Fq, Error> {
    Fq::from_slice(&input[..32]).map_err(|_| Error::Bn128FieldPointNotAMember)
}

/// Reads the `x` and `y` points from the input slice.
///
/// # Panics
///
/// Panics if the input is not at least 64 bytes long.
#[inline]
pub fn read_point(input: &[u8]) -> Result<G1, Error> {
    let px = read_fq(&input[0..32])?;
    let py = read_fq(&input[32..64])?;
    new_g1_point(px, py)
}

/// Creates a new `G1` point from the given `x` and `y` coordinates.
pub fn new_g1_point(px: Fq, py: Fq) -> Result<G1, Error> {
    if px == Fq::zero() && py == Fq::zero() {
        Ok(G1::zero())
    } else {
        AffineG1::new(px, py)
            .map(Into::into)
            .map_err(|_| Error::Bn128AffineGFailedToCreate)
    }
}

pub fn run_add(input: &[u8]) -> Result<Bytes, Error> {
    let input = right_pad::<ADD_INPUT_LEN>(input);

    let p1 = read_point(&input[..64])?;
    let p2 = read_point(&input[64..])?;

    let mut output = [0u8; 64];
    if let Some(sum) = AffineG1::from_jacobian(p1 + p2) {
        sum.x()
            .into_u256()
            .to_big_endian(&mut output[..32])
            .unwrap();
        sum.y()
            .into_u256()
            .to_big_endian(&mut output[32..])
            .unwrap();
    }

    Ok(output.into())
}

pub fn run_mul(input: &[u8]) -> Result<Bytes, Error> {
    let input = right_pad::<MUL_INPUT_LEN>(input);

    let p = read_point(&input[..64])?;

    // `Fr::from_slice` can only fail when the length is not 32.
    let fr = bn::Fr::from_slice(&input[64..96]).unwrap();

    let mut out = [0u8; 64];
    if let Some(mul) = AffineG1::from_jacobian(p * fr) {
        mul.x().to_big_endian(&mut out[..32]).unwrap();
        mul.y().to_big_endian(&mut out[32..]).unwrap();
    }
    Ok(out.into())
}

pub fn run_pair(
    input: &[u8],
    pair_per_point_cost: u64,
    pair_base_cost: u64,
    gas_limit: u64,
) -> PrecompileResult {
    let gas_used = (input.len() / PAIR_ELEMENT_LEN) as u64 * pair_per_point_cost + pair_base_cost;
    if gas_used > gas_limit {
        return Err(Error::OutOfGas);
    }

    if input.len() % PAIR_ELEMENT_LEN != 0 {
        return Err(Error::Bn128PairLength);
    }

    let success = if input.is_empty() {
        true
    } else {
        let elements = input.len() / PAIR_ELEMENT_LEN;

        let mut mul = Gt::one();
        for idx in 0..elements {
            let read_fq_at = |n: usize| {
                debug_assert!(n < PAIR_ELEMENT_LEN / 32);
                let start = idx * PAIR_ELEMENT_LEN + n * 32;
                // SAFETY: We're reading `6 * 32 == PAIR_ELEMENT_LEN` bytes from `input[idx..]`
                // per iteration. This is guaranteed to be in-bounds.
                let slice = unsafe { input.get_unchecked(start..start + 32) };
                Fq::from_slice(slice).map_err(|_| Error::Bn128FieldPointNotAMember)
            };
            let ax = read_fq_at(0)?;
            let ay = read_fq_at(1)?;
            let bay = read_fq_at(2)?;
            let bax = read_fq_at(3)?;
            let bby = read_fq_at(4)?;
            let bbx = read_fq_at(5)?;

            let a = new_g1_point(ax, ay)?;
            let b = {
                let ba = Fq2::new(bax, bay);
                let bb = Fq2::new(bbx, bby);
                if ba.is_zero() && bb.is_zero() {
                    G2::zero()
                } else {
                    G2::from(AffineG2::new(ba, bb).map_err(|_| Error::Bn128AffineGFailedToCreate)?)
                }
            };

            mul = mul * bn::pairing(a, b);
        }

        mul == Gt::one()
    };
    Ok((gas_used, bool_to_bytes32(success)))
}

/*
#[cfg(test)]
mod tests {
    use crate::test_utils::new_context;

    use super::*;

    #[test]
    fn test_alt_bn128_add() {
        let input = hex::decode(
            "\
             18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9\
             063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266\
             07c2b7f58a84bd6145f00c9c2bc0bb1a187f20ff2c92963a88019e7c6a014eed\
             06614e20c147e940f2d70da3f74c9a17df361706a4485c742bd6788478fa17d7",
        )
        .unwrap();
        let expected = hex::decode(
            "\
            2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee202839703\
            301d1d33be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c915",
        )
        .unwrap();

        let res = Bn128Add::<Byzantium>::run(&input, 500, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // zero sum test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = Bn128Add::<Byzantium>::run(&input, 500, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // out of gas test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let res = Bn128Add::<Byzantium>::run(&input, 499, &new_context(), false);
        assert!(matches!(res, Err(Return::OutOfGas)));

        // no input test
        let input = [0u8; 0];
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = Bn128Add::<Byzantium>::run(&input, 500, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // point not on curve fail
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111",
        )
        .unwrap();

        let res = Bn128Add::<Byzantium>::run(&input, 500, &new_context(), false);
        assert!(matches!(
            res,
            Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_POINT")))
        ));
    }

    #[test]
    fn test_alt_bn128_mul() {
        let input = hex::decode(
            "\
            2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb7\
            21611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb204\
            00000000000000000000000000000000000000000000000011138ce750fa15c2",
        )
        .unwrap();
        let expected = hex::decode(
            "\
            070a8d6a982153cae4be29d434e8faef8a47b274a053f5a4ee2a6c9c13c31e5c\
            031b8ce914eba3a9ffb989f9cdd5b0f01943074bf4f0f315690ec3cec6981afc",
        )
        .unwrap();

        let res = Bn128Mul::<Byzantium>::run(&input, 40_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // out of gas test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0200000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let res = Bn128Mul::<Byzantium>::run(&input, 39_999, &new_context(), false);
        assert!(matches!(res, Err(Return::OutOfGas)));

        // zero multiplication test
        let input = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000\
            0200000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = Bn128Mul::<Byzantium>::run(&input, 40_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // no input test
        let input = [0u8; 0];
        let expected = hex::decode(
            "\
            0000000000000000000000000000000000000000000000000000000000000000\
            0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = Bn128Mul::<Byzantium>::run(&input, 40_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // point not on curve fail
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            0f00000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        let res = Bn128Mul::<Byzantium>::run(&input, 40_000, &new_context(), false);
        assert!(matches!(
            res,
            Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_POINT")))
        ));
    }

    #[test]
    fn test_alt_bn128_pair() {
        let input = hex::decode(
            "\
            1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f59\
            3034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41\
            209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf7\
            04bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a41678\
            2bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d\
            120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550\
            111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c\
            2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411\
            198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
            1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
            090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\
            12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
        )
        .unwrap();
        let expected =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        let res = Bn128Pair::<Byzantium>::run(&input, 260_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // out of gas test
        let input = hex::decode(
            "\
            1c76476f4def4bb94541d57ebba1193381ffa7aa76ada664dd31c16024c43f59\
            3034dd2920f673e204fee2811c678745fc819b55d3e9d294e45c9b03a76aef41\
            209dd15ebff5d46c4bd888e51a93cf99a7329636c63514396b4a452003a35bf7\
            04bf11ca01483bfa8b34b43561848d28905960114c8ac04049af4b6315a41678\
            2bb8324af6cfc93537a2ad1a445cfd0ca2a71acd7ac41fadbf933c2a51be344d\
            120a2a4cf30c1bf9845f20c6fe39e07ea2cce61f0c9bb048165fe5e4de877550\
            111e129f1cf1097710d41c4ac70fcdfa5ba2023c6ff1cbeac322de49d1b6df7c\
            2032c61a830e3c17286de9462bf242fca2883585b93870a73853face6a6bf411\
            198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2\
            1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed\
            090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b\
            12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa",
        )
        .unwrap();
        let res = Bn128Pair::<Byzantium>::run(&input, 259_999, &new_context(), false);
        assert!(matches!(res, Err(Return::OutOfGas)));

        // no input test
        let input = [0u8; 0];
        let expected =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();

        let res = Bn128Pair::<Byzantium>::run(&input, 260_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // point not on curve fail
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111",
        )
        .unwrap();

        let res = Bn128Pair::<Byzantium>::run(&input, 260_000, &new_context(), false);
        assert!(matches!(
            res,
            Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_A")))
        ));

        // invalid input length
        let input = hex::decode(
            "\
            1111111111111111111111111111111111111111111111111111111111111111\
            1111111111111111111111111111111111111111111111111111111111111111\
            111111111111111111111111111111\
        ",
        )
        .unwrap();

        let res = Bn128Pair::<Byzantium>::run(&input, 260_000, &new_context(), false);
        assert!(matches!(
            res,
            Err(Return::Other(Cow::Borrowed("ERR_BN128_INVALID_LEN",)))
        ));
    }
}
*/
