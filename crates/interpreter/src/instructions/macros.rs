//! Utility macros to help implementing opcode instruction functions.

/// Fails the instruction if the current call is static.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! require_non_staticcall {
    ($interpreter:expr) => {
        if $interpreter.runtime_flag.is_static() {
            $crate::primitives::hints_util::cold_path();
            return Err($crate::InstructionResult::StateChangeDuringStaticCall);
        }
    };
}

/// Check if the `SPEC` is enabled, and fail the instruction if it is not.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! check {
    ($interpreter:expr, $min:ident) => {
        if !$interpreter
            .runtime_flag
            .spec_id()
            .is_enabled_in(primitives::hardfork::SpecId::$min)
        {
            $crate::primitives::hints_util::cold_path();
            return Err($crate::InstructionResult::NotActivated);
        }
    };
}

/// Records a state gas cost (EIP-8037) and fails the instruction if it would exceed the available gas.
/// State gas only deducts from `remaining` (not `regular_gas_remaining`).
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! state_gas {
    ($interpreter:expr, $gas:expr) => {{
        if !$interpreter.gas.record_state_cost($gas) {
            $crate::primitives::hints_util::cold_path();
            return Err($crate::InstructionResult::OutOfGas);
        }
    }};
}

/// Records a `gas` cost and fails the instruction if it would exceed the available gas.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! gas {
    ($interpreter:expr, $gas:expr) => {
        if !$interpreter.gas.record_regular_cost($gas) {
            $crate::primitives::hints_util::cold_path();
            return Err($crate::InstructionResult::OutOfGas);
        }
    };
}

/// Pops n values from the stack. Fails the instruction if n values can't be popped.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! popn {
    ([ $($x:ident),* ],$interpreter:expr) => {
        let Some([$( $x ),*]) = $interpreter.stack.popn() else {
            $crate::primitives::hints_util::cold_path();
            return Err($crate::InstructionResult::StackUnderflow);
        };
    };
}

#[doc(hidden)]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! _count {
    (@count) => { 0 };
    (@count $head:tt $($tail:tt)*) => { 1 + $crate::_count!(@count $($tail)*) };
    ($($arg:tt)*) => { $crate::_count!(@count $($arg)*) };
}

/// Pops n values from the stack and returns the top value. Fails the instruction if n values can't be popped.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! popn_top {
    ([ $($x:ident),* ], $top:ident, $interpreter:expr) => {
        /*
        let Some(([$( $x ),*], $top)) = $interpreter.stack.popn_top() else {
            $crate::primitives::hints_util::cold_path();
            return Err($crate::InstructionResult::StackUnderflow);
        };
        */

        // Workaround for https://github.com/rust-lang/rust/issues/144329.
        if $interpreter.stack.len() < (1 + $crate::_count!($($x)*)) {
            $crate::primitives::hints_util::cold_path();
            return Err($crate::InstructionResult::StackUnderflow);
        }
        let ([$( $x ),*], $top) = unsafe { $crate::interpreter_types::StackTr::popn_top(&mut $interpreter.stack).unwrap_unchecked() };
    };
}

/// Pushes a `B256` value onto the stack. Fails the instruction if the stack is full.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! push {
    ($interpreter:expr, $x:expr) => {
        if !$interpreter.stack.push($x) {
            $crate::primitives::hints_util::cold_path();
            return Err($crate::InstructionResult::StackOverflow);
        }
    };
}

/// Converts a `U256` value to a `u64`, saturating to `MAX` if the value is too large.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! as_u64_saturated {
    ($v:expr) => {
        u64::try_from($v).unwrap_or(u64::MAX)
    };
}

/// Converts a `U256` value to a `usize`, saturating to `MAX` if the value is too large.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! as_usize_saturated {
    ($v:expr) => {
        usize::try_from($v).unwrap_or(usize::MAX)
    };
}

/// Converts a `U256` value to a `usize`, failing the instruction if the value is too large.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! as_usize_or_fail {
    ($interpreter:expr, $v:expr) => {
        match $v.as_limbs() {
            x => {
                if (x[0] > usize::MAX as u64) | (x[1] != 0) | (x[2] != 0) | (x[3] != 0) {
                    $crate::primitives::hints_util::cold_path();
                    return Err($crate::InstructionResult::InvalidOperandOOG);
                }
                x[0] as usize
            }
        }
    };
}
