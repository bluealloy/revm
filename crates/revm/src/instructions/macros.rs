pub use crate::Return;
use primitive_types::U256;

macro_rules! gas {
    ($machine:expr, $gas:expr) => {
        if crate::USE_GAS {
            if !$machine.gas.record_cost(($gas)) {
                return Err(Return::OutOfGas);
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
                None => return Err(Return::OutOfGas),
            }
        }
    };
}

macro_rules! memory_resize {
    ($machine:expr, $offset:expr, $len:expr) => {{
        let len: usize = $len;
        let offset: usize = $offset;
        if let Some(new_size) =
            crate::machine::memory::next_multiple_of_32(offset.saturating_add(len))
        {
            if new_size > $machine.memory.len() {
                if crate::USE_GAS {
                    let num_bytes = new_size / 32;
                    if !$machine
                        .gas
                        .record_memory(crate::instructions::gas::memory_gas(num_bytes))
                    {
                        return Err(Return::OutOfGas);
                    }
                }
                $machine.memory.resize(new_size);
            }
        } else {
            return Err(Return::OutOfGas);
        }
    }};
}

macro_rules! op1_u256_fn {
    ( $machine:expr, $op:path ) => {{
        //gas!($machine, $gas);

        let op1 = $machine.stack.pop()?;
        let ret = $op(op1);
        $machine.stack.push_unchecked(ret);
        Ok(())
    }};
}

macro_rules! op2_u256_bool_ref {
    ( $machine:expr, $op:ident) => {{
        //gas!($machine, $gas);

        let (op1, op2) = $machine.stack.pop2()?;
        let ret = op1.$op(&op2);
        $machine
            .stack
            .push_unchecked(if ret { U256::one() } else { U256::zero() });
        Ok(())
    }};
}

macro_rules! op2_u256 {
    ( $machine:expr, $op:ident) => {{
        //gas!($machine, $gas);

        let (op1, op2) = $machine.stack.pop2()?;
        let ret = op1.$op(op2);
        $machine.stack.push_unchecked(ret);
        Ok(())
    }};
}

macro_rules! op2_u256_tuple {
    ( $machine:expr, $op:ident) => {{
        //gas!($machine, $gas);

        let (op1, op2) = $machine.stack.pop2()?;
        let (ret, ..) = op1.$op(op2);
        $machine.stack.push_unchecked(ret);
        Ok(())
    }};
}

macro_rules! op2_u256_fn {
    ( $machine:expr, $op:path ) => {{
        //gas!($machine, $gas);

        let (op1, op2) = $machine.stack.pop2()?;
        let ret = $op(op1, op2);
        $machine.stack.push_unchecked(ret);
        Ok(())
    }};
    ( $machine:expr, $op:path, $enabled:expr) => {{
        check!(($enabled));
        op2_u256_fn!($machine, $op)
    }};
}

macro_rules! op3_u256_fn {
    ( $machine:expr, $op:path) => {{
        //gas!($machine, $gas);

        let (op1, op2, op3) = $machine.stack.pop3()?;
        let ret = $op(op1, op2, op3);
        $machine.stack.push_unchecked(ret);
        Ok(())
    }};
    ( $machine:expr, $op:path, $spec:ident :: $enabled:ident) => {{
        check!($spec::$enabled);
        op3_u256_fn!($machine, $op)
    }};
}

macro_rules! as_usize_saturated {
    ( $v:expr ) => {{
        if { $v.0[1] != 0 || $v.0[2] != 0 || $v.0[3] != 0 } {
            usize::MAX
        } else {
            $v.0[0] as usize
        }
    }};
}

// XXX move
pub fn as_usize_or_fail(v: &U256, err: Return) -> Result<usize, Return> {
    if v.0[1] != 0 || v.0[2] != 0 || v.0[3] != 0 {
        return Err(err);
    }
    Ok(v.0[0] as usize)
}
