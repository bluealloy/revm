use ark_bls12_381::{Fq, Fr, G1Affine, G2Affine};
use ark_ec::AffineRepr;
use ark_serialize::CanonicalSerialize;
use arkworks_general::{encode_base_field, encode_field_32_bytes, random_field, random_points};
use criterion::{measurement::Measurement, BenchmarkGroup};
use primitives::Bytes;
use rand::{rngs::StdRng, SeedableRng};
use revm_precompile::bls12_381_const::{
    FP_LENGTH, FP_PAD_BY, PADDED_FP_LENGTH, PADDED_G1_LENGTH, PADDED_G2_LENGTH,
};

const RNG_SEED: u64 = 42;
const MAX_MSM_SIZE: usize = 16;
const MAX_PAIRING_PAIRS: usize = 10;

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

// Note: This has been copied in from precompile/src/bls12_381 since
// those are not public
pub fn encode_bls12381_g1_point(input: &G1Affine) -> [u8; PADDED_G1_LENGTH] {
    let mut output = [0u8; PADDED_G1_LENGTH];

    let Some((x, y)) = input.xy() else {
        return output; // Point at infinity, return all zeros
    };

    let mut x_bytes = [0u8; FP_LENGTH];
    x.serialize_uncompressed(&mut x_bytes[..])
        .expect("Failed to serialize x coordinate");

    let mut y_bytes = [0u8; FP_LENGTH];
    y.serialize_uncompressed(&mut y_bytes[..])
        .expect("Failed to serialize y coordinate");

    // Convert to big endian by reversing the bytes.
    x_bytes.reverse();
    y_bytes.reverse();

    // Add padding and place x in the first half, y in the second half.
    output[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&x_bytes);
    output[PADDED_FP_LENGTH + FP_PAD_BY..].copy_from_slice(&y_bytes);

    output
}
pub fn encode_bls12381_g2_point(input: &G2Affine) -> [u8; PADDED_G2_LENGTH] {
    let mut output = [0u8; PADDED_G2_LENGTH];

    let Some((x, y)) = input.xy() else {
        return output; // Point at infinity, return all zeros
    };

    // Serialize coordinates
    let mut x_c0_bytes = [0u8; FP_LENGTH];
    let mut x_c1_bytes = [0u8; FP_LENGTH];
    let mut y_c0_bytes = [0u8; FP_LENGTH];
    let mut y_c1_bytes = [0u8; FP_LENGTH];

    x.c0.serialize_uncompressed(&mut x_c0_bytes[..])
        .expect("Failed to serialize x.c0 coordinate");
    x.c1.serialize_uncompressed(&mut x_c1_bytes[..])
        .expect("Failed to serialize x.c1 coordinate");
    y.c0.serialize_uncompressed(&mut y_c0_bytes[..])
        .expect("Failed to serialize y.c0 coordinate");
    y.c1.serialize_uncompressed(&mut y_c1_bytes[..])
        .expect("Failed to serialize y.c1 coordinate");

    // Convert to big endian by reversing the bytes
    x_c0_bytes.reverse();
    x_c1_bytes.reverse();
    y_c0_bytes.reverse();
    y_c1_bytes.reverse();

    // Add padding and copy to output
    output[FP_PAD_BY..PADDED_FP_LENGTH].copy_from_slice(&x_c0_bytes);
    output[PADDED_FP_LENGTH + FP_PAD_BY..2 * PADDED_FP_LENGTH].copy_from_slice(&x_c1_bytes);
    output[2 * PADDED_FP_LENGTH + FP_PAD_BY..3 * PADDED_FP_LENGTH].copy_from_slice(&y_c0_bytes);
    output[3 * PADDED_FP_LENGTH + FP_PAD_BY..4 * PADDED_FP_LENGTH].copy_from_slice(&y_c1_bytes);

    output
}

type PrecompileInput = Vec<u8>;

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

pub fn add_g1_add_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::g1_add::PRECOMPILE;

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let test_vectors = g1_add_test_vectors(1, &mut rng);
    let input = Bytes::from(test_vectors[0].clone());

    let precompile = *PRECOMPILE.precompile();

    group.bench_function("g1_add operation", |b| {
        b.iter(|| precompile(&input, u64::MAX).unwrap());
    });
}

