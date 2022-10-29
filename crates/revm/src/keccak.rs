#![cfg(all(target_arch = "aarch64", target_feature = "sha3"))]
use core::arch::asm;

const RC: [u64; 24] = [
    0x0000000000000001,
    0x0000000000008082,
    0x800000000000808a,
    0x8000000080008000,
    0x000000000000808b,
    0x0000000080000001,
    0x8000000080008081,
    0x8000000000008009,
    0x000000000000008a,
    0x0000000000000088,
    0x0000000080008009,
    0x000000008000000a,
    0x000000008000808b,
    0x800000000000008b,
    0x8000000000008089,
    0x8000000000008003,
    0x8000000000008002,
    0x8000000000000080,
    0x000000000000800a,
    0x800000008000000a,
    0x8000000080008081,
    0x8000000000008080,
    0x0000000080000001,
    0x8000000080008008,
];

/// Keccak-f1600 on ARMv8-A with FEAT_SHA3.
///
/// See p. K12.2.2  p. 11,749 of the ARM Reference manual.
/// See <https://github.com/torvalds/linux/blob/master/arch/arm64/crypto/sha3-ce-core.S>
pub fn keccak_f1600(state: &mut [u64; 25]) {
    unsafe {
        asm!("
            // Read state
            ld1 {{ v0.1d- v3.1d}}, [x0], #32
            ld1 {{ v4.1d- v7.1d}}, [x0], #32
            ld1 {{ v8.1d-v11.1d}}, [x0], #32
            ld1 {{v12.1d-v15.1d}}, [x0], #32
            ld1 {{v16.1d-v19.1d}}, [x0], #32
            ld1 {{v20.1d-v23.1d}}, [x0], #32
            ld1 {{v24.1d}}, [x0]
            sub    x0, x0, #192

            // Loop 24 rounds
            mov x8, #24
        0:  sub x8, x8, #1

            // Theta Calculations
            eor3 v29.16b,  v4.16b,  v9.16b, v14.16b
            eor3 v26.16b,  v1.16b,  v6.16b, v11.16b
            eor3 v28.16b,  v3.16b,  v8.16b, v13.16b
            eor3 v25.16b,  v0.16b,  v5.16b, v10.16b
            eor3 v27.16b,  v2.16b,  v7.16b, v12.16b
            eor3 v29.16b, v29.16b, v19.16b, v24.16b
            eor3 v26.16b, v26.16b, v16.16b, v21.16b
            eor3 v28.16b, v28.16b, v18.16b, v23.16b
            eor3 v25.16b, v25.16b, v15.16b, v20.16b
            eor3 v27.16b, v27.16b, v17.16b, v22.16b

            rax1 v30.2d, v29.2d, v26.2d
            rax1 v26.2d, v26.2d, v28.2d
            rax1 v28.2d, v28.2d, v25.2d
            rax1 v25.2d, v25.2d, v27.2d
            rax1 v27.2d, v27.2d, v29.2d

            // Rho and Phi
            eor  v0.16b,  v0.16b, v30.16b
            xar  v29.2d,   v1.2d,  v25.2d, (64 - 1)
            xar   v1.2d,   v6.2d,  v25.2d, (64 - 44)
            xar   v6.2d,   v9.2d,  v28.2d, (64 - 20)
            xar   v9.2d,  v22.2d,  v26.2d, (64 - 61)
            xar  v22.2d,  v14.2d,  v28.2d, (64 - 39)
            xar  v14.2d,  v20.2d,  v30.2d, (64 - 18)
            xar  v31.2d,   v2.2d,  v26.2d, (64 - 62)
            xar   v2.2d,  v12.2d,  v26.2d, (64 - 43)
            xar  v12.2d,  v13.2d,  v27.2d, (64 - 25)
            xar  v13.2d,  v19.2d,  v28.2d, (64 - 8)
            xar  v19.2d,  v23.2d,  v27.2d, (64 - 56)
            xar  v23.2d,  v15.2d,  v30.2d, (64 - 41)
            xar  v15.2d,   v4.2d,  v28.2d, (64 - 27)
            xar  v28.2d,  v24.2d,  v28.2d, (64 - 14)
            xar  v24.2d,  v21.2d,  v25.2d, (64 - 2)
            xar   v8.2d,   v8.2d,  v27.2d, (64 - 55)
            xar   v4.2d,  v16.2d,  v25.2d, (64 - 45)
            xar  v16.2d,   v5.2d,  v30.2d, (64 - 36)
            xar   v5.2d,   v3.2d,  v27.2d, (64 - 28)
            xar  v27.2d,  v18.2d,  v27.2d, (64 - 21)
            xar   v3.2d,  v17.2d,  v26.2d, (64 - 15)
            xar  v25.2d,  v11.2d,  v25.2d, (64 - 10)
            xar  v26.2d,   v7.2d,  v26.2d, (64 - 6)
            xar  v30.2d,  v10.2d,  v30.2d, (64 - 3)

            // Chi
            bcax v20.16b, v31.16b, v22.16b,  v8.16b
            bcax v21.16b,  v8.16b, v23.16b, v22.16b
            bcax v22.16b, v22.16b, v24.16b, v23.16b
            bcax v23.16b, v23.16b, v31.16b, v24.16b
            bcax v24.16b, v24.16b,  v8.16b, v31.16b

            // Load round constant in now freed v31
            ld1r    {{v31.2d}}, [x1], #8

            bcax v17.16b, v25.16b, v19.16b,  v3.16b
            bcax v18.16b,  v3.16b, v15.16b, v19.16b
            bcax v19.16b, v19.16b, v16.16b, v15.16b
            bcax v15.16b, v15.16b, v25.16b, v16.16b
            bcax v16.16b, v16.16b,  v3.16b, v25.16b

            bcax v10.16b, v29.16b, v12.16b, v26.16b
            bcax v11.16b, v26.16b, v13.16b, v12.16b
            bcax v12.16b, v12.16b, v14.16b, v13.16b
            bcax v13.16b, v13.16b, v29.16b, v14.16b
            bcax v14.16b, v14.16b, v26.16b, v29.16b

            bcax  v7.16b, v30.16b,  v9.16b,  v4.16b
            bcax  v8.16b,  v4.16b,  v5.16b,  v9.16b
            bcax  v9.16b,  v9.16b,  v6.16b,  v5.16b
            bcax  v5.16b,  v5.16b, v30.16b,  v6.16b
            bcax  v6.16b,  v6.16b,  v4.16b, v30.16b

            bcax  v3.16b, v27.16b,  v0.16b, v28.16b
            bcax  v4.16b, v28.16b,  v1.16b,  v0.16b
            bcax  v0.16b,  v0.16b,  v2.16b,  v1.16b
            bcax  v1.16b,  v1.16b, v27.16b,  v2.16b
            bcax  v2.16b,  v2.16b, v28.16b, v27.16b

            // Iota: add round constant
            eor  v0.16b,  v0.16b, v31.16b

            // Rounds loop
            cbnz    w8, 0b

            // Write state
            st1 {{ v0.1d- v3.1d}}, [x0], #32
            st1 {{ v4.1d- v7.1d}}, [x0], #32
            st1 {{ v8.1d-v11.1d}}, [x0], #32
            st1 {{v12.1d-v15.1d}}, [x0], #32
            st1 {{v16.1d-v19.1d}}, [x0], #32
            st1 {{v20.1d-v23.1d}}, [x0], #32
            st1 {{v24.1d}}, [x0]
        ",
            in("x0") state.as_mut_ptr(),
            in("x1") &RC,
            clobber_abi("C"),
            options(nostack)
        );
    }
}

