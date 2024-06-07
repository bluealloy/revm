use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revm_precompile::{
    bn128::{
        add::ISTANBUL_ADD_GAS_COST,
        pair::{ISTANBUL_PAIR_BASE, ISTANBUL_PAIR_PER_POINT},
        run_add, run_pair,
    },
    kzg_point_evaluation::run,
    secp256k1::ec_recover_run,
    Bytes,
};
use revm_primitives::{hex, keccak256, Env, U256, VERSIONED_HASH_VERSION_KZG};
use secp256k1::{Message, SecretKey, SECP256K1};
use sha2::{Digest, Sha256};

/// Benchmarks different cryptography-related precompiles.
pub fn benchmark_crypto_precompiles(c: &mut Criterion) {
    let mut group = c.benchmark_group("Crypto Precompile benchmarks");
    let group_name = |description: &str| format!("precompile bench | {description}");

    // === ECPAIRING ===

    // set up ecpairing input
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

    let res = run_pair(
        &input,
        ISTANBUL_PAIR_PER_POINT,
        ISTANBUL_PAIR_BASE,
        u64::MAX,
    )
    .unwrap()
    .0;

    println!("gas used by regular pairing call: {:?}", res);

    // === BN128 ADD ===

    let ecadd_input = hex::decode(
        "\
         18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9\
         063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266\
         07c2b7f58a84bd6145f00c9c2bc0bb1a187f20ff2c92963a88019e7c6a014eed\
         06614e20c147e940f2d70da3f74c9a17df361706a4485c742bd6788478fa17d7",
    )
    .unwrap();

    let res = run_add(&ecadd_input, ISTANBUL_ADD_GAS_COST, 150).unwrap().0;
    println!("gas used by bn128 add precompile: {:?}", res);

    // === ECRECOVER ===

    // generate secp256k1 signature
    let data = hex::decode("1337133713371337").unwrap();
    let hash = keccak256(data);
    let secret_key = SecretKey::new(&mut rand::thread_rng());

    let message = Message::from_digest_slice(&hash[..]).unwrap();
    let s = SECP256K1.sign_ecdsa_recoverable(&message, &secret_key);
    let (rec_id, data) = s.serialize_compact();
    let mut rec_id = rec_id.to_i32() as u8;
    assert_eq!(rec_id, 0);
    rec_id += 27;

    let mut message_and_signature = [0u8; 128];
    message_and_signature[0..32].copy_from_slice(&hash[..]);

    // fit signature into format the precompile expects
    let rec_id = U256::from(rec_id as u64);
    message_and_signature[32..64].copy_from_slice(&rec_id.to_be_bytes::<32>());
    message_and_signature[64..128].copy_from_slice(&data);

    let message_and_signature = Bytes::from(message_and_signature);
    let gas = ec_recover_run(&message_and_signature, u64::MAX).unwrap().0;
    println!("gas used by ecrecover precompile: {:?}", gas);

    // === POINT_EVALUATION ===

    // now check kzg precompile gas
    let commitment = hex!("8f59a8d2a1a625a17f3fea0fe5eb8c896db3764f3185481bc22f91b4aaffcca25f26936857bc3a7c2539ea8ec3a952b7").to_vec();
    let mut versioned_hash = Sha256::digest(&commitment).to_vec();
    versioned_hash[0] = VERSIONED_HASH_VERSION_KZG;
    let z = hex!("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000000").to_vec();
    let y = hex!("1522a4a7f34e1ea350ae07c29c96c7e79655aa926122e95fe69fcbd932ca49e9").to_vec();
    let proof = hex!("a62ad71d14c5719385c0686f1871430475bf3a00f0aa3f7b8dd99a9abc2160744faf0070725e00b60ad9a026a15b1a8c").to_vec();

    let kzg_input = [versioned_hash, z, y, commitment, proof].concat().into();

    let gas = 50000;
    let env = Env::default();
    let (actual_gas, _actual_output) = run(&kzg_input, gas, &env).unwrap();
    println!("gas used by kzg precompile: {:?}", actual_gas);

    group.bench_function(group_name("ecrecover precompile"), |b| {
        b.iter(|| {
            ec_recover_run(&message_and_signature, u64::MAX).unwrap();
            black_box(())
        })
    });

    group.bench_function(group_name("bn128 add precompile"), |b| {
        b.iter(|| {
            run_add(&ecadd_input, ISTANBUL_ADD_GAS_COST, 150).unwrap();
            black_box(())
        })
    });

    group.bench_function(group_name("ecpairing precompile"), |b| {
        b.iter(|| {
            run_pair(
                &input,
                ISTANBUL_PAIR_PER_POINT,
                ISTANBUL_PAIR_BASE,
                u64::MAX,
            )
            .unwrap();
            black_box(())
        })
    });

    group.bench_function(group_name("kzg precompile"), |b| {
        b.iter(|| {
            run(&kzg_input, gas, &env).unwrap();
            black_box(())
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = benchmark_crypto_precompiles
}
criterion_main!(benches);
