//! Utility macros to help implementing opcode instruction functions.

/// Fails the instruction if the current call is static.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! require_non_staticcall {
    ($interpreter:expr) => {
        if $interpreter.runtime_flag.is_static() {
            $interpreter.halt($crate::InstructionResult::StateChangeDuringStaticCall);
            return;
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
            $interpreter.halt_not_activated();
            return;
        }
    };
}

/// Records a `gas` cost and fails the instruction if it would exceed the available gas.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! gas {
    ($interpreter:expr, $gas:expr) => {
        $crate::gas!($interpreter, $gas, ())
    };
    ($interpreter:expr, $gas:expr, $ret:expr) => {
        if !$interpreter.gas.record_cost($gas) {
            $interpreter.halt_oog();
            return $ret;
        }
    };
}

/// Loads account and account berlin gas cost accounting.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! berlin_load_account {
    ($context:expr, $address:expr, $load_code:expr) => {
        $crate::berlin_load_account!($context, $address, $load_code, ())
    };
    ($context:expr, $address:expr, $load_code:expr, $ret:expr) => {{
        let skip_cold_load =
            $context.interpreter.gas.remaining() < COLD_ACCOUNT_ACCESS_COST_ADDITIONAL;
        match $context
            .host
            .load_account_info_skip_cold_load($address, $load_code, skip_cold_load)
        {
            Ok(account) => {
                if account.is_cold {
                    $crate::gas!(
                        $context.interpreter,
                        COLD_ACCOUNT_ACCESS_COST_ADDITIONAL,
                        $ret
                    );
                }
                account
            }
            Err(LoadError::ColdLoadSkipped) => {
                $context.interpreter.halt_oog();
                return $ret;
            }
            Err(LoadError::DBError) => {
                $context.interpreter.halt_fatal();
                return $ret;
            }
        }
    }};
}

/// Resizes the interpreter memory if necessary. Fails the instruction if the memory or gas limit
/// is exceeded.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! resize_memory {
    ($interpreter:expr, $offset:expr, $len:expr) => {
        $crate::resize_memory!($interpreter, $offset, $len, ())
    };
    ($interpreter:expr, $offset:expr, $len:expr, $ret:expr) => {
        if let Err(result) = $crate::interpreter::resize_memory(
            &mut $interpreter.gas,
            &mut $interpreter.memory,
            &$interpreter.gas_table,
            $offset,
            $len,
        ) {
            $interpreter.halt(result);
            return $ret;
        }
    };
}

/// Pops n values from the stack. Fails the instruction if n values can't be popped.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! popn {
    ([ $($x:ident),* ],$interpreter:expr $(,$ret:expr)? ) => {
        let Some([$( $x ),*]) = $interpreter.stack.popn() else {
            $interpreter.halt_underflow();
            return $($ret)?;
        };
    };
}

#[doc(hidden)]
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! _count {
    (@count) => { 0 };
    (@count $head:tt $($tail:tt)*) => { 1 + _count!(@count $($tail)*) };
    ($($arg:tt)*) => { _count!(@count $($arg)*) };
}

/// Pops n values from the stack and returns the top value. Fails the instruction if n values can't be popped.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! popn_top {
    ([ $($x:ident),* ], $top:ident, $interpreter:expr $(,$ret:expr)? ) => {
        /*
        let Some(([$( $x ),*], $top)) = $interpreter.stack.popn_top() else {
            $interpreter.halt($crate::InstructionResult::StackUnderflow);
            return $($ret)?;
        };
        */

        // Workaround for https://github.com/rust-lang/rust/issues/144329.
        if $interpreter.stack.len() < (1 + $crate::_count!($($x)*)) {
            $interpreter.halt_underflow();
            return $($ret)?;
        }
        let ([$( $x ),*], $top) = unsafe { $interpreter.stack.popn_top().unwrap_unchecked() };
    };
}

/// Pushes a `B256` value onto the stack. Fails the instruction if the stack is full.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! push {
    ($interpreter:expr, $x:expr $(,$ret:item)?) => (
        if !($interpreter.stack.push($x)) {
            $interpreter.halt_overflow();
            return $($ret)?;
        }
    )
}

/// Converts a `U256` value to a `u64`, saturating to `MAX` if the value is too large.
#[macro_export]
#[collapse_debuginfo(yes)]
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
#[collapse_debuginfo(yes)]
macro_rules! as_usize_saturated {
    ($v:expr) => {
        usize::try_from($crate::as_u64_saturated!($v)).unwrap_or(usize::MAX)
    };
}

/// Converts a `U256` value to a `isize`, saturating to `isize::MAX` if the value is too large.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! as_isize_saturated {
    ($v:expr) => {
        // `isize_try_from(u64::MAX)`` will fail and return isize::MAX
        // This is expected behavior as we are saturating the value.
        isize::try_from($crate::as_u64_saturated!($v)).unwrap_or(isize::MAX)
    };
}

/// Converts a `U256` value to a `usize`, failing the instruction if the value is too large.
#[macro_export]
#[collapse_debuginfo(yes)]
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
#[collapse_debuginfo(yes)]
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
                    $interpreter.halt($reason);
                    return $ret;
                }
                x[0] as usize
            }
        }
    };
}
