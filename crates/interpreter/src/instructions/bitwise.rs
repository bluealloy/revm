use super::i256::i256_cmp;
use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{RuntimeFlag, StackTr},
    InstructionContextTr,
};
use core::cmp::Ordering;
use primitives::U256;

/// Implements the LT instruction - less than comparison.
#[inline]
pub fn lt<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);
    *op2 = U256::from(op1 < *op2);
    InstructionReturn::cont()
}

/// Implements the GT instruction - greater than comparison.
#[inline]
pub fn gt<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    *op2 = U256::from(op1 > *op2);
    InstructionReturn::cont()
}

/// Implements the CLZ instruction - count leading zeros.
#[inline]
pub fn clz<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, OSAKA);
    gas!(context, gas::LOW);
    popn_top!([], op1, context);

    let leading_zeros = op1.leading_zeros();
    *op1 = U256::from(leading_zeros);
    InstructionReturn::cont()
}

/// Implements the SLT instruction.
///
/// Signed less than comparison of two values from stack.
#[inline]
pub fn slt<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    *op2 = U256::from(i256_cmp(&op1, op2) == Ordering::Less);
    InstructionReturn::cont()
}

/// Implements the SGT instruction.
///
/// Signed greater than comparison of two values from stack.
#[inline]
pub fn sgt<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    *op2 = U256::from(i256_cmp(&op1, op2) == Ordering::Greater);
    InstructionReturn::cont()
}

/// Implements the EQ instruction.
///
/// Equality comparison of two values from stack.
#[inline]
pub fn eq<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    *op2 = U256::from(op1 == *op2);
    InstructionReturn::cont()
}

/// Implements the ISZERO instruction.
///
/// Checks if the top stack value is zero.
#[inline]
pub fn iszero<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([], op1, context);
    *op1 = U256::from(op1.is_zero());
    InstructionReturn::cont()
}

/// Implements the AND instruction.
///
/// Bitwise AND of two values from stack.
#[inline]
pub fn bitand<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);
    *op2 = op1 & *op2;
    InstructionReturn::cont()
}

/// Implements the OR instruction.
///
/// Bitwise OR of two values from stack.
#[inline]
pub fn bitor<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    *op2 = op1 | *op2;
    InstructionReturn::cont()
}

/// Implements the XOR instruction.
///
/// Bitwise XOR of two values from stack.
#[inline]
pub fn bitxor<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    *op2 = op1 ^ *op2;
    InstructionReturn::cont()
}

/// Implements the NOT instruction.
///
/// Bitwise NOT (negation) of the top stack value.
#[inline]
pub fn not<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([], op1, context);

    *op1 = !*op1;
    InstructionReturn::cont()
}

/// Implements the BYTE instruction.
///
/// Extracts a single byte from a word at a given index.
#[inline]
pub fn byte<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    let o1 = as_usize_saturated!(op1);
    *op2 = if o1 < 32 {
        // `31 - o1` because `byte` returns LE, while we want BE
        U256::from(op2.byte(31 - o1))
    } else {
        U256::ZERO
    };
    InstructionReturn::cont()
}

/// EIP-145: Bitwise shifting instructions in EVM
#[inline]
pub fn shl<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, CONSTANTINOPLE);
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    let shift = as_usize_saturated!(op1);
    *op2 = if shift < 256 {
        *op2 << shift
    } else {
        U256::ZERO
    };
    InstructionReturn::cont()
}

/// EIP-145: Bitwise shifting instructions in EVM
#[inline]
pub fn shr<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, CONSTANTINOPLE);
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    let shift = as_usize_saturated!(op1);
    *op2 = if shift < 256 {
        *op2 >> shift
    } else {
        U256::ZERO
    };
    InstructionReturn::cont()
}

/// EIP-145: Bitwise shifting instructions in EVM
#[inline]
pub fn sar<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, CONSTANTINOPLE);
    gas!(context, gas::VERYLOW);
    popn_top!([op1], op2, context);

    let shift = as_usize_saturated!(op1);
    *op2 = if shift < 256 {
        op2.arithmetic_shr(shift)
    } else if op2.bit(255) {
        U256::MAX
    } else {
        U256::ZERO
    };
    InstructionReturn::cont()
}

#[cfg(test)]
mod tests {
    use crate::{
        host::DummyHost,
        instructions::bitwise::{byte, clz, sar, shl, shr},
        InstructionContext, Interpreter,
    };
    use primitives::{hardfork::SpecId, uint, U256};

    #[test]
    fn test_shift_left() {
        let mut interpreter = Interpreter::default();
        let mut host = DummyHost;

        struct TestCase {
            value: U256,
            shift: U256,
            expected: U256,
        }

        uint! {
            let test_cases = [
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x00_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x01_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000002_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0xff_U256,
                    expected: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x0100_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x0101_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x00_U256,
                    expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x01_U256,
                    expected: 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0xff_U256,
                    expected: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x0100_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x01_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x01_U256,
                    expected: 0xfffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe_U256,
                },
            ];
        }

