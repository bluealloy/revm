use precompile::{
    bn128, {Precompile, PrecompileError, PrecompileResult, PrecompileWithAddress},
};

pub(crate) mod pair {
    use super::*;

    const GRANITE_MAX_INPUT_SIZE: usize = 112687;
    pub(crate) const GRANITE: PrecompileWithAddress = PrecompileWithAddress(
        bn128::pair::ADDRESS,
        Precompile::Standard(|input, gas_limit| run_pair(input, gas_limit)),
    );

    pub(crate) fn run_pair(input: &[u8], gas_limit: u64) -> PrecompileResult {
        if input.len() > GRANITE_MAX_INPUT_SIZE {
            return Err(PrecompileError::Bn128PairLength.into());
        }
        bn128::run_pair(
            input,
            bn128::pair::ISTANBUL_PAIR_PER_POINT,
            bn128::pair::ISTANBUL_PAIR_BASE,
            gas_limit,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm::{precompile::PrecompileErrors, primitives::hex};
    use std::vec;

    #[test]
    fn test_bn128_pair() {
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
        let outcome = pair::run_pair(&input, 260_000).unwrap();
        assert_eq!(outcome.bytes, expected);

        // invalid input length
        let input = hex::decode(
            "\
          1111111111111111111111111111111111111111111111111111111111111111\
          1111111111111111111111111111111111111111111111111111111111111111\
          111111111111111111111111111111\
      ",
        )
        .unwrap();

        let res = pair::run_pair(&input, 260_000);
        assert!(matches!(
            res,
            Err(PrecompileErrors::Error(PrecompileError::Bn128PairLength))
        ));

        // valid input length shorter than 112687
        let input = vec![1u8; 586 * bn128::PAIR_ELEMENT_LEN];
        let res = pair::run_pair(&input, 260_000);
        assert!(matches!(
            res,
            Err(PrecompileErrors::Error(PrecompileError::OutOfGas))
        ));

        // input length longer than 112687
        let input = vec![1u8; 587 * bn128::PAIR_ELEMENT_LEN];
        let res = pair::run_pair(&input, 260_000);
        assert!(matches!(
            res,
            Err(PrecompileErrors::Error(PrecompileError::Bn128PairLength))
        ));
    }
}
