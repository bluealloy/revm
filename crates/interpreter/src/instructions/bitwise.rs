use super::i256::{i256_cmp, i256_sign_compl, two_compl, Sign};
use crate::{
    gas,
    primitives::{Spec, U256},
    Host, Interpreter,
};
use core::cmp::Ordering;
use revm_primitives::uint;

pub fn lt<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 < *op2);
}

pub fn gt<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 > *op2);
}

pub fn slt<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(i256_cmp(&op1, op2) == Ordering::Less);
}

pub fn sgt<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(i256_cmp(&op1, op2) == Ordering::Greater);
}

pub fn eq<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = U256::from(op1 == *op2);
}

pub fn iszero<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = U256::from(*op1 == U256::ZERO);
}

pub fn bitand<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 & *op2;
}

pub fn bitor<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 | *op2;
}

pub fn bitxor<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 = op1 ^ *op2;
}

pub fn not<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1);
    *op1 = !*op1;
}

pub fn byte<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);

    let o1 = as_usize_saturated!(op1);
    *op2 = if o1 < 32 {
        // `31 - o1` because `byte` returns LE, while we want BE
        U256::from(op2.byte(31 - o1))
    } else {
        U256::ZERO
    };
}

/// EIP-145: Bitwise shifting instructions in EVM
pub fn shl<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 <<= as_usize_saturated!(op1);
}

/// EIP-145: Bitwise shifting instructions in EVM
pub fn shr<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);
    *op2 >>= as_usize_saturated!(op1);
}

