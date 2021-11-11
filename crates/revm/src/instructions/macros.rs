use primitive_types::{H256, U256};

pub use crate::Return;

macro_rules! try_or_fail {
    ( $e:expr ) => {
        match $e {
            Ok(v) => v,
            Err(e) => return e,
        }
    };
}

macro_rules! inspect {
    ($handler:ident, $inspect_fn:ident) => {
        if H::INSPECT {
            $handler.inspect().$inspect_fn();
        }
    };
    ($handler:ident, $inspect_fn:ident, $($args:expr),*) => {
        if H::INSPECT {
            $handler.inspect().$inspect_fn( $($args),* );
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
    ($machine:expr, $start:expr, $len:expr) => {{
        let new_gas_memory = try_or_fail!($machine.memory.resize_offset($start, $len));
        if crate::USE_GAS {
            if !$machine.gas.record_memory(new_gas_memory) {
                return Return::OutOfGas;
            }
        }
    }};
}

macro_rules! pop {
    ( $machine:expr, $x1:ident) => {
        if $machine.stack.len() < 1 {
            return Return::StackUnderflow;
        }

        let mut $x1 = H256::zero();
        unsafe {
            $machine
                .stack
                .pop_unsafe()
                .to_big_endian($x1.as_bytes_mut())
        };
    };
    ( $machine:expr, $x1:ident, $x2:ident) => {
        if $machine.stack.len() < 2 {
            return Return::StackUnderflow;
        }
        let mut $x1 = H256::zero();
        unsafe {
            $machine
                .stack
                .pop_unsafe()
                .to_big_endian($x1.as_bytes_mut())
        };
        let mut $x2 = H256::zero();
        unsafe {
            $machine
                .stack
                .pop_unsafe()
                .to_big_endian($x2.as_bytes_mut())
        };
    };
}

macro_rules! pop_u256 {
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

macro_rules! push_u256 {
	( $machine:expr, $( $x:expr ),* ) => (
		$(
			let mut value = H256::default();
			$x.to_big_endian(&mut value[..]);
			match $machine.stack.push(value) {
				Ok(()) => (),
				Err(e) => return e,
			}
		)*
	)
}

macro_rules! op1_u256_fn {
    ( $machine:expr, $op:path, $gas:expr ) => {{
        gas!($machine, $gas);
        pop_u256!($machine, op1);
        let ret = $op(op1);
        push_u256!($machine, ret);

        Return::Continue
    }};
}

macro_rules! op2_u256_bool_ref {
    ( $machine:expr, $op:ident, $gas:expr ) => {{
        gas!($machine, $gas);
        pop_u256!($machine, op1, op2);
        let ret = op1.$op(&op2);
        push_u256!($machine, if ret { U256::one() } else { U256::zero() });

        Return::Continue
    }};
}

macro_rules! op2_u256 {
    ( $machine:expr, $op:ident, $gas:expr ) => {{
        gas!($machine, $gas);
        pop_u256!($machine, op1, op2);
        let ret = op1.$op(op2);
        push_u256!($machine, ret);

        Return::Continue
    }};
}

macro_rules! op2_u256_tuple {
    ( $machine:expr, $op:ident, $gas:expr ) => {{
        gas!($machine, $gas);

        pop_u256!($machine, op1, op2);
        let (ret, ..) = op1.$op(op2);
        push_u256!($machine, ret);

        Return::Continue
    }};
}

macro_rules! op2_u256_fn {
    ( $machine:expr, $op:path, $gas:expr  ) => {{
        gas!($machine, $gas);

        pop_u256!($machine, op1, op2);
        let ret = $op(op1, op2);
        push_u256!($machine, ret);

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

        pop_u256!($machine, op1, op2, op3);
        let ret = $op(op1, op2, op3);
        push_u256!($machine, ret);

        Return::Continue
    }};
    ( $machine:expr, $op:path, $gas:expr, $spec:ident :: $enabled:ident) => {{
        check!($spec::$enabled);
        op3_u256_fn!($machine, $op, $gas)
    }};
}

macro_rules! as_usize_or_fail {
    ( $v:expr ) => {{
        if $v > U256::from(usize::MAX) {
            return Return::FatalNotSupported;
        }

        $v.as_usize()
    }};

    ( $v:expr, $reason:expr ) => {{
        if $v > U256::from(usize::MAX) {
            return $reason;
        }

        $v.as_usize()
    }};
}
