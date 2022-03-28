pub use crate::Return;

macro_rules! try_or_fail {
    ( $e:expr ) => {
        match $e {
            Ok(v) => v,
            Err(e) => return e,
        }
    };
}

macro_rules! check {
    ($expresion:expr) => {
        if !$expresion {
            return Return::NotActivated;
        }
    };
}

macro_rules! gas {
    ($interp:expr, $gas:expr) => {
        if crate::USE_GAS {
            if !$interp.gas.record_cost(($gas)) {
                return Return::OutOfGas;
            }
        }
    };
}

macro_rules! refund {
    ($interp:expr, $gas:expr) => {{
        if crate::USE_GAS {
            $interp.gas.gas_refund($gas);
        }
    }};
}

macro_rules! gas_or_fail {
    ($interp:expr, $gas:expr) => {
        if crate::USE_GAS {
            match $gas {
                Some(gas_used) => gas!($interp, gas_used),
                None => return Return::OutOfGas,
            }
        }
    };
}

macro_rules! memory_resize {
    ($interp:expr, $offset:expr, $len:expr) => {{
        let len: usize = $len;
        let offset: usize = $offset;
        if let Some(new_size) =
            crate::interpreter::memory::next_multiple_of_32(offset.saturating_add(len))
        {
            #[cfg(feature = "memory_limit")]
            if new_size > ($interp.memory_limit as usize) {
                return Return::OutOfGas;
            }

            if new_size > $interp.memory.len() {
                if crate::USE_GAS {
                    let num_bytes = new_size / 32;
                    if !$interp.gas.record_memory(crate::gas::memory_gas(num_bytes)) {
                        return Return::OutOfGas;
                    }
                }
                $interp.memory.resize(new_size);
            }
        } else {
            return Return::OutOfGas;
        }
    }};
}

macro_rules! pop_address {
    ( $interp:expr, $x1:ident) => {
        if $interp.stack.len() < 1 {
            return Return::StackUnderflow;
        }
        let mut temp = H256::zero();
        // Safety: Length is checked above.
        let $x1: H160 = {
            unsafe {
                $interp
                    .stack
                    .pop_unsafe()
                    .to_big_endian(temp.as_bytes_mut())
            };
            temp.into()
        };
    };
    ( $interp:expr, $x1:ident, $x2:ident) => {
        if $interp.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        let mut temp = H256::zero();
        $x1: H160 = {
            // Safety: Length is checked above.
            unsafe {
                $interp
                    .stack
                    .pop_unsafe()
                    .to_big_endian(temp.as_bytes_mut())
            };
            temp.into()
        };
        $x2: H160 = {
            temp = H256::zero();
            // Safety: Length is checked above.
            unsafe {
                $interp
                    .stack
                    .pop_unsafe()
                    .to_big_endian(temp.as_bytes_mut())
            };
            temp.into();
        };
    };
}

macro_rules! pop {
    ( $interp:expr, $x1:ident) => {
        if $interp.stack.len() < 1 {
            return Return::StackUnderflow;
        }
        // Safety: Length is checked above.
        let $x1 = unsafe { $interp.stack.pop_unsafe() };
    };
    ( $interp:expr, $x1:ident, $x2:ident) => {
        if $interp.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        // Safety: Length is checked above.
        let ($x1, $x2) = unsafe { $interp.stack.pop2_unsafe() };
    };
    ( $interp:expr, $x1:ident, $x2:ident, $x3:ident) => {
        if $interp.stack.len() < 3 {
            return Return::StackUnderflow;
        }
        // Safety: Length is checked above.
        let ($x1, $x2, $x3) = unsafe { $interp.stack.pop3_unsafe() };
    };

    ( $interp:expr, $x1:ident, $x2:ident, $x3:ident, $x4:ident) => {
        if $interp.stack.len() < 4 {
            return Return::StackUnderflow;
        }
        // Safety: Length is checked above.
        let ($x1, $x2, $x3, $x4) = unsafe { $interp.stack.pop4_unsafe() };
    };
}