/// EIP-145: Bitwise shifting instructions in EVM
pub fn sar<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, CONSTANTINOPLE);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, op1, op2);

    let value_sign = i256_sign_compl(op2);

    // If the shift count is 255+, we can short-circuit. This is because shifting by 255 bits is the
    // maximum shift that still leaves 1 bit in the original 256-bit number. Shifting by 256 bits or
    // more would mean that no original bits remain. The result depends on what the highest bit of
    // the value is.
    *op2 = if value_sign == Sign::Zero || op1 >= U256::from(255) {
        match value_sign {
            // value is 0 or >=1, pushing 0
            Sign::Plus | Sign::Zero => U256::ZERO,
            // value is <0, pushing -1
            Sign::Minus => U256::MAX,
        }
    } else {
        const ONE: U256 = uint!{1_U256};
        // SAFETY: shift count is checked above; it's less than 255.
        let shift = usize::try_from(op1).unwrap();
        match value_sign {
            Sign::Plus | Sign::Zero => op2.wrapping_shr(shift),
            Sign::Minus => two_compl(op2.wrapping_sub(ONE).wrapping_shr(shift).wrapping_add(ONE)),
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::instructions::bitwise::{sar, shl, shr};
    use crate::{BytecodeLocked, Contract, DummyHost, Interpreter};
    use core::str::FromStr;
    use revm_primitives::{Bytes, LatestSpec, Env, U256};

    #[test]
    fn test_shift_left() {
        let contract = Contract {
            input: Bytes::default(),
            bytecode: BytecodeLocked::default(),
            ..Default::default()
        };
        let mut host = DummyHost::new(Env::default());
        let mut interpreter = Interpreter::new(contract.clone(), u64::MAX, false);

        struct TestCase {
            value: &'static str,
            shift: &'static str,
            expected: &'static str,
        }

        let test_cases = [
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0x00
            SHL
            ---
            0x0000000000000000000000000000000000000000000000000000000000000001
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0x00",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000001",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0x01
            SHL
            ---
            0x0000000000000000000000000000000000000000000000000000000000000002
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0x01",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000002",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0xff
            SHL
            ---
            0x8000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0xff",
                expected: "0x8000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0x0100
            SHL
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0x0100",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0x0101
            SHL
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0x0101",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x00
            SHL
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x00",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x01
            SHL
            ---
            0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x01",
                expected: "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0xff
            SHL
            ---
            0x8000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0xff",
                expected: "0x8000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x0100
            SHL
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x0100",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x01
            SHL
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x01",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x01
            SHL
            ---
            0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe
            */
            TestCase {
                value: "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x01",
                expected: "0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe",
            },
        ];

        for test in test_cases {
            host.clear();
            push!(interpreter, U256::from_str(test.value).unwrap());
            push!(interpreter, U256::from_str(test.shift).unwrap());
            shl::<DummyHost, LatestSpec>(&mut interpreter, &mut host);
            pop!(interpreter, res);
            assert_eq!(res, U256::from_str(test.expected).unwrap());
        }
    }

    #[test]
    fn test_logical_shift_right() {
        let contract = Contract {
            input: Bytes::default(),
            bytecode: BytecodeLocked::default(),
            ..Default::default()
        };
        let mut host = DummyHost::new(Env::default());
        let mut interpreter = Interpreter::new(contract.clone(), u64::MAX, false);

        struct TestCase {
            value: &'static str,
            shift: &'static str,
            expected: &'static str,
        }

        let test_cases = [
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0x00
            SHR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000001
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0x00",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000001",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0x01
            SHR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0x01",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x01
            SHR
            ---
            0x4000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x8000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x01",
                expected: "0x4000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH 0xff
            SHR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000001
            */
            TestCase {
                value: "0x8000000000000000000000000000000000000000000000000000000000000000",
                shift: "0xff",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000001",
            },
            /*
            PUSH 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x0100
            SHR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x8000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x0100",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x0101
            SHR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x8000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x0101",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x00
            SHR
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x00",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x01
            SHR
            ---
            0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x01",
                expected: "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0xff
            SHR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000001
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0xff",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000001",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x0100
            SHR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x0100",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x01
            SHR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x01",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
        ];

        for test in test_cases {
            host.clear();
            push!(interpreter, U256::from_str(test.value).unwrap());
            push!(interpreter, U256::from_str(test.shift).unwrap());
            shr::<DummyHost, LatestSpec>(&mut interpreter, &mut host);
            pop!(interpreter, res);
            assert_eq!(res, U256::from_str(test.expected).unwrap());
        }
    }

    #[test]
    fn test_arithmetic_shift_right() {
        let contract = Contract {
            input: Bytes::default(),
            bytecode: BytecodeLocked::default(),
            ..Default::default()
        };
        let mut host = DummyHost::new(Env::default());
        let mut interpreter = Interpreter::new(contract.clone(), u64::MAX, false);

        struct TestCase {
            value: &'static str,
            shift: &'static str,
            expected: &'static str,
        }

        let test_cases = [
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0x00
            SAR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000001
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0x00",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000001",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000001
            PUSH 0x01
            SAR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000001",
                shift: "0x01",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x01
            SAR
            ---
            0xc000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x8000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x01",
                expected: "0xc000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH 0xff
            SAR
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0x8000000000000000000000000000000000000000000000000000000000000000",
                shift: "0xff",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x0100
            SAR
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0x8000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x0100",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0x8000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x0101
            SAR
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0x8000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x0101",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x00
            SAR
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x00",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x01
            SAR
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x01",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0xff
            SAR
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0xff",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x0100
            SAR
            ---
            0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            */
            TestCase {
                value: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x0100",
                expected: "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            },
            /*
            PUSH 0x0000000000000000000000000000000000000000000000000000000000000000
            PUSH 0x01
            SAR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x0000000000000000000000000000000000000000000000000000000000000000",
                shift: "0x01",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x4000000000000000000000000000000000000000000000000000000000000000
            PUSH 0xfe
            SAR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000001
            */
            TestCase {
                value: "0x4000000000000000000000000000000000000000000000000000000000000000",
                shift: "0xfe",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000001",
            },
            /*
            PUSH 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0xf8
            SAR
            ---
            0x000000000000000000000000000000000000000000000000000000000000007f
            */
            TestCase {
                value: "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0xf8",
                expected: "0x000000000000000000000000000000000000000000000000000000000000007f",
            },
            /*
            PUSH 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0xfe
            SAR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000001
            */
            TestCase {
                value: "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0xfe",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000001",
            },
            /*
            PUSH 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0xff
            SAR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0xff",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
            /*
            PUSH 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
            PUSH 0x0100
            SAR
            ---
            0x0000000000000000000000000000000000000000000000000000000000000000
            */
            TestCase {
                value: "0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                shift: "0x0100",
                expected: "0x0000000000000000000000000000000000000000000000000000000000000000",
            },
        ];

        for test in test_cases {
            host.clear();
            push!(interpreter, U256::from_str(test.value).unwrap());
            push!(interpreter, U256::from_str(test.shift).unwrap());
            sar::<DummyHost, LatestSpec>(&mut interpreter, &mut host);
            pop!(interpreter, res);
            assert_eq!(res, U256::from_str(test.expected).unwrap());
        }
    }
}
