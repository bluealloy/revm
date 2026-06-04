//! MODEXP precompile benchmarks based on public EEST performance cases.

use criterion::{BenchmarkGroup, BenchmarkId, Throughput};
use revm_precompile::{u64_to_address, Precompiles};
use std::hint::black_box;

const MODEXP_ADDRESS: u64 = 5;
const GAS_LIMIT: u64 = u64::MAX;
const RESERVOIR: u64 = 0;

struct ModexpCase {
    name: &'static str,
    base: Vec<u8>,
    exponent: Vec<u8>,
    modulus: Vec<u8>,
}

struct RawModexpCase {
    name: &'static str,
    input: &'static str,
}

fn repeated(byte: u8, len: usize) -> Vec<u8> {
    vec![byte; len]
}

fn hex_bytes(hex: &str) -> Vec<u8> {
    assert!(hex.len() % 2 == 0, "hex input length must be even");
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

fn case(name: &'static str, base: Vec<u8>, exponent: Vec<u8>, modulus: Vec<u8>) -> ModexpCase {
    ModexpCase {
        name,
        base,
        exponent,
        modulus,
    }
}

fn eest_worst_compute_cases() -> Vec<ModexpCase> {
    let mut cases = vec![
        // Generated from ethereum/execution-spec-tests tests/benchmark/test_worst_compute.py.
        case(
            "mod_even_8b_exp_896",
            repeated(0xff, 8),
            repeated(0xff, 112),
            [repeated(0xff, 7), vec![0x00]].concat(),
        ),
        case(
            "mod_even_16b_exp_320",
            repeated(0xff, 16),
            repeated(0xff, 40),
            [repeated(0xff, 15), vec![0x00]].concat(),
        ),
        case(
            "mod_even_24b_exp_168",
            repeated(0xff, 24),
            repeated(0xff, 21),
            [repeated(0xff, 23), vec![0x00]].concat(),
        ),
        case(
            "mod_even_32b_exp_40",
            repeated(0xff, 32),
            repeated(0xff, 5),
            [repeated(0xff, 31), vec![0x00]].concat(),
        ),
        case(
            "mod_even_32b_exp_96",
            repeated(0xff, 32),
            repeated(0xff, 12),
            [repeated(0xff, 31), vec![0x00]].concat(),
        ),
        case(
            "mod_even_32b_exp_256",
            repeated(0xff, 32),
            repeated(0xff, 32),
            [repeated(0xff, 31), vec![0x00]].concat(),
        ),
        case(
            "mod_even_64b_exp_512",
            repeated(0xff, 64),
            repeated(0xff, 64),
            [repeated(0xff, 63), vec![0x00]].concat(),
        ),
        case(
            "mod_even_128b_exp_1024",
            repeated(0xff, 128),
            repeated(0xff, 128),
            [repeated(0xff, 127), vec![0x00]].concat(),
        ),
        case(
            "mod_even_256b_exp_1024",
            repeated(0xff, 256),
            repeated(0xff, 128),
            [repeated(0xff, 255), vec![0x00]].concat(),
        ),
        case(
            "mod_even_512b_exp_1024",
            repeated(0xff, 512),
            repeated(0xff, 128),
            [repeated(0xff, 511), vec![0x00]].concat(),
        ),
        case(
            "mod_even_1024b_exp_1024",
            repeated(0xff, 1024),
            repeated(0xff, 128),
            [repeated(0xff, 1023), vec![0x00]].concat(),
        ),
        case(
            "mod_odd_32b_exp_96",
            repeated(0xff, 32),
            repeated(0xff, 12),
            [repeated(0xff, 31), vec![0x01]].concat(),
        ),
        case(
            "mod_odd_32b_exp_256",
            repeated(0xff, 32),
            repeated(0xff, 32),
            [repeated(0xff, 31), vec![0x01]].concat(),
        ),
        case(
            "mod_odd_64b_exp_512",
            repeated(0xff, 64),
            repeated(0xff, 64),
            [repeated(0xff, 63), vec![0x01]].concat(),
        ),
        case(
            "mod_odd_128b_exp_1024",
            repeated(0xff, 128),
            repeated(0xff, 128),
            [repeated(0xff, 127), vec![0x01]].concat(),
        ),
        case(
            "mod_odd_256b_exp_1024",
            repeated(0xff, 256),
            repeated(0xff, 128),
            [repeated(0xff, 255), vec![0x01]].concat(),
        ),
        case(
            "mod_odd_512b_exp_1024",
            repeated(0xff, 512),
            repeated(0xff, 128),
            [repeated(0xff, 511), vec![0x01]].concat(),
        ),
        case(
            "mod_odd_1024b_exp_1024",
            repeated(0xff, 1024),
            repeated(0xff, 128),
            [repeated(0xff, 1023), vec![0x01]].concat(),
        ),
        case(
            "mod_odd_32b_exp_cover_windows",
            repeated(0xff, 32),
            b"\x12\x34\x56\x70".repeat(8),
            [repeated(0xff, 31), vec![0x01]].concat(),
        ),
        case(
            "mod_min_gas_base_heavy",
            repeated(0xff, 192),
            vec![0x03],
            (0..6)
                .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                .collect(),
        ),
        case(
            "mod_min_gas_exp_heavy",
            repeated(0xff, 8),
            [vec![0x07], repeated(0xff, 75)].concat(),
            repeated(0xff, 7),
        ),
        case(
            "mod_min_gas_balanced",
            repeated(0xff, 40),
            [vec![0x01], repeated(0xff, 3)].concat(),
            [vec![0x00], repeated(0xff, 38)].concat(),
        ),
        case(
            "mod_exp_208_gas_balanced",
            repeated(0xff, 32),
            repeated(0xff, 5),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_exp_215_gas_exp_heavy",
            repeated(0xff, 8),
            repeated(0xff, 81),
            repeated(0xff, 7),
        ),
        case(
            "mod_exp_298_gas_exp_heavy",
            repeated(0xff, 8),
            repeated(0xff, 112),
            repeated(0xff, 7),
        ),
        case(
            "mod_pawel_2",
            repeated(0xff, 16),
            repeated(0xff, 40),
            repeated(0xff, 15),
        ),
        case(
            "mod_pawel_3",
            repeated(0xff, 24),
            repeated(0xff, 21),
            repeated(0xff, 23),
        ),
        case(
            "mod_pawel_4",
            repeated(0xff, 32),
            repeated(0xff, 12),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_408_gas_base_heavy",
            repeated(0xff, 280),
            vec![0x03],
            [
                (0..8)
                    .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                    .collect::<Vec<_>>(),
                repeated(0xff, 23),
            ]
            .concat(),
        ),
        case(
            "mod_400_gas_exp_heavy",
            repeated(0xff, 16),
            [vec![0x15], repeated(0xff, 37)].concat(),
            repeated(0xff, 15),
        ),
        case(
            "mod_408_gas_balanced",
            repeated(0xff, 48),
            [vec![0x07], repeated(0xff, 4)].concat(),
            [vec![0x00], repeated(0xff, 46)].concat(),
        ),
        case(
            "mod_616_gas_base_heavy",
            repeated(0xff, 344),
            vec![0x03],
            [
                (0..10)
                    .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                    .collect::<Vec<_>>(),
                repeated(0xff, 23),
            ]
            .concat(),
        ),
        case(
            "mod_600_gas_exp_heavy",
            repeated(0xff, 16),
            [vec![0x07], repeated(0xff, 56)].concat(),
            repeated(0xff, 15),
        ),
        case(
            "mod_600_gas_balanced",
            repeated(0xff, 48),
            [vec![0x07], repeated(0xff, 6)].concat(),
            [vec![0x00], repeated(0xff, 46)].concat(),
        ),
        case(
            "mod_800_gas_base_heavy",
            repeated(0xff, 392),
            vec![0x03],
            [
                (0..12)
                    .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                    .collect::<Vec<_>>(),
                repeated(0xff, 7),
            ]
            .concat(),
        ),
        case(
            "mod_800_gas_exp_heavy",
            repeated(0xff, 16),
            [vec![0x01], repeated(0xff, 75)].concat(),
            repeated(0xff, 15),
        ),
        case(
            "mod_767_gas_balanced",
            repeated(0xff, 56),
            repeated(0xff, 6),
            [vec![0x00], repeated(0xff, 54)].concat(),
        ),
        case(
            "mod_852_gas_exp_heavy",
            repeated(0xff, 16),
            repeated(0xff, 80),
            repeated(0xff, 15),
        ),
        case(
            "mod_867_gas_base_heavy",
            repeated(0xff, 408),
            vec![0x03],
            [
                (0..12)
                    .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                    .collect::<Vec<_>>(),
                repeated(0xff, 23),
            ]
            .concat(),
        ),
        case(
            "mod_996_gas_balanced",
            repeated(0xff, 56),
            [vec![0x2b], repeated(0xff, 7)].concat(),
            [vec![0x00], repeated(0xff, 54)].concat(),
        ),
        case(
            "mod_1045_gas_base_heavy",
            repeated(0xff, 448),
            vec![0x03],
            (0..14)
                .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                .collect(),
        ),
        case(
            "mod_677_gas_base_heavy",
            repeated(0xff, 32),
            repeated(0xff, 16),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_765_gas_exp_heavy",
            repeated(0xff, 24),
            repeated(0xff, 32),
            repeated(0xff, 23),
        ),
        case(
            "mod_1360_gas_balanced",
            repeated(0xff, 32),
            repeated(0xff, 32),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_8_exp_648",
            repeated(0xff, 8),
            repeated(0xff, 81),
            repeated(0xff, 7),
        ),
        case(
            "mod_8_exp_896",
            repeated(0xff, 8),
            repeated(0xff, 112),
            repeated(0xff, 7),
        ),
        case(
            "mod_32_exp_32",
            repeated(0xff, 32),
            repeated(0xff, 4),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_32_exp_36",
            repeated(0xff, 32),
            [vec![0x0d], repeated(0xff, 4)].concat(),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_32_exp_40",
            repeated(0xff, 32),
            repeated(0xff, 5),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_32_exp_64",
            repeated(0xff, 32),
            repeated(0xff, 8),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_32_exp_65",
            repeated(0xff, 32),
            [vec![0x01], repeated(0xff, 8)].concat(),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_32_exp_128",
            repeated(0xff, 32),
            repeated(0xff, 16),
            [vec![0x00], repeated(0xff, 31)].concat(),
        ),
        case(
            "mod_256_exp_2",
            repeated(0xff, 256),
            vec![0x03],
            (0..8)
                .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                .collect(),
        ),
        case(
            "mod_264_exp_2",
            repeated(0xff, 264),
            vec![0x03],
            [
                (0..8)
                    .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                    .collect::<Vec<_>>(),
                repeated(0xff, 7),
            ]
            .concat(),
        ),
        case(
            "mod_1024_exp_2",
            repeated(0xff, 1024),
            vec![0x03],
            (0..32)
                .flat_map(|_| [vec![0x00], repeated(0xff, 31)].concat())
                .collect(),
        ),
        case(
            "mod_vul_example_1",
            hex_bytes("03"),
            hex_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e"),
            hex_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"),
        ),
        case(
            "mod_vul_example_2",
            Vec::new(),
            hex_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2e"),
            hex_bytes("fffffffffffffffffffffffffffffffffffffffffffffffffffffffefffffc2f"),
        ),
    ];

    cases.push(case(
        "mod_zero_base",
        Vec::new(),
        repeated(0xff, 1),
        repeated(0xff, 1),
    ));
    cases
}

fn eest_eip7883_named_cases() -> Vec<RawModexpCase> {
    vec![
        RawModexpCase { name: "marius-1-even", input: "000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000c1000000000000000000000000000000000000000000000000000000000000000cffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe000007d7d7d83828282348286877d7d827d407d797d7d7d7d7d7d7d7d7d7d7d5b00000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000021000000000000000000000000000000000000000000000000000000000000000cffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff4000007d7d7d83828282348286877d7d82" },
        RawModexpCase { name: "guido-1-even", input: "000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000d80000000000000000000000000000000000000000000000000000000000000010ffffffffffffffff76ffffffffffffff1cffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c76ec7c7c7c7ffffffffffffffc7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7ffffffffffffc7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c76ec7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7c7ffffffffff3f000000000000000000000000" },
        RawModexpCase { name: "guido-2-even", input: "000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000d80000000000000000000000000000000000000000000000000000000000000010e0060000a921212121212121ff0000212b212121ffff1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f00feffff212121212121ffffffff1fe1e0e0e01e1f1f169f1f1f1f490afcefffffffffffffffff82828282828282828282828282828282828282828200ffff28ff2b212121ffff1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1fffffffffff0afceffffff7ffffffffff7c8282828282a1828282828282828282828282828200ffff28ff2b212121ffff1f1f1f1f1f1fd11f1f1f1f1f1f1f1f1f1f1fffffffffffffffff21212121212121fb2121212121ffff1f1f1f1f1f1f1f1fffaf82828282828200ffff28ff2b21828200" },
        RawModexpCase { name: "guido-3-even", input: "00000000000000000000000000000000000000000000000000000000000001e7000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000002cb0193585a48e18aad777e9c1b54221a0f58140392e4f091cd5f42b2e8644a9384fbd58ae1edec2477ebf7edbf7c0a3f8bd21d1890ee87646feab3c47be716f842cc3da9b940af312dc54450a960e3fc0b86e56abddd154068e10571a96fff6259431632bc15695c6c8679057e66c2c25c127e97e64ee5de6ea1fc0a4a0e431343fed1daafa072c238a45841da86a9806680bc9f298411173210790359209cd454b5af7b4d5688b4403924e5f863d97e2c5349e1a04b54fcf385b1e9d7714bab8fbf5835f6ff9ed575e77dff7af5cbb641db5d537933bae1fa6555d6c12d6fb31ca27b57771f4aebfbe0bf95e8990c0108ffe7cbdaf370be52cf3ade594543af75ad9329d2d11a402270b5b9a6bf4b83307506e118fca4862749d04e916fc7a039f0d13f2a02e0eedb800199ec95df15b4ccd8669b52586879624d51219e72102fad810b5909b1e372ddf33888fb9beb09b416e4164966edbabd89e4a286be36277fc576ed519a15643dac602e92b63d0b9121f0491da5b16ef793a967f096d80b6c81ecaaffad7e3f06a4a5ac2796f1ed9f68e6a0fd5cf191f0c5c2eec338952ff8d31abc68bf760febeb57e088995ba1d7726a2fdd6d8ca28a181378b8b4ab699bfd4b696739bbf17a9eb2df6251143046137fdbbfacac312ebf67a67da9741b596000000000000419a2917c61722b0713d3b00a2f0e1dd5aebbbe09615de424700eea3c3020fe6e9ea5de9fa1ace781df28b21f746d2ab61d0da496e08473c90ff7dfe25b43bcde76f4bafb82e0975bea75f5a0591dba80ba2fff80a07d8853bea5be13ab326ba70c57b153acc646151948d1cf061ca31b02d4719fac710e7c723ca44f5b1737824b7ccc74ba5bff980aabdbf267621cafc3d6dcc29d0ca9c16839a92ed34de136da7900aa3ee43d21aa57498981124357cf0ca9b86f9a8d3f9c604ca00c726e48f7a9945021ea6dfff92d6b2d6514693169ca133e993541bfa4c4c191de806aa80c48109bcfc9901eccfdeb2395ab75fe63c67de900829d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000" },
        RawModexpCase { name: "guido-4-even", input: "00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000181000000000000000000000000000000000000000000000000000000000000000801ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff2cffffffffffffffffffffffffffffffffffffffffffffffffffffffff3b10000000006c01ffffffffffffffffffffffffffffffffffffffffffffffdffffb97ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff3bffffffffffffffffffffffffffffffffffffffffffffffffffffffffebafd93b37ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc5bb6affffffff3fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff2a" },
    ]
}

fn push_len(input: &mut Vec<u8>, len: usize) {
    input.extend_from_slice(&[0; 24]);
    input.extend_from_slice(&(len as u64).to_be_bytes());
}

fn encode_modexp_input(case: &ModexpCase) -> Vec<u8> {
    let mut input =
        Vec::with_capacity(96 + case.base.len() + case.exponent.len() + case.modulus.len());

    push_len(&mut input, case.base.len());
    push_len(&mut input, case.exponent.len());
    push_len(&mut input, case.modulus.len());
    input.extend_from_slice(&case.base);
    input.extend_from_slice(&case.exponent);
    input.extend_from_slice(&case.modulus);
    input
}

pub fn add_benches(group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>) {
    let precompiles = Precompiles::berlin();
    let modexp = precompiles
        .get(&u64_to_address(MODEXP_ADDRESS))
        .expect("MODEXP precompile exists in Berlin");

    for case in eest_worst_compute_cases() {
        let input = encode_modexp_input(&case);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("modexp/eest-worst-compute", case.name),
            &input,
            |b, input| {
                b.iter(|| {
                    let output = modexp
                        .execute(black_box(input), GAS_LIMIT, RESERVOIR)
                        .expect("MODEXP benchmark input succeeds");
                    black_box(output);
                });
            },
        );
    }

    for case in eest_eip7883_named_cases() {
        let input = hex_bytes(case.input);
        group.throughput(Throughput::Bytes(input.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("modexp/eest-eip7883", case.name),
            &input,
            |b, input| {
                b.iter(|| {
                    let output = modexp
                        .execute(black_box(input), GAS_LIMIT, RESERVOIR)
                        .expect("MODEXP benchmark input succeeds");
                    black_box(output);
                });
            },
        );
    }
}