pub fn keccak256(mut bytes: &[u8]) -> [u8; 32] {
    const RATE: usize = 1088 / 8;
    assert_eq!(RATE % 8, 0);
    let mut state = [0u64; 25];

    // Intermediate whole blocks
    while bytes.len() >= RATE {
        dbg!();
        for (b, s) in bytes[..RATE].chunks_exact(8).zip(state.iter_mut()) {
            *s ^= u64::from_le_bytes(b.try_into().unwrap());
        }
        bytes = &bytes[RATE..];
        keccak_f1600(&mut state);
    }
    debug_assert!(bytes.len() < RATE);

    // Final block with padding
    for (b, s) in bytes.chunks_exact(8).zip(state.iter_mut()) {
        *s ^= u64::from_le_bytes(b.try_into().unwrap());
    }
    let mut last_word = [0u8; 8];
    last_word[..(bytes.len() % 8)].copy_from_slice(&bytes[bytes.len() - (bytes.len() % 8)..]);
    state[bytes.len() / 8] ^= u64::from_le_bytes(last_word);
    state[bytes.len() / 8] ^= 1 << (8 * (bytes.len() % 8));
    state[(RATE / 8) - 1] ^= 0x8000000000000000;
    keccak_f1600(&mut state);

    // Output
    let mut output = [0_u8; 32];
    for (o, s) in output.chunks_exact_mut(8).zip(state.iter()) {
        o.copy_from_slice(&s.to_le_bytes());
    }
    output
}

