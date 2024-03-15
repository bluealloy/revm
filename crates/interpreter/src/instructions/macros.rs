macro_rules! check_staticcall {
    ($interp:expr) => {
        if $interp.is_static {
            $interp.instruction_result = InstructionResult::StateChangeDuringStaticCall;
            return;
        }
    };
}

macro_rules! check {
    ($interp:expr, $min:ident) => {
        // TODO: Force const-eval on the condition with a `const {}` block once they are stable
        if !<SPEC as $crate::primitives::Spec>::enabled($crate::primitives::SpecId::$min) {
            $interp.instruction_result = InstructionResult::NotActivated;
            return;
        }
    };
}

#[macro_export]
macro_rules! gas {
    ($interp:expr, $gas:expr) => {
        gas!($interp, $gas, ())
    };
    ($interp:expr, $gas:expr, $ret:expr) => {
        if !$interp.gas.record_cost($gas) {
            $interp.instruction_result = InstructionResult::OutOfGas;
            return $ret;
        }
    };
}

macro_rules! refund {
    ($interp:expr, $gas:expr) => {
        $interp.gas.record_refund($gas)
    };
}

macro_rules! gas_or_fail {
    ($interp:expr, $gas:expr) => {
        match $gas {
            Some(gas_used) => gas!($interp, gas_used),
            None => {
                $interp.instruction_result = InstructionResult::OutOfGas;
                return;
            }
        }
    };
}

#[macro_export]
macro_rules! shared_memory_resize {
    ($interp:expr, $offset:expr, $len:expr) => {
        shared_memory_resize!($interp, $offset, $len, ())
    };
    ($interp:expr, $offset:expr, $len:expr, $ret:expr) => {
        let size = $offset.saturating_add($len);
        if size > $interp.shared_memory.len() {
            // We are fine with saturating to usize if size is close to MAX value.
            let rounded_size = $crate::interpreter::next_multiple_of_32(size);

            #[cfg(feature = "memory_limit")]
            if $interp.shared_memory.limit_reached(size) {
                $interp.instruction_result = InstructionResult::MemoryLimitOOG;
                return $ret;
            }

            // Gas is calculated in evm words (256bits).
            let words_num = rounded_size / 32;
            if !$interp
                .gas
                .record_memory($crate::gas::memory_gas(words_num))
            {
                $interp.instruction_result = InstructionResult::MemoryLimitOOG;
                return $ret;
            }
            $interp.shared_memory.resize(rounded_size);
        }
    };
}

macro_rules! pop_address {
    ($interp:expr, $x1:ident) => {
        if $interp.stack.len() < 1 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return;
        }
        // SAFETY: Length is checked above.
        let $x1 = Address::from_word(B256::from(unsafe { $interp.stack.pop_unsafe() }));
    };
    ($interp:expr, $x1:ident, $x2:ident) => {
        if $interp.stack.len() < 2 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return;
        }
        // SAFETY: Length is checked above.
        let $x1 = Address::from_word(B256::from(unsafe { $interp.stack.pop_unsafe() }));
        let $x2 = Address::from_word(B256::from(unsafe { $interp.stack.pop_unsafe() }));
    };
}

#[macro_export]
macro_rules! pop {
    ($interp:expr, $x1:ident) => {
        $crate::pop_ret!($interp, $x1, ())
    };
    ($interp:expr, $x1:ident, $x2:ident) => {
        $crate::pop_ret!($interp, $x1, $x2, ())
    };
    ($interp:expr, $x1:ident, $x2:ident, $x3:ident) => {
        $crate::pop_ret!($interp, $x1, $x2, $x3, ())
    };
    ($interp:expr, $x1:ident, $x2:ident, $x3:ident, $x4:ident) => {
        $crate::pop_ret!($interp, $x1, $x2, $x3, $x4, ())
    };
}

