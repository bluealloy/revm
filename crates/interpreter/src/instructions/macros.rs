//! Utility macros to help implementing opcode instruction functions.

/// `const` Option `?`.
#[macro_export]
#[collapse_debuginfo(yes)]
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
#[collapse_debuginfo(yes)]
macro_rules! require_non_staticcall {
    ($context:expr) => {
        if $context.runtime_flag().is_static() {
            return $context.halt($crate::InstructionResult::StateChangeDuringStaticCall);
        }
    };
}

/// Macro for optional try - returns early if the expression evaluates to None.
/// Similar to the `?` operator but for use in instruction implementations.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! otry {
    ($expression: expr) => {{
        let Some(value) = $expression else {
            return $crate::instructions::InstructionReturn::halt();
        };
        value
    }};
}

/// Check if the `SPEC` is enabled, and fail the instruction if it is not.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! check {
    ($context:expr, $min:ident) => {
        if !$context
            .runtime_flag()
            .spec_id()
            .is_enabled_in(primitives::hardfork::SpecId::$min)
        {
            return $context.halt($crate::InstructionResult::NotActivated);
        }
    };
}

/// Records a `gas` cost and fails the instruction if it would exceed the available gas.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! gas {
    ($context:expr, $gas:expr) => {
        $crate::gas!(
            $context,
            $gas,
            $crate::instructions::InstructionReturn::halt()
        )
    };
    ($context:expr, $gas:expr, $ret:expr) => {{
        let gas = $gas;
        if !$context.record_gas_cost(gas) {
            $context.halt($crate::InstructionResult::OutOfGas);
            return $ret;
        }
    }};
}

/// Same as [`gas!`], but with `gas` as an option.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! gas_or_fail {
    ($context:expr, $gas:expr) => {
        $crate::gas_or_fail!(
            $context,
            $gas,
            $crate::instructions::InstructionReturn::halt()
        )
    };
    ($context:expr, $gas:expr, $ret:expr) => {
        match $gas {
            Some(gas_used) => $crate::gas!($context, gas_used, $ret),
            None => {
                $context.halt($crate::InstructionResult::OutOfGas);
                return $ret;
            }
        }
    };
}

/// Resizes the interpreter memory if necessary. Fails the instruction if the memory or gas limit
/// is exceeded.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! resize_memory {
    ($context:expr, $offset:expr, $len:expr) => {
        $crate::resize_memory!(
            $context,
            $offset,
            $len,
            $crate::instructions::InstructionReturn::halt()
        )
    };
    ($context:expr, $offset:expr, $len:expr, $ret:expr) => {
        if !$context.resize_memory($offset, $len) {
            $context.halt($crate::InstructionResult::MemoryOOG);
            return $ret;
        }
    };
}

/// Pops n values from the stack. Fails the instruction if n values can't be popped.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! popn {
    ([ $($x:ident),* ], $context:expr) => {
        $crate::popn!([ $($x),* ], $context, $crate::instructions::InstructionReturn::halt())
    };
    ([ $($x:ident),* ], $context:expr, $ret:expr) => {
        let Some([$( $x ),*]) = $context.stack().popn() else {
            $context.halt($crate::InstructionResult::StackUnderflow);
            return $ret;
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
    ([ $($x:ident),* ], $top:ident, $context:expr) => {
        popn_top!([ $($x),* ], $top, $context, $crate::instructions::InstructionReturn::halt())
    };
    ([ $($x:ident),* ], $top:ident, $context:expr, $ret:expr) => {
        // Workaround for https://github.com/rust-lang/rust/issues/144329.
        if $context.stack().len() < (1 + $crate::_count!($($x)*)) {
            $context.halt($crate::InstructionResult::StackUnderflow);
            return $ret;
        }
        let ([$( $x ),*], top) = unsafe { $context.stack().popn_top().unwrap_unchecked() };
        let $top = unsafe { $crate::extend_lt_mut(top) };
    };
}

/// Pushes a `B256` value onto the stack. Fails the instruction if the stack is full.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! push {
    ($context:expr, $x:expr) => {
        $crate::push!(
            $context,
            $x,
            $crate::instructions::InstructionReturn::halt()
        )
    };
    ($context:expr, $x:expr, $ret:expr) => {
        let x = $x;
        if !$context.stack().push(x) {
            $context.halt($crate::InstructionResult::StackOverflow);
            return $ret;
        }
    };
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
    ($context:expr, $v:expr) => {
        $crate::as_usize_or_fail_ret!(
            $context,
            $v,
            $crate::instructions::InstructionReturn::halt()
        )
    };
    ($context:expr, $v:expr, $reason:expr) => {
        $crate::as_usize_or_fail_ret!(
            $context,
            $v,
            $reason,
            $crate::instructions::InstructionReturn::halt()
        )
    };
}

/// Converts a `U256` value to a `usize` and returns `ret`,
/// failing the instruction if the value is too large.
#[macro_export]
#[collapse_debuginfo(yes)]
macro_rules! as_usize_or_fail_ret {
    ($context:expr, $v:expr, $ret:expr) => {
        $crate::as_usize_or_fail_ret!(
            $context,
            $v,
            $crate::InstructionResult::InvalidOperandOOG,
            $ret
        )
    };

    ($context:expr, $v:expr, $reason:expr, $ret:expr) => {
        match $v.as_limbs() {
            x => {
                if (x[0] > usize::MAX as u64) | (x[1] != 0) | (x[2] != 0) | (x[3] != 0) {
                    $context.halt($reason);
                    return $ret;
                }
                x[0] as usize
            }
        }
    };
}

macro_rules! fuck_lt {
    ($e:expr) => {
        unsafe { $crate::extend_lt($e) }
    };
}