#[test]
fn test_keccak_f1600() {
    // Test vectors are copied from XKCP (eXtended Keccak Code Package)
    // https://github.com/XKCP/XKCP/blob/master/tests/TestVectors/KeccakF-1600-IntermediateValues.txt
    let state_first = [
        0xF1258F7940E1DDE7,
        0x84D5CCF933C0478A,
        0xD598261EA65AA9EE,
        0xBD1547306F80494D,
        0x8B284E056253D057,
        0xFF97A42D7F8E6FD4,
        0x90FEE5A0A44647C4,
        0x8C5BDA0CD6192E76,
        0xAD30A6F71B19059C,
        0x30935AB7D08FFC64,
        0xEB5AA93F2317D635,
        0xA9A6E6260D712103,
        0x81A57C16DBCF555F,
        0x43B831CD0347C826,
        0x01F22F1A11A5569F,
        0x05E5635A21D9AE61,
        0x64BEFEF28CC970F2,
        0x613670957BC46611,
        0xB87C5A554FD00ECB,
        0x8C3EE88A1CCF32C8,
        0x940C7922AE3A2614,
        0x1841F924A2C509E4,
        0x16F53526E70465C2,
        0x75F644E97F30A13B,
        0xEAF1FF7B5CECA249,
    ];
    let state_second = [
        0x2D5C954DF96ECB3C,
        0x6A332CD07057B56D,
        0x093D8D1270D76B6C,
        0x8A20D9B25569D094,
        0x4F9C4F99E5E7F156,
        0xF957B9A2DA65FB38,
        0x85773DAE1275AF0D,
        0xFAF4F247C3D810F7,
        0x1F1B9EE6F79A8759,
        0xE4FECC0FEE98B425,
        0x68CE61B6B9CE68A1,
        0xDEEA66C4BA8F974F,
        0x33C43D836EAFB1F5,
        0xE00654042719DBD9,
        0x7CF8A9F009831265,
        0xFD5449A6BF174743,
        0x97DDAD33D8994B40,
        0x48EAD5FC5D0BE774,
        0xE3B8C8EE55B7B03C,
        0x91A0226E649E42E9,
        0x900E3129E7BADD7B,
        0x202A9EC5FAA3CCE8,
        0x5B3402464E1C3DB6,
        0x609F4E62A44C1059,
        0x20D06CD26A8FBF5C,
    ];

    let mut state = [0u64; 25];
    keccak_f1600(&mut state);
    assert_eq!(state, state_first);
    keccak_f1600(&mut state);
    assert_eq!(state, state_second);
}

#[test]
fn test_keccak256() {
    let input = b"testing";
    // 5f16f4c7f149ac4f9510d9cf8cf384038ad348b3bcdc01915f95de12df9d1b02
    let expected = [
        95, 22, 244, 199, 241, 73, 172, 79, 149, 16, 217, 207, 140, 243, 132, 3, 138, 211, 72, 179,
        188, 220, 1, 145, 95, 149, 222, 18, 223, 157, 27, 2,
    ];
    assert_eq!(keccak256(input), expected);
}
