//! MODEXP precompile benchmarks based on public EEST performance cases.

use criterion::{BenchmarkGroup, BenchmarkId, Throughput};
use revm_precompile::{u64_to_address, Precompiles};
use std::hint::black_box;

const MODEXP_ADDRESS: u64 = 5;
const GAS_LIMIT: u64 = u64::MAX;
const RESERVOIR: u64 = 0;

enum ModexpCase {
    Encoded {
        name: &'static str,
        base: Vec<u8>,
        exponent: Vec<u8>,
        modulus: Vec<u8>,
    },
    Raw {
        name: &'static str,
        input: &'static str,
    },
}

fn repeated(byte: u8, len: usize) -> Vec<u8> {
    vec![byte; len]
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert!(hex.len().is_multiple_of(2), "hex input length must be even");
    hex.as_bytes()
        .chunks_exact(2)
        .map(|chunk| {
            let high = hex_value(chunk[0]);
            let low = hex_value(chunk[1]);
            (high << 4) | low
        })
        .collect()
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => panic!("invalid hex digit"),
    }
}

const fn encoded(
    name: &'static str,
    base: Vec<u8>,
    exponent: Vec<u8>,
    modulus: Vec<u8>,
) -> ModexpCase {
    ModexpCase::Encoded {
        name,
        base,
        exponent,
        modulus,
    }
}

fn modexp_cases() -> Vec<ModexpCase> {
    vec![
        // EIP-7883 named vector from execution-spec-tests.
        ModexpCase::Raw {
            name: "marius-1-even",
            input: "000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c1000000000000000000000000000000000000000000000000000000000000000cffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe000007d7d7d83828282348286877d7d827d407d797d7d7d7d7d7d7d7d7d7d7d5b00000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000000cffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff4000007d7d7d83828282348286877d7d82",
        },
        // Representative EEST worst-compute shapes without benchmarking the full fixture matrix.
        encoded(
            "mod_even_8b_exp_896",
            repeated(0xff, 8),
            repeated(0xff, 112),
            [repeated(0xff, 7), vec![0x00]].concat(),
        ),
        encoded(
            "mod_even_32b_exp_256",
            repeated(0xff, 32),
            repeated(0xff, 32),
            [repeated(0xff, 31), vec![0x00]].concat(),
        ),
        encoded(
            "mod_odd_256b_exp_1024",
            repeated(0xff, 256),
            repeated(0xff, 128),
            [repeated(0xff, 255), vec![0x01]].concat(),
        ),
        encoded(
            "mod_1024_exp_2",
            repeated(0xff, 1024),
            vec![0x03],
            (0..32)
                .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                .collect(),
        ),
    ]
}

fn push_len(input: &mut Vec<u8>, len: usize) {
    input.extend_from_slice(&[0; 24]);
    input.extend_from_slice(&(len as u64).to_be_bytes());
}

fn encode_modexp_input(base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
    let mut input = Vec::with_capacity(96 + base.len() + exponent.len() + modulus.len());

    push_len(&mut input, base.len());
    push_len(&mut input, exponent.len());
    push_len(&mut input, modulus.len());
    input.extend_from_slice(base);
    input.extend_from_slice(exponent);
    input.extend_from_slice(modulus);
    input
}

pub fn add_benches(group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    let precompiles = Precompiles::berlin();
    let modexp = precompiles
        .get(&u64_to_address(MODEXP_ADDRESS))
        .expect("MODEXP precompile exists in Berlin");

    for case in modexp_cases() {
        let (name, input) = match case {
            ModexpCase::Encoded {
                name,
                base,
                exponent,
                modulus,
            } => (name, encode_modexp_input(&base, &exponent, &modulus)),
            ModexpCase::Raw { name, input } => (name, hex_bytes(input)),
        };

        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(BenchmarkId::new("modexp", name), &input, |b, input| {
            b.iter(|| {
                let output = modexp
                    .execute(black_box(input), GAS_LIMIT, RESERVOIR)
                    .expect("MODEXP benchmark input succeeds");
                black_box(output);
            });
        });
    }
}
