//! Benchmarks for the BLS12-381 precompiles
use ark_bls12_381::{Fq, Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use arkworks_general::{encode_base_field, encode_field_32_bytes, random_field, random_points};
use criterion::{measurement::Measurement, BenchmarkGroup};
use primitives::Bytes;
use rand::{rngs::StdRng, SeedableRng};
use revm_precompile::bls12_381_const::{PADDED_FP_LENGTH, PADDED_G1_LENGTH, PADDED_G2_LENGTH};

const RNG_SEED: u64 = 42;
const MAX_MSM_SIZE: usize = 256;
const MAX_PAIRING_PAIRS: usize = 16;

type PrecompileInput = Vec<u8>;

mod arkworks_general {
    use ark_bls12_381::Fq;
    use ark_ec::AffineRepr;
    use ark_ff::Field;

    use ark_serialize::CanonicalSerialize;
    use rand::rngs::StdRng;
    use revm_precompile::bls12_381_const::{FP_LENGTH, FP_PAD_BY, PADDED_FP_LENGTH};

    pub(super) fn random_points<P: AffineRepr>(num_points: usize, rng: &mut StdRng) -> Vec<P> {
        let mut points = Vec::new();
        for _ in 0..num_points {
            points.push(P::rand(rng));
        }
        points
    }

    pub(super) fn random_field<F: Field>(num_scalars: usize, rng: &mut StdRng) -> Vec<F> {
        let mut points = Vec::new();
        for _ in 0..num_scalars {
            points.push(F::rand(rng));
        }
        points
    }

    // Note: This is kept separate from encode_base_field since it's for Fr scalars (32 bytes)
    // while encode_base_field is for Fq field elements (padded to 64 bytes)
    pub(super) fn encode_field_32_bytes<F: Field>(field: &F) -> Vec<u8> {
        let mut bytes_be = vec![0u8; 32];
        field
            .serialize_uncompressed(&mut bytes_be[..])
            .expect("Failed to serialize field element");
        bytes_be.reverse();

        bytes_be
    }

    // Add padding to Fq field element and convert it to big endian (BE) format
    pub(super) fn encode_base_field(fp: &Fq) -> Vec<u8> {
        let mut bytes = [0u8; FP_LENGTH];
        fp.serialize_uncompressed(&mut bytes[..])
            .expect("Failed to serialize field element");
        bytes.reverse(); // Convert to big endian

        // Add padding
        let mut padded_bytes = vec![0; PADDED_FP_LENGTH];
        padded_bytes[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&bytes);

        padded_bytes
    }
}

/// Encode a BLS12-381 G1 point
// Note: This has been copied in from precompile/src/bls12_381 since
// those are not public
pub fn encode_bls12381_g1_point(input: &G1Affine) -> [u8; PADDED_G1_LENGTH] {
    let mut output = [0u8; PADDED_G1_LENGTH];

    let Some((x, y)) = input.xy() else {
        return output; // Point at infinity, return all zeros
    };

    let x_encoded = encode_base_field(&x);
    let y_encoded = encode_base_field(&y);

    // Copy the encoded values to the output
    output[..PADDED_FP_LENGTH].copy_from_slice(&x_encoded);
    output[PADDED_FP_LENGTH..].copy_from_slice(&y_encoded);

    output
}

/// Encode a BLS12-381 G2 point
pub fn encode_bls12381_g2_point(input: &G2Affine) -> [u8; PADDED_G2_LENGTH] {
    let mut output = [0u8; PADDED_G2_LENGTH];

    let Some((x, y)) = input.xy() else {
        return output; // Point at infinity, return all zeros
    };

    let x_c0_encoded = encode_base_field(&x.c0);
    let x_c1_encoded = encode_base_field(&x.c1);
    let y_c0_encoded = encode_base_field(&y.c0);
    let y_c1_encoded = encode_base_field(&y.c1);

    // Copy encoded values to output
    output[..PADDED_FP_LENGTH].copy_from_slice(&x_c0_encoded);
    output[PADDED_FP_LENGTH..2 * PADDED_FP_LENGTH].copy_from_slice(&x_c1_encoded);
    output[2 * PADDED_FP_LENGTH..3 * PADDED_FP_LENGTH].copy_from_slice(&y_c0_encoded);
    output[3 * PADDED_FP_LENGTH..4 * PADDED_FP_LENGTH].copy_from_slice(&y_c1_encoded);

    output
}

fn g1_add_test_vectors(num_test_vectors: usize, rng: &mut StdRng) -> Vec<PrecompileInput> {
    let num_g1_points = num_test_vectors * 2;
    let points: Vec<G1Affine> = random_points(num_g1_points, rng);

    points
        .chunks_exact(2)
        .map(|chunk| {
            let lhs = chunk[0];
            let rhs = chunk[1];
            let mut g1_add_input = Vec::new();
            g1_add_input.extend(encode_bls12381_g1_point(&lhs));
            g1_add_input.extend(encode_bls12381_g1_point(&rhs));
            g1_add_input
        })
        .collect()
}

fn g2_add_test_vectors(num_test_vectors: usize, rng: &mut StdRng) -> Vec<PrecompileInput> {
    let num_g2_points = num_test_vectors * 2;
    let points: Vec<G2Affine> = random_points(num_g2_points, rng);

    points
        .chunks_exact(2)
        .map(|chunk| {
            let lhs = chunk[0];
            let rhs = chunk[1];
            let mut g2_add_input = Vec::new();
            g2_add_input.extend(encode_bls12381_g2_point(&lhs));
            g2_add_input.extend(encode_bls12381_g2_point(&rhs));
            g2_add_input
        })
        .collect()
}

/// Add benches for the BLS12-381 G1 add precompile
pub fn add_g1_add_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::g1_add::PRECOMPILE;

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let test_vectors = g1_add_test_vectors(1, &mut rng);
    let input = Bytes::from(test_vectors[0].clone());

    let precompile = *PRECOMPILE.precompile();

    group.bench_function("g1_add", |b| {
        b.iter(|| precompile(&input, u64::MAX).unwrap());
    });
}

