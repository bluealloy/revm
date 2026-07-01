//! Benchmarks for the BLS12-381 precompiles based on benchmarkoor compute cases.

use ark_bls12_381::{Fq, Fr, G1Affine, G2Affine};
use ark_ec::{AffineRepr, CurveGroup};
use ark_ff::UniformRand;
use ark_std::rand::{rngs::StdRng, SeedableRng};
use criterion::{measurement::WallTime, BenchmarkGroup, Throughput};
use primitives::{hex, Bytes};
use revm_precompile::{
    bls12_381_const::{
        FP_LENGTH, FP_PAD_BY, G1_MSM_INPUT_LENGTH, G2_MSM_INPUT_LENGTH, PADDED_FP_LENGTH,
        PADDED_G1_LENGTH, PADDED_G2_LENGTH, PAIRING_INPUT_LENGTH, SCALAR_LENGTH,
    },
    Precompile,
};
use std::hint::black_box;

const GAS_LIMIT: u64 = u64::MAX;
const RESERVOIR: u64 = 0;
const RNG_SEED: u64 = 42;
const UNCACHABLE_INPUTS: usize = 32;

const SPEC_Q: [u8; SCALAR_LENGTH] =
    hex!("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001");

type PrecompileInput = Vec<u8>;

mod arkworks_general {
    use super::{Fq, FP_LENGTH, FP_PAD_BY, PADDED_FP_LENGTH};
    use ark_serialize::CanonicalSerialize;

    pub(super) fn encode_base_field(fp: &Fq) -> [u8; PADDED_FP_LENGTH] {
        let mut bytes = [0u8; FP_LENGTH];
        fp.serialize_uncompressed(&mut bytes[..])
            .expect("failed to serialize field element");
        bytes.reverse();

        let mut padded_bytes = [0u8; PADDED_FP_LENGTH];
        padded_bytes[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&bytes);
        padded_bytes
    }
}

fn push_padded_fp(input: &mut PrecompileInput, fp: [u8; FP_LENGTH]) {
    input.extend_from_slice(&[0u8; FP_PAD_BY]);
    input.extend_from_slice(&fp);
}

fn fp_from_unpadded(fp: [u8; FP_LENGTH]) -> PrecompileInput {
    let mut input = Vec::with_capacity(PADDED_FP_LENGTH);
    push_padded_fp(&mut input, fp);
    input
}

fn spec_p_minus_1() -> PrecompileInput {
    fp_from_unpadded(hex!(
        "1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaaa"
    ))
}

fn fp2_from_unpadded(c0: [u8; FP_LENGTH], c1: [u8; FP_LENGTH]) -> PrecompileInput {
    let mut input = Vec::with_capacity(2 * PADDED_FP_LENGTH);
    push_padded_fp(&mut input, c0);
    push_padded_fp(&mut input, c1);
    input
}

fn g1_from_unpadded(x: [u8; FP_LENGTH], y: [u8; FP_LENGTH]) -> [u8; PADDED_G1_LENGTH] {
    let mut input = [0u8; PADDED_G1_LENGTH];
    input[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&x);
    input[PADDED_FP_LENGTH + FP_PAD_BY..2 * PADDED_FP_LENGTH].copy_from_slice(&y);
    input
}

fn g2_from_unpadded(
    x_c0: [u8; FP_LENGTH],
    x_c1: [u8; FP_LENGTH],
    y_c0: [u8; FP_LENGTH],
    y_c1: [u8; FP_LENGTH],
) -> [u8; PADDED_G2_LENGTH] {
    let mut input = [0u8; PADDED_G2_LENGTH];
    input[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&x_c0);
    input[PADDED_FP_LENGTH + FP_PAD_BY..2 * PADDED_FP_LENGTH].copy_from_slice(&x_c1);
    input[2 * PADDED_FP_LENGTH + FP_PAD_BY..3 * PADDED_FP_LENGTH].copy_from_slice(&y_c0);
    input[3 * PADDED_FP_LENGTH + FP_PAD_BY..4 * PADDED_FP_LENGTH].copy_from_slice(&y_c1);
    input
}

