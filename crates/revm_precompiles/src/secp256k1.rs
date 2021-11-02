use crate::{gas_query, Precompile, PrecompileOutput, PrecompileResult, StandardPrecompileFn};
use alloc::vec::Vec;
use core::cmp::min;
use primitive_types::{H160 as Address, H256};
use sha3::{Digest, Keccak256};

const ECRECOVER_BASE: u64 = 3_000;

pub const ECRECOVER: (Address, Precompile) = (
    super::make_address(0, 1),
    Precompile::Standard(ec_recover_run as StandardPrecompileFn),
);

/*
// use k256::{
//     ecdsa::{recoverable, signature::Signer, Error, SigningKey},
//     EncodedPoint as K256PublicKey,
// };
fn secp256k1_ecdsa_recover(sig: &mut [u8; 65], msg: &[u8; 32]) -> Result<Address, Error> {
    sig[64] -= 27;
    let sig = recoverable::Signature::try_from(sig.as_ref()).unwrap();
    let verify_key = sig.recover_verify_key(msg)?;
    let uncompressed_pub_key = K256PublicKey::from(&verify_key).decompress();
    if let Some(public_key) = uncompressed_pub_key {
        let public_key = public_key.as_bytes();
        debug_assert_eq!(public_key[0], 0x04);
        let hash = if public_key[0] == 0x04 {
            //println!("\n\n public_key {:?} \n\n",hex::encode(public_key));
            let hash = Keccak256::digest(public_key[1..].as_ref());
            //println!("\n\n hash {:?} \n\n",hex::encode(hash));
            hash
        } else {
            Keccak256::digest(&public_key[1..])
        };
        //let hash = Keccak256::digest(&public_key[1..]);
        let mut address = Address::zero();
        address.as_bytes_mut().copy_from_slice(&hash[12..]);
        Ok(address)
    } else {
        Err(Error::new())
    }
}*/

// return padded address as H256

use secp256k1::{
    recovery::{RecoverableSignature, RecoveryId},
    Message, Secp256k1,
};

fn secp256k1_ecdsa_recover(sig: &[u8; 65], msg: &[u8; 32]) -> Result<Address, secp256k1::Error> {
    let sig = RecoverableSignature::from_compact(
        &sig[0..64],
        RecoveryId::from_i32((sig[64] - 27) as i32)?,
    )?;

    let secp = Secp256k1::new();
    let public = secp.recover(&Message::from_slice(&msg[..32])?, &sig)?;

    let mut out = vec![0; 20];
    out.copy_from_slice(&Keccak256::digest(&public.serialize_uncompressed()[1..])[12..]);
    Ok(Address::from_slice(&out))
}

fn ec_recover_run(i: &[u8], target_gas: u64) -> PrecompileResult {
    let cost = gas_query(ECRECOVER_BASE, target_gas)?;
    let mut input = [0u8; 128];
    input[..min(i.len(), 128)].copy_from_slice(&i[..min(i.len(), 128)]);

    let mut msg = [0u8; 32];
    let mut sig = [0u8; 65];

    msg[0..32].copy_from_slice(&input[0..32]);
    sig[0..32].copy_from_slice(&input[64..96]);
    sig[32..64].copy_from_slice(&input[96..128]);

    // TODO do this correctly: return if there is junk in V.
    if input[32..63] != [0u8; 31] || !matches!(input[63], 27 | 28) {
        return Ok(PrecompileOutput::without_logs(cost, Vec::new()));
    }

    // TODO hm it will fail for chainId that are more then one byte;
    sig[64] = input[63];

    let out = match secp256k1_ecdsa_recover(&mut sig, &msg) {
        Ok(out) => H256::from(out).as_bytes().to_vec(),
        Err(_) => Vec::new(),
    };

    Ok(PrecompileOutput::without_logs(cost, out))
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
