use super::i256::{i256_div, i256_mod};
use crate::{
    interpreter_types::{InterpreterTypes as ITy, StackTr},
    InstructionContext as Ictx, InstructionExecResult as Result,
};
use context_interface::Host;

/// Implements the ADD instruction - adds two values from stack.
pub fn add<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1], op2, context.interpreter);
    *op2 = op1.wrapping_add(*op2);
    Ok(())
}

/// Implements the MUL instruction - multiplies two values from stack.
pub fn mul<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1], op2, context.interpreter);
    *op2 = op1.wrapping_mul(*op2);
    Ok(())
}

/// Implements the SUB instruction - subtracts two values from stack.
pub fn sub<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1], op2, context.interpreter);
    *op2 = op1.wrapping_sub(*op2);
    Ok(())
}

/// Implements the DIV instruction - divides two values from stack.
pub fn div<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1], op2, context.interpreter);
    if !op2.is_zero() {
        *op2 = op1.wrapping_div(*op2);
    }
    Ok(())
}

/// Implements the SDIV instruction.
///
/// Performs signed division of two values from stack.
pub fn sdiv<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1], op2, context.interpreter);
    *op2 = i256_div(op1, *op2);
    Ok(())
}

/// Implements the MOD instruction.
///
/// Pops two values from stack and pushes the remainder of their division.
pub fn rem<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1], op2, context.interpreter);
    if !op2.is_zero() {
        *op2 = op1.wrapping_rem(*op2);
    }
    Ok(())
}

/// Implements the SMOD instruction.
///
/// Performs signed modulo of two values from stack.
pub fn smod<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1], op2, context.interpreter);
    *op2 = i256_mod(op1, *op2);
    Ok(())
}

/// Implements the ADDMOD instruction.
///
/// Pops three values from stack and pushes (a + b) % n.
pub fn addmod<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1, op2], op3, context.interpreter);
    *op3 = op1.add_mod(op2, *op3);
    Ok(())
}

/// Implements the MULMOD instruction.
///
/// Pops three values from stack and pushes (a * b) % n.
pub fn mulmod<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1, op2], op3, context.interpreter);
    *op3 = op1.mul_mod(op2, *op3);
    Ok(())
}

/// Implements the EXP instruction - exponentiates two values from stack.
pub fn exp<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([op1], op2, context.interpreter);
    gas!(
        context.interpreter,
        context.host.gas_params().exp_cost(*op2)
    );
    *op2 = op1.pow(*op2);
    Ok(())
}

/// Implements the `SIGNEXTEND` opcode as defined in the Ethereum Yellow Paper.
///
/// Sign-extends `x` from `(ext + 1)` bytes using arithmetic shift:
///   `shift = 248 - 8 * ext`
///   `result = (x << shift) >>s shift`
///
/// If `ext >= 31` the value already fills 32 bytes, so `x` is unchanged.
pub fn signextend<IT: ITy, H: ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    popn_top!([ext], x, context.interpreter);
    // For ext >= 31 the value already fills all 32 bytes; nothing to do.
    if ext < 31 {
        let shift = 248 - 8 * ext.as_limbs()[0] as usize;
        *x = (*x << shift).arithmetic_shr(shift);
    }
    Ok(())
}