fn spec_p1() -> [u8; PADDED_G1_LENGTH] {
    g1_from_unpadded(
        hex!("112b98340eee2777cc3c14163dea3ec97977ac3dc5c70da32e6e87578f44912e902ccef9efe28d4a78b8999dfbca9426"),
        hex!("186b28d92356c4dfec4b5201ad099dbdede3781f8998ddf929b4cd7756192185ca7b8f4ef7088f813270ac3d48868a21"),
    )
}

fn spec_g1() -> [u8; PADDED_G1_LENGTH] {
    g1_from_unpadded(
        hex!("17f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb"),
        hex!("08b3f481e3aaa0f1a09e30ed741d8ae4fcf5e095d5d00af600db18cb2c04b3edd03cc744a2888ae40caa232946c5e7e1"),
    )
}

fn spec_p2() -> [u8; PADDED_G2_LENGTH] {
    g2_from_unpadded(
        hex!("103121a2ceaae586d240843a398967325f8eb5a93e8fea99b62b9f88d8556c80dd726a4b30e84a36eeabaf3592937f27"),
        hex!("086b990f3da2aeac0a36143b7d7c824428215140db1bb859338764cb58458f081d92664f9053b50b3fbd2e4723121b68"),
        hex!("0f9e7ba9a86a8f7624aa2b42dcc8772e1af4ae115685e60abc2c9b90242167acef3d0be4050bf935eed7c3b6fc7ba77e"),
        hex!("0d22c3652d0dc6f0fc9316e14268477c2049ef772e852108d269d9c38dba1d4802e8dae479818184c08f9a569d878451"),
    )
}

fn spec_g2() -> [u8; PADDED_G2_LENGTH] {
    g2_from_unpadded(
        hex!("024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8"),
        hex!("13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e"),
        hex!("0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801"),
        hex!("0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be"),
    )
}

/// Encode a BLS12-381 G1 point.
pub fn encode_bls12381_g1_point(input: &G1Affine) -> [u8; PADDED_G1_LENGTH] {
    let mut output = [0u8; PADDED_G1_LENGTH];

    let Some((x, y)) = input.xy() else {
        return output;
    };

    let x_encoded = arkworks_general::encode_base_field(&x);
    let y_encoded = arkworks_general::encode_base_field(&y);

    output[..PADDED_FP_LENGTH].copy_from_slice(&x_encoded);
    output[PADDED_FP_LENGTH..].copy_from_slice(&y_encoded);

    output
}

/// Encode a BLS12-381 G2 point.
pub fn encode_bls12381_g2_point(input: &G2Affine) -> [u8; PADDED_G2_LENGTH] {
    let mut output = [0u8; PADDED_G2_LENGTH];

    let Some((x, y)) = input.xy() else {
        return output;
    };

    let x_c0_encoded = arkworks_general::encode_base_field(&x.c0);
    let x_c1_encoded = arkworks_general::encode_base_field(&x.c1);
    let y_c0_encoded = arkworks_general::encode_base_field(&y.c0);
    let y_c1_encoded = arkworks_general::encode_base_field(&y.c1);

    output[..PADDED_FP_LENGTH].copy_from_slice(&x_c0_encoded);
    output[PADDED_FP_LENGTH..2 * PADDED_FP_LENGTH].copy_from_slice(&x_c1_encoded);
    output[2 * PADDED_FP_LENGTH..3 * PADDED_FP_LENGTH].copy_from_slice(&y_c0_encoded);
    output[3 * PADDED_FP_LENGTH..4 * PADDED_FP_LENGTH].copy_from_slice(&y_c1_encoded);

    output
}

fn splitmix32(seed: u64) -> u32 {
    let mut z = seed.wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    ((z ^ (z >> 31)) as u32).max(1)
}