        for test in test_cases {
            assert!(interpreter.stack.push(test.value));
            assert!(interpreter.stack.push(test.shift));
            shl(&mut InstructionContext::new(&mut interpreter, &mut host));
            let res = interpreter.stack.pop().unwrap();
            assert_eq!(res, test.expected);
        }
    }

    #[test]
    fn test_logical_shift_right() {
        let mut interpreter = Interpreter::default();
        let mut host = DummyHost;

        struct TestCase {
            value: U256,
            shift: U256,
            expected: U256,
        }

        uint! {
            let test_cases = [
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x00_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                    shift: 0x01_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x01_U256,
                    expected: 0x4000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0xff_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x0100_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x0101_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x00_U256,
                    expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x01_U256,
                    expected: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0xff_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    shift: 0x0100_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
                TestCase {
                    value: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                    shift: 0x01_U256,
                    expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                },
            ];
        }

        for test in test_cases {
            assert!(interpreter.stack.push(test.value));
            assert!(interpreter.stack.push(test.shift));
            shr(&mut InstructionContext::new(&mut interpreter, &mut host));
            let res = interpreter.stack.pop().unwrap();
            assert_eq!(res, test.expected);
        }
    }

    #[test]
    fn test_arithmetic_shift_right() {
        let mut interpreter = Interpreter::default();
        let mut host = DummyHost;

        struct TestCase {
            value: U256,
            shift: U256,
            expected: U256,
        }

        uint! {
        let test_cases = [
            TestCase {
                value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                shift: 0x00_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
            },
            TestCase {
                value: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
                shift: 0x01_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
            },
            TestCase {
                value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0x01_U256,
                expected: 0xc000000000000000000000000000000000000000000000000000000000000000_U256,
            },
            TestCase {
                value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0xff_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0x0100_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0x0101_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0x00_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0x01_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0xff_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0x0100_U256,
                expected: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
            },
            TestCase {
                value: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0x01_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
            },
            TestCase {
                value: 0x4000000000000000000000000000000000000000000000000000000000000000_U256,
                shift: 0xfe_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
            },
            TestCase {
                value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0xf8_U256,
                expected: 0x000000000000000000000000000000000000000000000000000000000000007f_U256,
            },
            TestCase {
                value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0xfe_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000001_U256,
            },
            TestCase {
                value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0xff_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
            },
            TestCase {
                value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                shift: 0x0100_U256,
                expected: 0x0000000000000000000000000000000000000000000000000000000000000000_U256,
            },
        ];
            }

        for test in test_cases {
            assert!(interpreter.stack.push(test.value));
            assert!(interpreter.stack.push(test.shift));
            sar(&mut InstructionContext::new(&mut interpreter, &mut host));
            let res = interpreter.stack.pop().unwrap();
            assert_eq!(res, test.expected);
        }
    }

    #[test]
    fn test_byte() {
        struct TestCase {
            input: U256,
            index: usize,
            expected: U256,
        }

        let mut interpreter = Interpreter::default();
        let mut host = DummyHost;

        let input_value = U256::from(0x1234567890abcdef1234567890abcdef_u128);
        let test_cases = (0..32)
            .map(|i| {
                let byte_pos = 31 - i;

                let shift_amount = U256::from(byte_pos * 8);
                let byte_value = (input_value >> shift_amount) & U256::from(0xFF);
                TestCase {
                    input: input_value,
                    index: i,
                    expected: byte_value,
                }
            })
            .collect::<Vec<_>>();

        for test in test_cases.iter() {
            assert!(interpreter.stack.push(test.input));
            assert!(interpreter.stack.push(U256::from(test.index)));
            byte(&mut InstructionContext::new(&mut interpreter, &mut host));
            let res = interpreter.stack.pop().unwrap();
            assert_eq!(res, test.expected, "Failed at index: {}", test.index);
        }
    }

    #[test]
    fn test_clz() {
        let mut interpreter = Interpreter::default();
        interpreter.set_spec_id(SpecId::OSAKA);

        let mut host = DummyHost;

        struct TestCase {
            value: U256,
            expected: U256,
        }

        uint! {
            let test_cases = [
                TestCase { value: 0x0_U256, expected: 256_U256 },
                TestCase { value: 0x1_U256, expected: 255_U256 },
                TestCase { value: 0x2_U256, expected: 254_U256 },
                TestCase { value: 0x3_U256, expected: 254_U256 },
                TestCase { value: 0x4_U256, expected: 253_U256 },
                TestCase { value: 0x7_U256, expected: 253_U256 },
                TestCase { value: 0x8_U256, expected: 252_U256 },
                TestCase { value: 0xff_U256, expected: 248_U256 },
                TestCase { value: 0x100_U256, expected: 247_U256 },
                TestCase { value: 0xffff_U256, expected: 240_U256 },
                TestCase {
                    value: 0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256, // U256::MAX
                    expected: 0_U256,
                },
                TestCase {
                    value: 0x8000000000000000000000000000000000000000000000000000000000000000_U256, // 1 << 255
                    expected: 0_U256,
                },
                TestCase { // Smallest value with 1 leading zero
                    value: 0x4000000000000000000000000000000000000000000000000000000000000000_U256, // 1 << 254
                    expected: 1_U256,
                },
                TestCase { // Value just below 1 << 255
                    value: 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff_U256,
                    expected: 1_U256,
                },
            ];
        }

        for test in test_cases {
            assert!(interpreter.stack.push(test.value));
            clz(&mut InstructionContext::new(&mut interpreter, &mut host));
            let res = interpreter.stack.pop().unwrap();
            assert_eq!(
                res, test.expected,
                "CLZ for value {:#x} failed. Expected: {}, Got: {}",
                test.value, test.expected, res
            );
        }
    }
}
