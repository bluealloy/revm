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
    ($machine:expr, $gas:expr) => {
        if crate::USE_GAS {
            if !$machine.gas.record_cost(($gas)) {
                return Return::OutOfGas;
            }
        }
    };
}

macro_rules! refund {
    ($machine:expr, $gas:expr) => {{
        if crate::USE_GAS {
            $machine.gas.gas_refund($gas);
        }
    }};
}

macro_rules! gas_or_fail {
    ($machine:expr, $gas:expr) => {
        if crate::USE_GAS {
            match $gas {
                Some(gas_used) => gas!($machine, gas_used),
                None => return Return::OutOfGas,
            }
        }
    };
}

macro_rules! memory_resize {
    ($machine:expr, $offset:expr, $len:expr) => {{
        let len: usize = $len;
        if len != 0 {
            let offset: usize = $offset;
            if let Some(new_size) =
                crate::machine::memory::next_multiple_of_32(offset.saturating_add(len))
            {
                let num_bytes = new_size / 32;
                if !$machine
                    .gas
                    .record_memory(crate::instructions::gas::memory_gas(num_bytes))
                {
                    return Return::OutOfGas;
                }
                $machine.memory.resize(new_size);
            } else {
                return Return::OutOfGas;
            }
        }
    }};
}

macro_rules! pop_address {
    ( $machine:expr, $x1:ident) => {
        if $machine.stack.len() < 1 {
            return Return::StackUnderflow;
        }
        let mut temp = H256::zero();
        let $x1: H160 = {
            unsafe {
                $machine
                    .stack
                    .pop_unsafe()
                    .to_big_endian(temp.as_bytes_mut())
            };
            temp.into()
        };
    };
    ( $machine:expr, $x1:ident, $x2:ident) => {
        if $machine.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        let mut temp = H256::zero();
        $x1: H160 = {
            unsafe {
                $machine
                    .stack
                    .pop_unsafe()
                    .to_big_endian(temp.as_bytes_mut())
            };
            temp.into()
        };
        $x2: H160 = {
            temp = H256::zero();
            unsafe {
                $machine
                    .stack
                    .pop_unsafe()
                    .to_big_endian(temp.as_bytes_mut())
            };
            temp.into();
        };
    };
}

macro_rules! pop {
    ( $machine:expr, $x1:ident) => {
        if $machine.stack.len() < 1 {
            return Return::StackUnderflow;
        }
        let $x1 = unsafe { $machine.stack.pop_unsafe() };
    };
    ( $machine:expr, $x1:ident, $x2:ident) => {
        if $machine.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        let $x1 = unsafe { $machine.stack.pop_unsafe() };
        let $x2 = unsafe { $machine.stack.pop_unsafe() };
    };
    ( $machine:expr, $x1:ident, $x2:ident, $x3:ident) => {
        if $machine.stack.len() < 3 {
            return Return::StackUnderflow;
        }
        let $x1 = unsafe { $machine.stack.pop_unsafe() };
        let $x2 = unsafe { $machine.stack.pop_unsafe() };
        let $x3 = unsafe { $machine.stack.pop_unsafe() };
    };

    ( $machine:expr, $x1:ident, $x2:ident, $x3:ident, $x4:ident) => {
        if $machine.stack.len() < 4 {
            return Return::StackUnderflow;
        }
        let $x1 = unsafe { $machine.stack.pop_unsafe() };
        let $x2 = unsafe { $machine.stack.pop_unsafe() };
        let $x3 = unsafe { $machine.stack.pop_unsafe() };
        let $x4 = unsafe { $machine.stack.pop_unsafe() };
    };
}

macro_rules! push_h256 {
	( $machine:expr, $( $x:expr ),* ) => (
		$(
			match $machine.stack.push_h256($x) {
				Ok(()) => (),
				Err(e) => return e,
			}
		)*
	)
}

macro_rules! push {
    ( $machine:expr, $( $x:expr ),* ) => (
		$(
			match $machine.stack.push($x) {
				Ok(()) => (),
				Err(e) => return e,
			}
		)*
	)
}

macro_rules! op1_u256_fn {
    ( $machine:expr, $op:path, $gas:expr ) => {{
        gas!($machine, $gas);
        pop!($machine, op1);
        let ret = $op(op1);
        push!($machine, ret);

        Return::Continue
    }};
}

macro_rules! op2_u256_bool_ref {
    ( $machine:expr, $op:ident, $gas:expr ) => {{
        gas!($machine, $gas);
        pop!($machine, op1, op2);
        let ret = op1.$op(&op2);
        push!($machine, if ret { U256::one() } else { U256::zero() });

        Return::Continue
    }};
}

macro_rules! op2_u256 {
    ( $machine:expr, $op:ident, $gas:expr ) => {{
        gas!($machine, $gas);
        pop!($machine, op1, op2);
        let ret = op1.$op(op2);
        push!($machine, ret);

        Return::Continue
    }};
}

macro_rules! op2_u256_tuple {
    ( $machine:expr, $op:ident, $gas:expr ) => {{
        gas!($machine, $gas);

        pop!($machine, op1, op2);
        let (ret, ..) = op1.$op(op2);
        push!($machine, ret);

        Return::Continue
    }};
}

macro_rules! op2_u256_fn {
    ( $machine:expr, $op:path, $gas:expr  ) => {{
        gas!($machine, $gas);

        pop!($machine, op1, op2);
        let ret = $op(op1, op2);
        push!($machine, ret);

        Return::Continue
    }};
    ( $machine:expr, $op:path, $gas:expr, $enabled:expr) => {{
        check!(($enabled));
        op2_u256_fn!($machine, $op, $gas)
    }};
}

macro_rules! op3_u256_fn {
    ( $machine:expr, $op:path, $gas:expr  ) => {{
        gas!($machine, $gas);

        pop!($machine, op1, op2, op3);
        let ret = $op(op1, op2, op3);
        push!($machine, ret);

        Return::Continue
    }};
    ( $machine:expr, $op:path, $gas:expr, $spec:ident :: $enabled:ident) => {{
        check!($spec::$enabled);
        op3_u256_fn!($machine, $op, $gas)
    }};
}

macro_rules! as_usize_saturated {
    ( $v:expr ) => {{
        if unsafe {
            *$v.0.get_unchecked(1) != 0
                || *$v.0.get_unchecked(2) != 0
                || *$v.0.get_unchecked(3) != 0
        } {
            usize::MAX
        } else {
            unsafe { *$v.0.get_unchecked(0) as usize }
        }
    }};
}

macro_rules! as_usize_or_fail {
    ( $v:expr ) => {{
        if unsafe {
            *$v.0.get_unchecked(1) != 0
                || *$v.0.get_unchecked(2) != 0
                || *$v.0.get_unchecked(3) != 0
        } {
            return Return::OutOfGas;
        }

        unsafe { *$v.0.get_unchecked(0) as usize }
    }};

    ( $v:expr, $reason:expr ) => {{
        if unsafe {
            *$v.0.get_unchecked(1) != 0
                || *$v.0.get_unchecked(2) != 0
                || *$v.0.get_unchecked(3) != 0
        } {
            return $reason;
        }

        unsafe { *$v.0.get_unchecked(0) as usize }
    }};
}