fn seeded_g1_point(seed: u64) -> [u8; PADDED_G1_LENGTH] {
    let scalar = Fr::from(splitmix32(seed) as u64);
    let point = (G1Affine::generator() * scalar).into_affine();
    encode_bls12381_g1_point(&point)
}

fn seeded_g2_point(seed: u64) -> [u8; PADDED_G2_LENGTH] {
    let scalar = Fr::from(splitmix32(seed) as u64);
    let point = (G2Affine::generator() * scalar).into_affine();
    encode_bls12381_g2_point(&point)
}

fn seeded_fp(seed: u64) -> PrecompileInput {
    let mut rng = StdRng::seed_from_u64(seed);
    let fp = Fq::rand(&mut rng);
    arkworks_general::encode_base_field(&fp).to_vec()
}

fn seeded_fp2(seed: u64) -> PrecompileInput {
    let mut rng = StdRng::seed_from_u64(seed);
    let c0 = Fq::rand(&mut rng);
    let c1 = Fq::rand(&mut rng);

    let mut input = Vec::with_capacity(2 * PADDED_FP_LENGTH);
    input.extend_from_slice(&arkworks_general::encode_base_field(&c0));
    input.extend_from_slice(&arkworks_general::encode_base_field(&c1));
    input
}

fn g1_add_input(lhs: [u8; PADDED_G1_LENGTH]) -> PrecompileInput {
    let mut input = Vec::with_capacity(2 * PADDED_G1_LENGTH);
    input.extend_from_slice(&lhs);
    input.extend_from_slice(&spec_p1());
    input
}

fn g2_add_input(lhs: [u8; PADDED_G2_LENGTH]) -> PrecompileInput {
    let mut input = Vec::with_capacity(2 * PADDED_G2_LENGTH);
    input.extend_from_slice(&lhs);
    input.extend_from_slice(&spec_p2());
    input
}

fn g1_msm_input(point: [u8; PADDED_G1_LENGTH], k: usize) -> PrecompileInput {
    let mut input = Vec::with_capacity(k * G1_MSM_INPUT_LENGTH);
    for _ in 0..k {
        input.extend_from_slice(&point);
        input.extend_from_slice(&SPEC_Q);
    }
    input
}

fn g2_msm_input(point: [u8; PADDED_G2_LENGTH], k: usize) -> PrecompileInput {
    let mut input = Vec::with_capacity(k * G2_MSM_INPUT_LENGTH);
    for _ in 0..k {
        input.extend_from_slice(&point);
        input.extend_from_slice(&SPEC_Q);
    }
    input
}

fn pairing_input(num_pairs: usize) -> PrecompileInput {
    let mut input = Vec::with_capacity(num_pairs * PAIRING_INPUT_LENGTH);
    for _ in 0..num_pairs {
        input.extend_from_slice(&spec_g1());
        input.extend_from_slice(&spec_g2());
    }
    input
}

fn seeded_pairing_input(num_pairs: usize, seed: u64) -> PrecompileInput {
    let mut input = Vec::with_capacity(num_pairs * PAIRING_INPUT_LENGTH);
    for i in 0..num_pairs as u64 {
        input.extend_from_slice(&seeded_g1_point(seed + 2 * i));
        input.extend_from_slice(&seeded_g2_point(seed + 2 * i + 1));
    }
    input
}

fn bench_precompile(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &'static str,
    precompile: &Precompile,
    input: PrecompileInput,
) {
    let input = Bytes::from(input);
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function(name, |b| {
        b.iter(|| {
            let output = precompile
                .execute(black_box(&input), GAS_LIMIT, RESERVOIR)
                .expect("BLS12-381 benchmark input succeeds");
            black_box(output);
        });
    });
}