/// Add benches for the BLS12-381 G2 add precompile
pub fn add_g2_add_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::g2_add::PRECOMPILE;

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let test_vectors = g2_add_test_vectors(1, &mut rng);
    let input = Bytes::from(test_vectors[0].clone());

    let precompile = *PRECOMPILE.precompile();

    group.bench_function("g2_add", |b| {
        b.iter(|| precompile(&input, u64::MAX).unwrap());
    });
}

/// Add benches for the BLS12-381 G1 msm precompile
pub fn add_g1_msm_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::g1_msm::PRECOMPILE;

    let precompile = *PRECOMPILE.precompile();

    let sizes_to_bench = [MAX_MSM_SIZE, MAX_MSM_SIZE / 2, 2, 1];

    for size in sizes_to_bench {
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let test_vector = g1_msm_test_vectors(size, &mut rng);
        let input = Bytes::from(test_vector);

        group.bench_function(format!("g1_msm (size {})", size), |b| {
            b.iter(|| precompile(&input, u64::MAX).unwrap());
        });
    }
}

fn g1_msm_test_vectors(msm_size: usize, rng: &mut StdRng) -> PrecompileInput {
    let points: Vec<G1Affine> = random_points(msm_size, rng);
    let scalars: Vec<Fr> = random_field(msm_size, rng);

    let mut input = Vec::new();
    for (point, scalar) in points.iter().zip(scalars.iter()) {
        input.extend(encode_bls12381_g1_point(point));
        input.extend(encode_field_32_bytes(scalar));
    }

    input
}

fn g2_msm_test_vectors(msm_size: usize, rng: &mut StdRng) -> PrecompileInput {
    let points: Vec<G2Affine> = random_points(msm_size, rng);
    let scalars: Vec<Fr> = random_field(msm_size, rng);

    let mut input = Vec::new();
    for (point, scalar) in points.iter().zip(scalars.iter()) {
        input.extend(encode_bls12381_g2_point(point));
        input.extend(encode_field_32_bytes(scalar));
    }

    input
}

/// Add benches for the BLS12-381 G2 msm precompile
pub fn add_g2_msm_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::g2_msm::PRECOMPILE;

    let precompile = *PRECOMPILE.precompile();

    let sizes_to_bench = [MAX_MSM_SIZE, MAX_MSM_SIZE / 2, 2, 1];

    for size in sizes_to_bench {
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let test_vector = g2_msm_test_vectors(size, &mut rng);
        let input = Bytes::from(test_vector);

        group.bench_function(format!("g2_msm (size {})", size), |b| {
            b.iter(|| precompile(&input, u64::MAX).unwrap());
        });
    }
}

fn pairing_test_vectors(num_pairs: usize, rng: &mut StdRng) -> PrecompileInput {
    // Generate random G1 and G2 points for pairing
    let g1_points: Vec<G1Affine> = random_points(num_pairs, rng);
    let g2_points: Vec<G2Affine> = random_points(num_pairs, rng);

    let mut input = Vec::new();
    for (g1, g2) in g1_points.iter().zip(g2_points.iter()) {
        input.extend(encode_bls12381_g1_point(g1));
        input.extend(encode_bls12381_g2_point(g2));
    }

    input
}

/// Add benches for the BLS12-381 pairing precompile
pub fn add_pairing_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::pairing::PRECOMPILE;

    let precompile = *PRECOMPILE.precompile();

    let sizes_to_bench = [MAX_PAIRING_PAIRS, MAX_PAIRING_PAIRS / 2, 2, 1];

    for pairs in sizes_to_bench {
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let test_vector = pairing_test_vectors(pairs, &mut rng);
        let input = Bytes::from(test_vector);

        group.bench_function(format!("pairing ({} pairs)", pairs), |b| {
            b.iter(|| precompile(&input, u64::MAX).unwrap());
        });
    }
}

fn map_fp_to_g1_test_vectors(rng: &mut StdRng) -> PrecompileInput {
    let fp: Fq = random_field(1, rng)[0];
    encode_base_field(&fp)
}

/// Add benches for the BLS12-381 map fp to g1 precompiles
pub fn add_map_fp_to_g1_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::map_fp_to_g1::PRECOMPILE;

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let test_vector = map_fp_to_g1_test_vectors(&mut rng);
    let input = Bytes::from(test_vector);

    let precompile = *PRECOMPILE.precompile();

    group.bench_function("map_fp_to_g1", |b| {
        b.iter(|| precompile(&input, u64::MAX).unwrap());
    });
}

fn map_fp2_to_g2_test_vectors(rng: &mut StdRng) -> PrecompileInput {
    let fp_c0: Fq = random_field(1, rng)[0];
    let fp_c1: Fq = random_field(1, rng)[0];

    let mut input = Vec::new();

    input.extend(encode_base_field(&fp_c0));
    input.extend(encode_base_field(&fp_c1));

    input
}

/// Add benches for the BLS12-381 map fp2 to g2 precompiles
pub fn add_map_fp2_to_g2_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::map_fp2_to_g2::PRECOMPILE;

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let test_vector = map_fp2_to_g2_test_vectors(&mut rng);
    let input = Bytes::from(test_vector);

    let precompile = *PRECOMPILE.precompile();

    group.bench_function("map_fp2_to_g2", |b| {
        b.iter(|| precompile(&input, u64::MAX).unwrap());
    });
}
