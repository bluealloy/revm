use primitive_types::U256;

pub fn log2floor(value: U256) -> u64 {
    assert!(!value.is_zero());
    let mut l: u64 = 256;
    for i in 0..4 {
        let i = 3 - i;
        if value.0[i] == 0u64 {
            l -= 64;
        } else {
            l -= value.0[i].leading_zeros() as u64;
            if l == 0 {
                return l;
            } else {
                return l - 1;
            }
        }
    }
    l
}