fn bench_feedback_precompile(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &'static str,
    precompile: &Precompile,
    input: PrecompileInput,
    ret_size: usize,
) {
    group.throughput(Throughput::Bytes(input.len() as u64));
    group.bench_function(name, move |b| {
        let mut input = input.clone();
        b.iter(|| {
            // Benchmarkoor's uncachable BLS loop uses args_offset=0, ret_offset=0, and
            // args_size=CALLDATASIZE, so each call writes its output over the next input prefix.
            let output = precompile
                .execute(black_box(input.as_slice()), GAS_LIMIT, RESERVOIR)
                .expect("BLS12-381 benchmark input succeeds");
            let copy_len = ret_size.min(output.bytes.len()).min(input.len());
            input[..copy_len].copy_from_slice(&output.bytes[..copy_len]);
            black_box(&input);
            black_box(output);
        });
    });
}

fn bench_cycling_precompile(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &'static str,
    precompile: &Precompile,
    inputs: Vec<PrecompileInput>,
) {
    let inputs: Vec<Bytes> = inputs.into_iter().map(Bytes::from).collect();
    group.throughput(Throughput::Bytes(inputs[0].len() as u64));
    group.bench_function(name, move |b| {
        let mut index = 0;
        b.iter(|| {
            let input = &inputs[index % inputs.len()];
            index = index.wrapping_add(1);

            let output = precompile
                .execute(black_box(input), GAS_LIMIT, RESERVOIR)
                .expect("BLS12-381 benchmark input succeeds");
            black_box(output);
        });
    });
}

/// Add benchmarkoor-named benches for the BLS12-381 precompiles.
pub fn add_benches(group: &mut BenchmarkGroup<'_, WallTime>) {
    add_cached_benches(group);
    add_msm_size_benches(group);
    add_pairing_size_benches(group);
    add_uncachable_benches(group);
}

fn add_cached_benches(group: &mut BenchmarkGroup<'_, WallTime>) {
    bench_precompile(
        group,
        "test_bls12_381[bls12_g1add]",
        &revm_precompile::bls12_381::g1_add::PRECOMPILE,
        g1_add_input(spec_g1()),
    );
    bench_precompile(
        group,
        "test_bls12_381[bls12_g1msm]",
        &revm_precompile::bls12_381::g1_msm::PRECOMPILE,
        g1_msm_input(spec_p1(), 128),
    );
    bench_precompile(
        group,
        "test_bls12_381[bls12_g2add]",
        &revm_precompile::bls12_381::g2_add::PRECOMPILE,
        g2_add_input(spec_g2()),
    );
    bench_precompile(
        group,
        "test_bls12_381[bls12_g2msm]",
        &revm_precompile::bls12_381::g2_msm::PRECOMPILE,
        g2_msm_input(spec_p2(), 64),
    );
    bench_precompile(
        group,
        "test_bls12_381[bls12_pairing_check]",
        &revm_precompile::bls12_381::pairing::PRECOMPILE,
        pairing_input(1),
    );
    bench_precompile(
        group,
        "test_bls12_381[bls12_fp_to_g1]",
        &revm_precompile::bls12_381::map_fp_to_g1::PRECOMPILE,
        spec_p_minus_1(),
    );
    bench_precompile(
        group,
        "test_bls12_381[bls12_fp_to_g2]",
        &revm_precompile::bls12_381::map_fp2_to_g2::PRECOMPILE,
        fp2_from_unpadded(
            hex!("1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaaa"),
            hex!("1a0111ea397fe69a4b1ba7b6434bacd764774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaaa"),
        ),
    );
}

