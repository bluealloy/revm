const HASH_LOG: u16 = 13;
const MAX_L1_DISTANCE: u16 = 8192;

pub(crate) fn flz_compress_len(input: &[u8]) -> u32 {
    let mut idx: u32 = 0;
    let idx_limit: u32 = if input.len() < 13 {
        0
    } else {
        input.len() as u32 - 13
    };

    let mut anchor = 0;

    idx += 2;

    let mut length = 0;

    let mut htab = [0; 8192];

    while idx < idx_limit {
        let mut r: u32;
        let mut distance: u32;

        loop {
            let seq = u24(input, idx);
            let hash = hash(seq);
            r = htab[hash as usize];
            htab[hash as usize] = idx;
            distance = idx - r;
            if idx >= idx_limit {
                break;
            }
            idx += 1;
            if distance <= MAX_L1_DISTANCE as u32 && seq == u24(input, r) {
                break;
            }
        }

        if idx >= idx_limit {
            break;
        }

        idx -= 1;

        if idx > anchor {
            length = literals(idx - anchor, length);
        }

        let len = cmp(input, r + 3, idx + 3, idx_limit + 9);
        length = flz_match(len, length);

        idx = set_next_hash(&mut htab, input, idx + len);
        idx = set_next_hash(&mut htab, input, idx);
        anchor = idx;
    }

    literals(input.len() as u32 - anchor, length)
}

fn literals(r: u32, length: u32) -> u32 {
    let length = length + 0x21 * (r / 0x20);
    let r = r % 0x20;
    if r != 0 {
        length + r + 1
    } else {
        length
    }
}

fn cmp(input: &[u8], p: u32, q: u32, r: u32) -> u32 {
    let mut l = 0;
    let mut r = r - q;
    while l < r {
        if input[(p + l) as usize] != input[(q + l) as usize] {
            r = 0;
        }
        l += 1;
    }
    l
}

fn flz_match(l: u32, length: u32) -> u32 {
    let l = l - 1;
    let length = length + (3 * (l / 262));
    if l % 262 >= 6 {
        length + 3
    } else {
        length + 2
    }
}

fn set_next_hash(htab: &mut [u32; 8192], input: &[u8], idx: u32) -> u32 {
    htab[hash(u24(input, idx)) as usize] = idx;
    idx + 1
}

fn hash(v: u32) -> u16 {
    let hash = (v as u64 * 2654435769) >> (32 - HASH_LOG);
    hash as u16 & 0x1fff
}

fn u24(input: &[u8], idx: u32) -> u32 {
    u32::from(input[idx as usize])
        + (u32::from(input[(idx + 1) as usize]) << 8)
        + (u32::from(input[(idx + 2) as usize]) << 16)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::bytes;

    #[test]
    fn test_flz_compress_len() {
        assert_eq!(flz_compress_len(&bytes!("")), 0);

        assert_eq!(flz_compress_len(&[1; 1000]), 21);
        assert_eq!(flz_compress_len(&[0; 1000]), 21);

        let contract_call_tx_bytes = bytes!("FACADE");
        assert_eq!(flz_compress_len(&contract_call_tx_bytes), 4);

        let contract_call_tx_bytes = bytes!("02f901550a758302df1483be21b88304743f94f80e51afb613d764fa61751affd3313c190a86bb870151bd62fd12adb8e41ef24f3f000000000000000000000000000000000000000000000000000000000000006e000000000000000000000000af88d065e77c8cc2239327c5edb3a432268e5831000000000000000000000000000000000000000000000000000000000003c1e5000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000148c89ed219d02f1a5be012c689b4f5b731827bebe000000000000000000000000c001a033fd89cb37c31b2cba46b6466e040c61fc9b2a3675a7f5f493ebd5ad77c497f8a07cdf65680e238392693019b4092f610222e71b7cec06449cb922b93b6a12744e");
        assert_eq!(flz_compress_len(&contract_call_tx_bytes), 202);
    }
}