macro_rules! pop_top {
    ( $interp:expr, $x1:ident) => {
        if $interp.stack.len() < 1 {
            return Return::StackUnderflow;
        }
        // Safety: Length is checked above.
        let $x1 = unsafe { $interp.stack.top_unsafe() };
    };
    ( $interp:expr, $x1:ident, $x2:ident) => {
        if $interp.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        // Safety: Length is checked above.
        let ($x1, $x2) = unsafe { $interp.stack.pop_top_unsafe() };
    };
    ( $interp:expr, $x1:ident, $x2:ident, $x3:ident) => {
        if $interp.stack.len() < 3 {
            return Return::StackUnderflow;
        }
        // Safety: Length is checked above.
        let ($x1, $x2, $x3) = unsafe { $interp.stack.pop2_top_unsafe() };
    };
}

macro_rules! push_h256 {
	( $interp:expr, $( $x:expr ),* ) => (
		$(
			match $interp.stack.push_h256($x) {
				Ok(()) => (),
				Err(e) => return e,
			}
		)*
	)
}

macro_rules! push {
    ( $interp:expr, $( $x:expr ),* ) => (
		$(
			match $interp.stack.push($x) {
				Ok(()) => (),
				Err(e) => return e,
			}
		)*
	)
}

macro_rules! op1_u256_fn {
    ( $interp:expr, $op:path ) => {{
        // gas!($interp, $gas);
        pop_top!($interp, op1);
        *op1 = $op(*op1);

        Return::Continue
    }};
}

macro_rules! op2_u256_bool_ref {
    ( $interp:expr, $op:ident) => {{
        // gas!($interp, $gas);
        pop_top!($interp, op1, op2);
        let ret = op1.$op(&op2);
        *op2 = if ret { U256::one() } else { U256::zero() };

        Return::Continue
    }};
}

macro_rules! op2_u256 {
    ( $interp:expr, $op:ident) => {{
        // gas!($interp, $gas);
        pop_top!($interp, op1, op2);
        *op2 = op1.$op(*op2);
        Return::Continue
    }};
}

macro_rules! op2_u256_tuple {
    ( $interp:expr, $op:ident) => {{
        // gas!($interp, $gas);

        pop_top!($interp, op1, op2);
        let (ret, ..) = op1.$op(*op2);
        *op2 = ret;

        Return::Continue
    }};
    ( $interp:expr, $op:ident ) => {{
        pop_top!($interp, op1, op2);
        let (ret, ..) = op1.$op(op2);
        *op2 = ret;

        Return::Continue
    }};
}

macro_rules! op2_u256_fn {
    ( $interp:expr, $op:path ) => {{
        // gas!($interp, $gas);

        pop_top!($interp, op1, op2);
        *op2 = $op(op1, *op2);

        Return::Continue
    }};
    ( $interp:expr, $op:path, $enabled:expr) => {{
        check!(($enabled));
        op2_u256_fn!($interp, $op)
    }};
}

macro_rules! op3_u256_fn {
    ( $interp:expr, $op:path) => {{
        // gas!($interp, $gas);

        pop_top!($interp, op1, op2, op3);
        *op3 = $op(op1, op2, *op3);

        Return::Continue
    }};
    ( $interp:expr, $op:path, $spec:ident :: $enabled:ident) => {{
        check!($spec::$enabled);
        op3_u256_fn!($interp, $op)
    }};
}

macro_rules! as_usize_saturated {
    ( $v:expr ) => {{
        if $v.0[1] != 0 || $v.0[2] != 0 || $v.0[3] != 0 {
            usize::MAX
        } else {
            $v.0[0] as usize
        }
    }};
}

macro_rules! as_usize_or_fail {
    ( $v:expr ) => {{
        if $v.0[1] != 0 || $v.0[2] != 0 || $v.0[3] != 0 {
            return Return::OutOfGas;
        }

        $v.0[0] as usize
    }};

    ( $v:expr, $reason:expr ) => {{
        if $v.0[1] != 0 || $v.0[2] != 0 || $v.0[3] != 0 {
            return $reason;
        }

        $v.0[0] as usize
    }};
}