#[macro_export]
macro_rules! pop_ret {
    ($interp:expr, $x1:ident, $ret:expr) => {
        if $interp.stack.len() < 1 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return $ret;
        }
        // SAFETY: Length is checked above.
        let $x1 = unsafe { $interp.stack.pop_unsafe() };
    };
    ($interp:expr, $x1:ident, $x2:ident, $ret:expr) => {
        if $interp.stack.len() < 2 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return $ret;
        }
        // SAFETY: Length is checked above.
        let ($x1, $x2) = unsafe { $interp.stack.pop2_unsafe() };
    };
    ($interp:expr, $x1:ident, $x2:ident, $x3:ident, $ret:expr) => {
        if $interp.stack.len() < 3 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return $ret;
        }
        // SAFETY: Length is checked above.
        let ($x1, $x2, $x3) = unsafe { $interp.stack.pop3_unsafe() };
    };
    ($interp:expr, $x1:ident, $x2:ident, $x3:ident, $x4:ident, $ret:expr) => {
        if $interp.stack.len() < 4 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return $ret;
        }
        // SAFETY: Length is checked above.
        let ($x1, $x2, $x3, $x4) = unsafe { $interp.stack.pop4_unsafe() };
    };
}

macro_rules! pop_top {
    ($interp:expr, $x1:ident) => {
        if $interp.stack.len() < 1 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return;
        }
        // SAFETY: Length is checked above.
        let $x1 = unsafe { $interp.stack.top_unsafe() };
    };
    ($interp:expr, $x1:ident, $x2:ident) => {
        if $interp.stack.len() < 2 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return;
        }
        // SAFETY: Length is checked above.
        let ($x1, $x2) = unsafe { $interp.stack.pop_top_unsafe() };
    };
    ($interp:expr, $x1:ident, $x2:ident, $x3:ident) => {
        if $interp.stack.len() < 3 {
            $interp.instruction_result = InstructionResult::StackUnderflow;
            return;
        }
        // SAFETY: Length is checked above.
        let ($x1, $x2, $x3) = unsafe { $interp.stack.pop2_top_unsafe() };
    };
}

#[macro_export]
macro_rules! push_b256 {
	($interp:expr, $($x:expr),* $(,)?) => ($(
        match $interp.stack.push_b256($x) {
            Ok(()) => {},
            Err(e) => {
                $interp.instruction_result = e;
                return;
            },
        }
    )*)
}

#[macro_export]
macro_rules! push {
    ($interp:expr, $($x:expr),* $(,)?) => ($(
        match $interp.stack.push($x) {
            Ok(()) => {},
            Err(e) => {
                $interp.instruction_result = e;
                return;
            }
        }
    )*)
}

macro_rules! as_u64_saturated {
    ($v:expr) => {{
        let x: &[u64; 4] = $v.as_limbs();
        if x[1] == 0 && x[2] == 0 && x[3] == 0 {
            x[0]
        } else {
            u64::MAX
        }
    }};
}

macro_rules! as_usize_saturated {
    ($v:expr) => {
        usize::try_from(as_u64_saturated!($v)).unwrap_or(usize::MAX)
    };
}

macro_rules! as_usize_or_fail {
    ($interp:expr, $v:expr) => {
        as_usize_or_fail_ret!($interp, $v, ())
    };
    ($interp:expr, $v:expr, $reason:expr) => {
        as_usize_or_fail_ret!($interp, $v, $reason, ())
    };
}

macro_rules! as_usize_or_fail_ret {
    ($interp:expr, $v:expr, $ret:expr) => {
        as_usize_or_fail_ret!($interp, $v, InstructionResult::InvalidOperandOOG, $ret)
    };

    ($interp:expr, $v:expr, $reason:expr, $ret:expr) => {{
        let x = $v.as_limbs();
        if x[1] != 0 || x[2] != 0 || x[3] != 0 {
            $interp.instruction_result = $reason;
            return $ret;
        }
        let Ok(val) = usize::try_from(x[0]) else {
            $interp.instruction_result = $reason;
            return $ret;
        };
        val
    }};
}