fn add_msm_size_benches(group: &mut BenchmarkGroup<'_, WallTime>) {
    for k in [1, 16, 64, 128] {
        bench_precompile(
            group,
            match k {
                1 => "test_bls12_g1_msm[k=1]",
                16 => "test_bls12_g1_msm[k=16]",
                64 => "test_bls12_g1_msm[k=64]",
                128 => "test_bls12_g1_msm[k=128]",
                _ => unreachable!(),
            },
            &revm_precompile::bls12_381::g1_msm::PRECOMPILE,
            g1_msm_input(spec_p1(), k),
        );

        bench_precompile(
            group,
            match k {
                1 => "test_bls12_g2_msm[k=1]",
                16 => "test_bls12_g2_msm[k=16]",
                64 => "test_bls12_g2_msm[k=64]",
                128 => "test_bls12_g2_msm[k=128]",
                _ => unreachable!(),
            },
            &revm_precompile::bls12_381::g2_msm::PRECOMPILE,
            g2_msm_input(spec_p2(), k),
        );
    }
}

fn add_pairing_size_benches(group: &mut BenchmarkGroup<'_, WallTime>) {
    for num_pairs in [1, 3, 6, 12, 24] {
        bench_precompile(
            group,
            match num_pairs {
                1 => "test_bls12_pairing[num_pairs=1]",
                3 => "test_bls12_pairing[num_pairs=3]",
                6 => "test_bls12_pairing[num_pairs=6]",
                12 => "test_bls12_pairing[num_pairs=12]",
                24 => "test_bls12_pairing[num_pairs=24]",
                _ => unreachable!(),
            },
            &revm_precompile::bls12_381::pairing::PRECOMPILE,
            pairing_input(num_pairs),
        );
    }
}

fn add_uncachable_benches(group: &mut BenchmarkGroup<'_, WallTime>) {
    bench_feedback_precompile(
        group,
        "test_bls12_381_uncachable[bls12_g1add]",
        &revm_precompile::bls12_381::g1_add::PRECOMPILE,
        g1_add_input(seeded_g1_point(0)),
        PADDED_G1_LENGTH,
    );
    bench_feedback_precompile(
        group,
        "test_bls12_381_uncachable[bls12_g2add]",
        &revm_precompile::bls12_381::g2_add::PRECOMPILE,
        g2_add_input(seeded_g2_point(0)),
        PADDED_G2_LENGTH,
    );
    bench_feedback_precompile(
        group,
        "test_bls12_381_uncachable[bls12_g1msm]",
        &revm_precompile::bls12_381::g1_msm::PRECOMPILE,
        g1_msm_input(seeded_g1_point(0), 1),
        PADDED_G1_LENGTH,
    );
    bench_feedback_precompile(
        group,
        "test_bls12_381_uncachable[bls12_g2msm]",
        &revm_precompile::bls12_381::g2_msm::PRECOMPILE,
        g2_msm_input(seeded_g2_point(0), 1),
        PADDED_G2_LENGTH,
    );
    bench_feedback_precompile(
        group,
        "test_bls12_381_uncachable[bls12_fp_to_g1]",
        &revm_precompile::bls12_381::map_fp_to_g1::PRECOMPILE,
        seeded_fp(0),
        PADDED_FP_LENGTH,
    );
    bench_feedback_precompile(
        group,
        "test_bls12_381_uncachable[bls12_fp_to_g2]",
        &revm_precompile::bls12_381::map_fp2_to_g2::PRECOMPILE,
        seeded_fp2(0),
        2 * PADDED_FP_LENGTH,
    );

    for num_pairs in [1, 3, 6, 12, 24] {
        bench_cycling_precompile(
            group,
            match num_pairs {
                1 => "test_bls12_pairing_uncachable[num_pairs=1]",
                3 => "test_bls12_pairing_uncachable[num_pairs=3]",
                6 => "test_bls12_pairing_uncachable[num_pairs=6]",
                12 => "test_bls12_pairing_uncachable[num_pairs=12]",
                24 => "test_bls12_pairing_uncachable[num_pairs=24]",
                _ => unreachable!(),
            },
            &revm_precompile::bls12_381::pairing::PRECOMPILE,
            (0..UNCACHABLE_INPUTS)
                .map(|seed| seeded_pairing_input(num_pairs, RNG_SEED + seed as u64))
                .collect(),
        );
    }
}
