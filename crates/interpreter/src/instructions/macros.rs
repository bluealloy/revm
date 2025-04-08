//! Utility macros to help implementing opcode instruction functions.

/// `const` Option `?`.
#[macro_export]
macro_rules! tri {
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return None,
        }
    };
}

/// Fails the instruction if the current call is static.
#[macro_export]
macro_rules! require_non_staticcall {
    ($interpreter:expr) => {
        if $interpreter.runtime_flag.is_static() {
            $interpreter
                .control
                .set_instruction_result($crate::InstructionResult::StateChangeDuringStaticCall);
            return;
        }
    };
}

#[macro_export]
macro_rules! otry {
    ($expression: expr) => {{
        let Some(value) = $expression else {
            return;
        };
        value
    }};
}

/// Error if the current call is executing EOF.
#[macro_export]
macro_rules! require_eof {
    ($interpreter:expr) => {
        if !$interpreter.runtime_flag.is_eof() {
            $interpreter
                .control
                .set_instruction_result($crate::InstructionResult::EOFOpcodeDisabledInLegacy);
            return;
        }
    };
}

/// Check if the `SPEC` is enabled, and fail the instruction if it is not.
#[macro_export]
macro_rules! check {
    ($interpreter:expr, $min:ident) => {
        if !$interpreter
            .runtime_flag
            .spec_id()
            .is_enabled_in(primitives::hardfork::SpecId::$min)
        {
            $interpreter
                .control
                .set_instruction_result($crate::InstructionResult::NotActivated);
            return;
        }
    };
}

/// Records a `gas` cost and fails the instruction if it would exceed the available gas.
#[macro_export]
macro_rules! gas {
    ($interpreter:expr, $gas:expr) => {
        $crate::gas!($interpreter, $gas, ())
    };
    ($interpreter:expr, $gas:expr, $ret:expr) => {
        if !$interpreter.control.gas_mut().record_cost($gas) {
            $interpreter
                .control
                .set_instruction_result($crate::InstructionResult::OutOfGas);
            return $ret;
        }
    };
}

/// Same as [`gas!`], but with `gas` as an option.
#[macro_export]
macro_rules! gas_or_fail {
    ($interpreter:expr, $gas:expr) => {
        $crate::gas_or_fail!($interpreter, $gas, ())
    };
    ($interpreter:expr, $gas:expr, $ret:expr) => {
        match $gas {
            Some(gas_used) => $crate::gas!($interpreter, gas_used, $ret),
            None => {
                $interpreter
                    .control
                    .set_instruction_result($crate::InstructionResult::OutOfGas);
                return $ret;
            }
        }
    };
}

/// Resizes the interpreterreter memory if necessary. Fails the instruction if the memory or gas limit
/// is exceeded.
#[macro_export]
macro_rules! resize_memory {
    ($interpreter:expr, $offset:expr, $len:expr) => {
        $crate::resize_memory!($interpreter, $offset, $len, ())
    };
    ($interpreter:expr, $offset:expr, $len:expr, $ret:expr) => {
        let words_num = $crate::interpreter::num_words($offset.saturating_add($len));
        match $interpreter
            .control
            .gas_mut()
            .record_memory_expansion(words_num)
        {
            $crate::gas::MemoryExtensionResult::Extended => {
                $interpreter.memory.resize(words_num * 32);
            }
            $crate::gas::MemoryExtensionResult::OutOfGas => {
                $interpreter
                    .control
                    .set_instruction_result($crate::InstructionResult::MemoryOOG);
                return $ret;
            }
            $crate::gas::MemoryExtensionResult::Same => (), // no action
        };
    };
}

/// Pops n values from the stack. Fails the instruction if n values can't be popped.
#[macro_export]
macro_rules! popn {
    ([ $($x:ident),* ],$interpreterreter:expr $(,$ret:expr)? ) => {
        let Some([$( $x ),*]) = $interpreterreter.stack.popn() else {
            $interpreterreter.control.set_instruction_result($crate::InstructionResult::StackUnderflow);
            return $($ret)?;
        };
    };
}

/// Pops n values from the stack and returns the top value. Fails the instruction if n values can't be popped.
#[macro_export]
macro_rules! popn_top {
    ([ $($x:ident),* ], $top:ident, $interpreterreter:expr $(,$ret:expr)? ) => {
        let Some(([$( $x ),*], $top)) = $interpreterreter.stack.popn_top() else {
            $interpreterreter.control.set_instruction_result($crate::InstructionResult::StackUnderflow);
            return $($ret)?;
        };
    };
}

/// Pushes a `B256` value onto the stack. Fails the instruction if the stack is full.
#[macro_export]
macro_rules! push {
    ($interpreter:expr, $x:expr $(,$ret:item)?) => (
        if !($interpreter.stack.push($x)) {
            $interpreter.control.set_instruction_result($crate::InstructionResult::StackOverflow);
            return $($ret)?;
        }
    )
}

/// Converts a `U256` value to a `u64`, saturating to `MAX` if the value is too large.
#[macro_export]
macro_rules! as_u64_saturated {
    ($v:expr) => {
        match $v.as_limbs() {
            x => {
                if (x[1] == 0) & (x[2] == 0) & (x[3] == 0) {
                    x[0]
                } else {
                    u64::MAX
                }
            }
        }
    };
}

/// Converts a `U256` value to a `usize`, saturating to `MAX` if the value is too large.
#[macro_export]
macro_rules! as_usize_saturated {
    ($v:expr) => {
        usize::try_from($crate::as_u64_saturated!($v)).unwrap_or(usize::MAX)
    };
}

/// Converts a `U256` value to a `isize`, saturating to `isize::MAX` if the value is too large.
#[macro_export]
macro_rules! as_isize_saturated {
    ($v:expr) => {
        // `isize_try_from(u64::MAX)`` will fail and return isize::MAX
        // This is expected behavior as we are saturating the value.
        isize::try_from($crate::as_u64_saturated!($v)).unwrap_or(isize::MAX)
    };
}

/// Converts a `U256` value to a `usize`, failing the instruction if the value is too large.
#[macro_export]
macro_rules! as_usize_or_fail {
    ($interpreter:expr, $v:expr) => {
        $crate::as_usize_or_fail_ret!($interpreter, $v, ())
    };
    ($interpreter:expr, $v:expr, $reason:expr) => {
        $crate::as_usize_or_fail_ret!($interpreter, $v, $reason, ())
    };
}

/// Converts a `U256` value to a `usize` and returns `ret`,
/// failing the instruction if the value is too large.
#[macro_export]
macro_rules! as_usize_or_fail_ret {
    ($interpreter:expr, $v:expr, $ret:expr) => {
        $crate::as_usize_or_fail_ret!(
            $interpreter,
            $v,
            $crate::InstructionResult::InvalidOperandOOG,
            $ret
        )
    };

    ($interpreter:expr, $v:expr, $reason:expr, $ret:expr) => {
        match $v.as_limbs() {
            x => {
                if (x[0] > usize::MAX as u64) | (x[1] != 0) | (x[2] != 0) | (x[3] != 0) {
                    $interpreter.control.set_instruction_result($reason);
                    return $ret;
                }
                x[0] as usize
            }
        }
    };
}
