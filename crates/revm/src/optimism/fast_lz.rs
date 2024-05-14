const HASH_LOG: u16 = 13;
const MAX_L1_DISTANCE: u16 = 8192;

/// Returns the length of the data after compression through FastLZ, based on
// https://github.com/Vectorized/solady/blob/5315d937d79b335c668896d7533ac603adac5315/js/solady.js
pub(crate) fn flz_compress_len(input: &[u8]) -> u32 {
    let mut idx: u32 = 0;
    let idx_limit: u32 = if input.len() < 13 {
        0
    } else {
        input.len() as u32 - 13
    };

    let mut anchor = 0;

    idx += 2;

    let mut size = 0;

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
            size = literals(idx - anchor, size);
        }

        let len = cmp(input, r + 3, idx + 3, idx_limit + 9);
        size = flz_match(len, size);

        idx = set_next_hash(&mut htab, input, idx + len);
        idx = set_next_hash(&mut htab, input, idx);
        anchor = idx;
    }

    literals(input.len() as u32 - anchor, size)
}

fn literals(r: u32, size: u32) -> u32 {
    let size = size + 0x21 * (r / 0x20);
    let r = r % 0x20;
    if r != 0 {
        size + r + 1
    } else {
        size
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

fn flz_match(l: u32, size: u32) -> u32 {
    let l = l - 1;
    let size = size + (3 * (l / 262));
    if l % 262 >= 6 {
        size + 3
    } else {
        size + 2
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

/// This is a more faithful adaptation of the solady implementation of FastLZ, returning only the compressed size.
/// Unfortunately this is even harder to read due to the use of nearly-minified javascript.
// https://github.com/Vectorized/solady/blob/5315d937d79b335c668896d7533ac603adac5315/js/solady.js
fn solady_flz_compress(ib: &[u8]) -> u32 {
    let b: u32 = (ib.len() - 4) as u32;
    let mut ht = [0; 8192];
    let mut ob = vec![];
    let mut a = 0;
    let mut i = 2;
    let mut r;
    let mut s;
    let mut d;
    let m = 0xffffff;

    let read_u32 = |i: u32| -> u32 {
        u32::from(ib[i as usize])
            | (u32::from(ib[(i + 1) as usize]) << 8)
            | (u32::from(ib[(i + 2) as usize]) << 16)
            | (u32::from(ib[(i + 3) as usize]) << 24)
    };

    fn hash(x: u32) -> u32 {
        (((2654435769 * x as u64) >> 19) & 8191) as u32
    };

    fn literals(mut r: u32, mut s: u32, ib: &[u8], ob: &mut Vec<u8>) {
        while r >= 32 {
            ob.push(31);
            let mut j = 32;
            while j > 0 {
                ob.push(ib[s as usize]);
                s += 1;
                j -= 1;
                r -= 1;
            }
        }
        if r > 0 {
            ob.push((r - 1) as u8);
            while r > 0 {
                ob.push(ib[s as usize]);
                s += 1;
                r -= 1;
            }
        };
    };

    while i < b - 9 {
        loop {
            s = read_u32(i) & m;
            let h = hash(s);
            r = ht[h as usize];
            ht[h as usize] = i;
            d = i - r;
            let c = if d < 8192 { read_u32(r) & m } else { m + 1 };
            if i >= b - 9 {
                break;
            };
            i += 1;
            if s == c {
                break;
            };
        }
        if i >= b - 9 {
            break;
        };
        i -= 1;
        if i > a {
            literals(i - a, a, ib, &mut ob);
        };
        let mut l = 0;
        let p = r + 3;
        let q = i + 3;
        let mut e = b - q;
        while l < e {
            if ib[(p + l) as usize] != ib[(q + l) as usize] {
                e = 0;
            }
            l += 1;
        }
        i += l;
        s = read_u32(i);
        d -= 1;
        while l > 262 {
            ob.push(224 + (d >> 8) as u8);
            ob.push(253);
            ob.push(d as u8);
            l -= 262;
        }
        if l < 7 {
            ob.push(((l << 5) + (d >> 8)) as u8);
            ob.push(d as u8);
        } else {
            ob.push(224 + (d >> 8) as u8);
            ob.push((l - 7) as u8);
            ob.push(d as u8);
        };
        ht[hash(s & m) as usize] = i;
        ht[hash(s >> 8) as usize] = i + 1;
        i += 2;
        a = i;
    }
    literals(b + 4 - a, a, ib, &mut ob);
    ob.len() as u32
}

#[cfg(test)]
mod tests {
    use rand::RngCore;

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

    #[test]
    fn test_flz_compress_len_no_repeats() {
        let mut input = Vec::new();
        let mut len = 0;

        for i in 0..256 {
            input.push(i as u8);
            let prev_len = len;
            len = flz_compress_len(&input);
            assert!(len > prev_len);
        }
    }

    #[test]
    fn test_flz_solady_parity() {
        for _ in 0..1000 {
            let mut input = [0; 4096];
            rand::thread_rng().fill_bytes(&mut input);
            assert_eq!(flz_compress_len(&input), solady_flz_compress(&input));
        }
    }
}
