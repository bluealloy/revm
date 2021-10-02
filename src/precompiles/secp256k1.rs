use crate::precompiles::{Precompile, PrecompileOutput, PrecompileResult};
use crate::collection::*;
use primitive_types::{H160 as Address,H256};
use crate::{CallContext, ExitError};


mod costs {
    pub(super) const ECRECOVER_BASE: u64 = 3_000;
}

mod consts {
    pub(super) const INPUT_LEN: usize = 128;
}

/// See: https://ethereum.github.io/yellowpaper/paper.pdf
/// See: https://docs.soliditylang.org/en/develop/units-and-global-variables.html#mathematical-and-cryptographic-functions
/// See: https://etherscan.io/address/0000000000000000000000000000000000000001
// Quite a few library methods rely on this and that should be changed. This
// should only be for precompiles.
pub(crate) fn ecrecover(hash: H256, signature: &[u8]) -> Result<Address, ExitError> {
    assert_eq!(signature.len(), 65);

    #[cfg(feature = "contract")]
    return crate::sdk::ecrecover(hash, signature)
        .map_err(|e| ExitError::Other(Borrowed(e.as_str())));

    #[cfg(not(feature = "contract"))]
    internal_impl(hash, signature)
}

#[cfg(not(feature = "contract"))]
fn internal_impl(hash: H256, signature: &[u8]) -> Result<Address, ExitError> {
    use sha3::{Digest,secp256k1};

    let hash = secp256k1::Message::parse_slice(hash.as_bytes()).unwrap();
    let v = signature[64];
    let signature = secp256k1::Signature::parse_slice(&signature[0..64]).unwrap();
    let bit = match v {
        0..=26 => v,
        _ => v - 27,
    };

    if let Ok(recovery_id) = secp256k1::RecoveryId::parse(bit) {
        if let Ok(public_key) = secp256k1::recover(&hash, &signature, &recovery_id) {
            // recover returns a 65-byte key, but addresses come from the raw 64-byte key
            let r = sha3::Keccak256::digest(&public_key.serialize()[1..]);
            return Ok(Address::from_slice(&r[12..]));
        }
    }

    Err(ExitError::Other(Borrowed(
        "Internal error",
    )))
}

pub(super) struct ECRecover;

impl ECRecover {
    pub(super) const ADDRESS: Address = super::make_address(0, 1);
}

impl Precompile for ECRecover {
    fn required_gas(_input: &[u8]) -> Result<u64, ExitError> {
        Ok(costs::ECRECOVER_BASE)
    }

    fn run(
        input: &[u8],
        target_gas: u64,
        _context: &CallContext,
        _is_static: bool,
    ) -> PrecompileResult {
        let cost = Self::required_gas(input)?;
        if cost > target_gas {
            return Err(ExitError::OutOfGas);
        }

        let mut input = input.to_vec();
        input.resize(consts::INPUT_LEN, 0);

        let mut hash = [0; 32];
        hash.copy_from_slice(&input[0..32]);

        let mut v = [0; 32];
        v.copy_from_slice(&input[32..64]);

        let mut signature = [0; 65]; // signature is (r, s, v), typed (uint256, uint256, uint8)
        signature[0..32].copy_from_slice(&input[64..96]); // r
        signature[32..64].copy_from_slice(&input[96..128]); // s

        let v_bit = match v[31] {
            27 | 28 if v[..31] == [0; 31] => v[31] - 27,
            _ => {
                return Ok(PrecompileOutput::without_logs(0, vec![255u8; 32]));
            }
        };
        signature[64] = v_bit; // v

        let address_res = ecrecover(H256::from_slice(&hash), &signature);
        let output = match address_res {
            Ok(a) => {
                let mut output = [0u8; 32];
                output[12..32].copy_from_slice(a.as_bytes());
                output.to_vec()
            }
            Err(_) => {
                vec![255u8; 32]
            }
        };

        Ok(PrecompileOutput::without_logs(cost, output))
    }
}

/*

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::new_context;

    fn ecverify(hash: H256, signature: &[u8], signer: Address) -> bool {
        matches!(ecrecover(hash, signature), Ok(s) if s == signer)
    }

    #[test]
    fn test_ecverify() {
        let hash = H256::from_slice(
            &hex::decode("1111111111111111111111111111111111111111111111111111111111111111")
                .unwrap(),
        );
        let signature =
            &hex::decode("b9f0bb08640d3c1c00761cdd0121209268f6fd3816bc98b9e6f3cc77bf82b69812ac7a61788a0fdc0e19180f14c945a8e1088a27d92a74dce81c0981fb6447441b")
                .unwrap();
        let signer =
            Address::from_slice(&hex::decode("1563915e194D8CfBA1943570603F7606A3115508").unwrap());
        assert!(ecverify(hash, &signature, signer));
    }

    #[test]
    fn test_ecrecover() {
        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();
        let expected =
            hex::decode("000000000000000000000000c08b5542d177ac6686946920409741463a15dddb")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // out of gas
        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();

        let res = ECRecover::run(&input, 2_999, &new_context(), false);
        assert!(matches!(res, Err(ExitError::OutOfGas)));

        // bad inputs
        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001a650acf9d3f5f0a2c799776a1254355d5f4061762a237396a99a0e0e3fc2bcd6729514a0dacb2e623ac4abd157cb18163ff942280db4d5caad66ddf941ba12e03").unwrap();
        let expected =
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let expected =
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b").unwrap();
        let expected =
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000001b").unwrap();
        let expected =
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap();

        let res = ECRecover::run(&input, 3_000, &new_context(), false)
            .unwrap()
            .output;
        assert_eq!(res, expected);

        // Why is this test returning an address???
        // let input = hex::decode("47173285a8d7341e5e972fc677286384f802f8ef42a5ec5f03bbfa254cb01fad000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000001bffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        // let expected = hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();
        //
        // let res = ecrecover_raw(&input, Some(500)).unwrap().output;
        // assert_eq!(res, expected);
    }
}
*/