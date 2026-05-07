//! Microbenchmark for the BLS12-381 pairing zero-check (point-at-infinity filter).
//!
//! Compares:
//! - `iter_all`: original `arr.iter().all(|&b| b == 0)` (byte loop)
//! - `u64_safe`: `[u64::from_ne_bytes(arr[k..k+8].try_into().unwrap()); 6]` then
//!   `.iter().all(|&x| x == 0)` (current production)

use criterion::{black_box, measurement::Measurement, BenchmarkGroup};

const FP: usize = 48;

type G1 = ([u8; FP], [u8; FP]);
type G2 = ([u8; FP], [u8; FP], [u8; FP], [u8; FP]);
type Pair = (G1, G2);

#[derive(Copy, Clone)]
enum WorkloadMode {
    AllZero,
    AllNonZero,
    Mixed,
    TrailingNonZero,
}

fn build_pairs(n: usize, mode: WorkloadMode) -> Vec<Pair> {
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        let pair = match mode {
            WorkloadMode::AllZero => (
                ([0u8; FP], [0u8; FP]),
                ([0u8; FP], [0u8; FP], [0u8; FP], [0u8; FP]),
            ),
            WorkloadMode::AllNonZero => {
                let mut a = [0u8; FP];
                a[FP - 1] = (i as u8).wrapping_add(1);
                a[0] = 1;
                let mut b = [0u8; FP];
                b[0] = 2;
                ((a, b), (a, b, a, b))
            }
            WorkloadMode::Mixed => {
                if i % 2 == 0 {
                    (
                        ([0u8; FP], [0u8; FP]),
                        ([0u8; FP], [0u8; FP], [0u8; FP], [0u8; FP]),
                    )
                } else {
                    let mut a = [0u8; FP];
                    a[0] = 7;
                    ((a, a), (a, a, a, a))
                }
            }
            // Worst case for byte-level early-exit: only the last byte is non-zero.
            WorkloadMode::TrailingNonZero => {
                let mut a = [0u8; FP];
                a[FP - 1] = 1;
                ((a, a), (a, a, a, a))
            }
        };
        v.push(pair);
    }
    v
}

#[inline(never)]
fn check_iter_all(pairs: &[Pair]) -> (usize, usize) {
    let mut g1_zero_count = 0usize;
    let mut g2_zero_count = 0usize;
    for ((g1_x, g1_y), (g2_x_0, g2_x_1, g2_y_0, g2_y_1)) in pairs {
        let g1_is_zero = g1_x.iter().all(|&b| b == 0) && g1_y.iter().all(|&b| b == 0);
        let g2_is_zero = g2_x_0.iter().all(|&b| b == 0)
            && g2_x_1.iter().all(|&b| b == 0)
            && g2_y_0.iter().all(|&b| b == 0)
            && g2_y_1.iter().all(|&b| b == 0);
        if g1_is_zero {
            g1_zero_count += 1;
        }
        if g2_is_zero {
            g2_zero_count += 1;
        }
    }
    (g1_zero_count, g2_zero_count)
}

#[inline(always)]
fn fp_is_zero(a: &[u8; FP]) -> bool {
    let w = [
        u64::from_ne_bytes(a[0..8].try_into().unwrap()),
        u64::from_ne_bytes(a[8..16].try_into().unwrap()),
        u64::from_ne_bytes(a[16..24].try_into().unwrap()),
        u64::from_ne_bytes(a[24..32].try_into().unwrap()),
        u64::from_ne_bytes(a[32..40].try_into().unwrap()),
        u64::from_ne_bytes(a[40..48].try_into().unwrap()),
    ];
    w.iter().all(|&x| x == 0)
}

#[inline(never)]
fn check_u64_safe(pairs: &[Pair]) -> (usize, usize) {
    let mut g1_zero_count = 0usize;
    let mut g2_zero_count = 0usize;
    for ((g1_x, g1_y), (g2_x_0, g2_x_1, g2_y_0, g2_y_1)) in pairs {
        let g1_is_zero = fp_is_zero(g1_x) && fp_is_zero(g1_y);
        let g2_is_zero =
            fp_is_zero(g2_x_0) && fp_is_zero(g2_x_1) && fp_is_zero(g2_y_0) && fp_is_zero(g2_y_1);
        if g1_is_zero {
            g1_zero_count += 1;
        }
        if g2_is_zero {
            g2_zero_count += 1;
        }
    }
    (g1_zero_count, g2_zero_count)
}

fn assert_equivalent(pairs: &[Pair]) {
    assert_eq!(check_iter_all(pairs), check_u64_safe(pairs));
}

pub fn add_zero_check_benches<M: Measurement>(group: &mut BenchmarkGroup<'_, M>) {
    let sizes = [1usize, 2, 8, 16];
    let modes = [
        ("all_zero", WorkloadMode::AllZero),
        ("all_nonzero", WorkloadMode::AllNonZero),
        ("mixed", WorkloadMode::Mixed),
        ("trailing_nonzero", WorkloadMode::TrailingNonZero),
    ];

    for (mode_name, mode) in modes {
        for n in sizes {
            let pairs = build_pairs(n, mode);
            assert_equivalent(&pairs);

            group.bench_function(format!("zero_check/iter_all/{mode_name}/n={n}"), |b| {
                b.iter(|| {
                    let r = check_iter_all(black_box(&pairs));
                    black_box(r);
                });
            });

            group.bench_function(format!("zero_check/u64_safe/{mode_name}/n={n}"), |b| {
                b.iter(|| {
                    let r = check_u64_safe(black_box(&pairs));
                    black_box(r);
                });
            });
        }
    }
}