pub fn add_g2_add_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::g2_add::PRECOMPILE;

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let test_vectors = g2_add_test_vectors(1, &mut rng);
    let input = Bytes::from(test_vectors[0].clone());

    let precompile = *PRECOMPILE.precompile();

    group.bench_function("g2_add operation", |b| {
        b.iter(|| precompile(&input, u64::MAX).unwrap());
    });
}

pub fn add_g1_msm_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::g1_msm::PRECOMPILE;
    let precompile = *PRECOMPILE.precompile();

    // Benchmark from MAX_MSM_SIZE down to 1
    for size in (1..=MAX_MSM_SIZE).rev() {
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let test_vector = g1_msm_test_vectors(size, &mut rng);
        let input = Bytes::from(test_vector);

        group.bench_function(format!("g1_msm operation (size {})", size), |b| {
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

pub fn add_g2_msm_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::g2_msm::PRECOMPILE;
    let precompile = *PRECOMPILE.precompile();

    // Benchmark from MAX_MSM_SIZE down to 1
    for size in (1..=MAX_MSM_SIZE).rev() {
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let test_vector = g2_msm_test_vectors(size, &mut rng);
        let input = Bytes::from(test_vector);

        group.bench_function(format!("g2_msm operation (size {})", size), |b| {
            b.iter(|| precompile(&input, u64::MAX).unwrap());
        });
    }
}

fn pairing_test_vectors(num_pairs: usize, rng: &mut StdRng) -> PrecompileInput {
    // Generate random G1 and G2 points for pairing
    let g1_points: Vec<G1Affine> = random_points(num_pairs, rng);
    let g2_points: Vec<G2Affine> = random_points(num_pairs, rng);

    // Construct pairing input
    let mut input = Vec::new();
    for (g1, g2) in g1_points.iter().zip(g2_points.iter()) {
        input.extend(encode_bls12381_g1_point(g1));
        input.extend(encode_bls12381_g2_point(g2));
    }

    input
}

pub fn add_pairing_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::pairing::PRECOMPILE;
    let precompile = *PRECOMPILE.precompile();

    // Benchmark from MAX_PAIRING_PAIRS down to 1
    for pairs in (1..=MAX_PAIRING_PAIRS).rev() {
        let mut rng = StdRng::seed_from_u64(RNG_SEED);
        let test_vector = pairing_test_vectors(pairs, &mut rng);
        let input = Bytes::from(test_vector);

        group.bench_function(format!("pairing operation ({} pairs)", pairs), |b| {
            b.iter(|| precompile(&input, u64::MAX).unwrap());
        });
    }
}

fn map_fp_to_g1_test_vectors(rng: &mut StdRng) -> PrecompileInput {
    let fp: Fq = random_field(1, rng)[0];
    encode_base_field(&fp)
}

pub fn add_map_fp_to_g1_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::map_fp_to_g1::PRECOMPILE;

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let test_vector = map_fp_to_g1_test_vectors(&mut rng);
    let input = Bytes::from(test_vector);

    let precompile = *PRECOMPILE.precompile();

    group.bench_function("map_fp_to_g1 operation", |b| {
        b.iter(|| precompile(&input, u64::MAX).unwrap());
    });
}

fn map_fp2_to_g2_test_vectors(rng: &mut StdRng) -> PrecompileInput {
    let fp_c0: Fq = random_field(1, rng)[0];
    let fp_c1: Fq = random_field(1, rng)[0];

    // Construct the input using the encode_base_field function
    let mut input = Vec::new();

    // Encode and add both components
    input.extend(encode_base_field(&fp_c0));
    input.extend(encode_base_field(&fp_c1));

    input
}

pub fn add_map_fp2_to_g2_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    use revm_precompile::bls12_381::map_fp2_to_g2::PRECOMPILE;

    let mut rng = StdRng::seed_from_u64(RNG_SEED);
    let test_vector = map_fp2_to_g2_test_vectors(&mut rng);
    let input = Bytes::from(test_vector);

    let precompile = *PRECOMPILE.precompile();

    group.bench_function("map_fp2_to_g2 operation", |b| {
        b.iter(|| precompile(&input, u64::MAX).unwrap());
    });
}
