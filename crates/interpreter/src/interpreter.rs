//! Core interpreter implementation and components.

/// Extended bytecode functionality.
pub mod ext_bytecode;
mod input;
mod loop_control;
mod return_data;
mod runtime_flags;
mod shared_memory;
mod stack;

use context_interface::cfg::GasParams;
// re-exports
pub use ext_bytecode::ExtBytecode;
pub use input::InputsImpl;
pub use return_data::ReturnDataImpl;
pub use runtime_flags::RuntimeFlags;
pub use shared_memory::{num_words, resize_memory, SharedMemory};
pub use stack::{Stack, STACK_LIMIT};

// imports
use crate::{
    host::DummyHost,
    instruction_context::InstructionContext,
    instructions::{
        arithmetic, bitwise, block_info, contract, control, gas, host, memory, stack, system,
        tx_info,
    },
    interpreter_types::*,
    Gas, Host, InstructionResult, InstructionTable, InterpreterAction,
};
use bytecode::opcode::*;
use bytecode::Bytecode;
use primitives::{hardfork::SpecId, Bytes};

/// Main interpreter structure that contains all components defined in [`InterpreterTypes`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Interpreter<WIRE: InterpreterTypes = EthInterpreter> {
    /// Bytecode being executed.
    pub bytecode: WIRE::Bytecode,
    /// Gas tracking for execution costs.
    pub gas: Gas,
    /// EVM stack for computation.
    pub stack: WIRE::Stack,
    /// Buffer for return data from calls.
    pub return_data: WIRE::ReturnData,
    /// EVM memory for data storage.
    pub memory: WIRE::Memory,
    /// Input data for current execution context.
    pub input: WIRE::Input,
    /// Runtime flags controlling execution behavior.
    pub runtime_flag: WIRE::RuntimeFlag,
    /// Extended functionality and customizations.
    pub extend: WIRE::Extend,
}

impl<EXT: Default> Interpreter<EthInterpreter<EXT>> {
    /// Create new interpreter
    pub fn new(
        memory: SharedMemory,
        bytecode: ExtBytecode,
        input: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
    ) -> Self {
        Self::new_inner(
            Stack::new(),
            memory,
            bytecode,
            input,
            is_static,
            spec_id,
            gas_limit,
        )
    }

    /// Create a new interpreter with default extended functionality.
    pub fn default_ext() -> Self {
        Self::do_default(Stack::new(), SharedMemory::new())
    }

    /// Create a new invalid interpreter.
    pub fn invalid() -> Self {
        Self::do_default(Stack::invalid(), SharedMemory::invalid())
    }

    fn do_default(stack: Stack, memory: SharedMemory) -> Self {
        Self::new_inner(
            stack,
            memory,
            ExtBytecode::default(),
            InputsImpl::default(),
            false,
            SpecId::default(),
            u64::MAX,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_inner(
        stack: Stack,
        memory: SharedMemory,
        bytecode: ExtBytecode,
        input: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
    ) -> Self {
        Self {
            bytecode,
            gas: Gas::new(gas_limit),
            stack,
            return_data: Default::default(),
            memory,
            input,
            runtime_flag: RuntimeFlags { is_static, spec_id },
            extend: Default::default(),
        }
    }

    /// Clears and reinitializes the interpreter with new parameters.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn clear(
        &mut self,
        memory: SharedMemory,
        bytecode: ExtBytecode,
        input: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
    ) {
        let Self {
            bytecode: bytecode_ref,
            gas,
            stack,
            return_data,
            memory: memory_ref,
            input: input_ref,
            runtime_flag,
            extend,
        } = self;
        *bytecode_ref = bytecode;
        *gas = Gas::new(gas_limit);
        if stack.data().capacity() == 0 {
            *stack = Stack::new();
        } else {
            stack.clear();
        }
        return_data.0.clear();
        *memory_ref = memory;
        *input_ref = input;
        *runtime_flag = RuntimeFlags { spec_id, is_static };
        *extend = EXT::default();
    }

    /// Sets the bytecode that is going to be executed
    pub fn with_bytecode(mut self, bytecode: Bytecode) -> Self {
        self.bytecode = ExtBytecode::new(bytecode);
        self
    }
}

impl Default for Interpreter<EthInterpreter> {
    fn default() -> Self {
        Self::default_ext()
    }
}

/// Default types for Ethereum interpreter.
#[derive(Debug)]
pub struct EthInterpreter<EXT = (), MG = SharedMemory> {
    _phantom: core::marker::PhantomData<fn() -> (EXT, MG)>,
}

impl<EXT> InterpreterTypes for EthInterpreter<EXT> {
    type Stack = Stack;
    type Memory = SharedMemory;
    type Bytecode = ExtBytecode;
    type ReturnData = ReturnDataImpl;
    type Input = InputsImpl;
    type RuntimeFlag = RuntimeFlags;
    type Extend = EXT;
    type Output = InterpreterAction;
}

impl<IW: InterpreterTypes> Interpreter<IW> {
    /// Performs EVM memory resize.
    #[inline]
    #[must_use]
    pub fn resize_memory(&mut self, gas_params: &GasParams, offset: usize, len: usize) -> bool {
        if let Err(result) = resize_memory(&mut self.gas, &mut self.memory, gas_params, offset, len)
        {
            self.halt(result);
            return false;
        }
        true
    }

    /// Takes the next action from the control and returns it.
    #[inline]
    pub fn take_next_action(&mut self) -> InterpreterAction {
        self.bytecode.reset_action();
        // Return next action if it is some.
        let action = core::mem::take(self.bytecode.action()).expect("Interpreter to set action");
        action
    }

    /// Halt the interpreter with the given result.
    ///
    /// This will set the action to [`InterpreterAction::Return`] and set the gas to the current gas.
    #[cold]
    #[inline(never)]
    pub fn halt(&mut self, result: InstructionResult) {
        self.bytecode
            .set_action(InterpreterAction::new_halt(result, self.gas));
    }

    /// Halt the interpreter with the given result.
    ///
    /// This will set the action to [`InterpreterAction::Return`] and set the gas to the current gas.
    #[cold]
    #[inline(never)]
    pub fn halt_fatal(&mut self) {
        self.bytecode.set_action(InterpreterAction::new_halt(
            InstructionResult::FatalExternalError,
            self.gas,
        ));
    }

    /// Halt the interpreter with an out-of-gas error.
    #[cold]
    #[inline(never)]
    pub fn halt_oog(&mut self) {
        self.gas.spend_all();
        self.halt(InstructionResult::OutOfGas);
    }

    /// Halt the interpreter with an out-of-gas error.
    #[cold]
    #[inline(never)]
    pub fn halt_memory_oog(&mut self) {
        self.halt(InstructionResult::MemoryOOG);
    }

    /// Halt the interpreter with an out-of-gas error.
    #[cold]
    #[inline(never)]
    pub fn halt_memory_limit_oog(&mut self) {
        self.halt(InstructionResult::MemoryLimitOOG);
    }

    /// Halt the interpreter with and overflow error.
    #[cold]
    #[inline(never)]
    pub fn halt_overflow(&mut self) {
        self.halt(InstructionResult::StackOverflow);
    }

    /// Halt the interpreter with and underflow error.
    #[cold]
    #[inline(never)]
    pub fn halt_underflow(&mut self) {
        self.halt(InstructionResult::StackUnderflow);
    }

    /// Halt the interpreter with and not activated error.
    #[cold]
    #[inline(never)]
    pub fn halt_not_activated(&mut self) {
        self.halt(InstructionResult::NotActivated);
    }

    /// Return with the given output.
    ///
    /// This will set the action to [`InterpreterAction::Return`] and set the gas to the current gas.
    pub fn return_with_output(&mut self, output: Bytes) {
        self.bytecode.set_action(InterpreterAction::new_return(
            InstructionResult::Return,
            output,
            self.gas,
        ));
    }

    /// Executes the instruction at the current instruction pointer using match-based dispatch.
    ///
    /// This uses a match statement for better branch prediction compared to
    /// indirect function pointer calls.
    ///
    /// Internally it will increment instruction pointer by one.
    #[inline]
    pub fn step<H: Host + ?Sized>(
        &mut self,
        _instruction_table: &InstructionTable<IW, H>,
        host: &mut H,
    ) {
        // Get current opcode.
        let opcode = self.bytecode.opcode();

        // SAFETY: In analysis we are doing padding of bytecode so that we are sure that last
        // byte instruction is STOP so we are safe to just increment program_counter bcs on last instruction
        // it will do noop and just stop execution of this contract
        self.bytecode.relative_jump(1);

        let gas_cost = static_gas(opcode);
        if gas_cost != 0 && self.gas.record_cost_unsafe(gas_cost) {
            return self.halt_oog();
        }

        execute_instruction(self, host, opcode);
    }

    /// Executes the instruction at the current instruction pointer.
    ///
    /// Internally it will increment instruction pointer by one.
    ///
    /// This uses dummy Host.
    #[inline]
    pub fn step_dummy(&mut self, instruction_table: &InstructionTable<IW, DummyHost>) {
        self.step(instruction_table, &mut DummyHost::default());
    }

    /// Executes the interpreter until it returns or stops.
    #[inline]
    pub fn run_plain<H: Host + ?Sized>(
        &mut self,
        instruction_table: &InstructionTable<IW, H>,
        host: &mut H,
    ) -> InterpreterAction {
        while self.bytecode.is_not_end() {
            self.step(instruction_table, host);
        }
        self.take_next_action()
    }
}

/// Returns the static gas cost for an opcode.
///
/// This is a const function that allows the compiler to inline and optimize
/// gas cost lookups at compile time where possible.
#[inline]
const fn static_gas(opcode: u8) -> u64 {
    match opcode {
        STOP => 0,
        ADD => 3,
        MUL => 5,
        SUB => 3,
        DIV => 5,
        SDIV => 5,
        MOD => 5,
        SMOD => 5,
        ADDMOD => 8,
        MULMOD => 8,
        EXP => gas::EXP,
        SIGNEXTEND => 5,

        LT => 3,
        GT => 3,
        SLT => 3,
        SGT => 3,
        EQ => 3,
        ISZERO => 3,
        AND => 3,
        OR => 3,
        XOR => 3,
        NOT => 3,
        BYTE => 3,
        SHL => 3,
        SHR => 3,
        SAR => 3,
        CLZ => 5,

        KECCAK256 => gas::KECCAK256,

        ADDRESS => 2,
        BALANCE => 20,
        ORIGIN => 2,
        CALLER => 2,
        CALLVALUE => 2,
        CALLDATALOAD => 3,
        CALLDATASIZE => 2,
        CALLDATACOPY => 3,
        CODESIZE => 2,
        CODECOPY => 3,

        GASPRICE => 2,
        EXTCODESIZE => 20,
        EXTCODECOPY => 20,
        RETURNDATASIZE => 2,
        RETURNDATACOPY => 3,
        EXTCODEHASH => 400,
        BLOCKHASH => 20,
        COINBASE => 2,
        TIMESTAMP => 2,
        NUMBER => 2,
        DIFFICULTY => 2,
        GASLIMIT => 2,
        CHAINID => 2,
        SELFBALANCE => 5,
        BASEFEE => 2,
        BLOBHASH => 3,
        BLOBBASEFEE => 2,
        SLOTNUM => 2,

        POP => 2,
        MLOAD => 3,
        MSTORE => 3,
        MSTORE8 => 3,
        SLOAD => 50,
        SSTORE => 0,
        JUMP => 8,
        JUMPI => 10,
        PC => 2,
        MSIZE => 2,
        GAS => 2,
        JUMPDEST => 1,
        TLOAD => 100,
        TSTORE => 100,
        MCOPY => 3,

        PUSH0 => 2,
        PUSH1..=PUSH32 => 3,

        DUP1..=DUP16 => 3,
        SWAP1..=SWAP16 => 3,

        DUPN => 3,
        SWAPN => 3,
        EXCHANGE => 3,

        LOG0..=LOG4 => gas::LOG,

        CREATE => 0,
        CALL => 40,
        CALLCODE => 40,
        RETURN => 0,
        DELEGATECALL => 40,
        CREATE2 => 0,
        STATICCALL => 40,
        REVERT => 0,
        INVALID => 0,
        SELFDESTRUCT => 0,

        _ => 0,
    }
}

/// Execute an instruction using match-based dispatch.
///
/// This provides better branch prediction than function pointer tables
/// by allowing the compiler to generate a jump table with statically known targets.
#[inline]
fn execute_instruction<IW: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<IW>,
    host: &mut H,
    opcode: u8,
) {
    match opcode {
        STOP => control::stop(InstructionContext { interpreter, host }),
        ADD => arithmetic::add(InstructionContext { interpreter, host }),
        MUL => arithmetic::mul(InstructionContext { interpreter, host }),
        SUB => arithmetic::sub(InstructionContext { interpreter, host }),
        DIV => arithmetic::div(InstructionContext { interpreter, host }),
        SDIV => arithmetic::sdiv(InstructionContext { interpreter, host }),
        MOD => arithmetic::rem(InstructionContext { interpreter, host }),
        SMOD => arithmetic::smod(InstructionContext { interpreter, host }),
        ADDMOD => arithmetic::addmod(InstructionContext { interpreter, host }),
        MULMOD => arithmetic::mulmod(InstructionContext { interpreter, host }),
        EXP => arithmetic::exp(InstructionContext { interpreter, host }),
        SIGNEXTEND => arithmetic::signextend(InstructionContext { interpreter, host }),

        LT => bitwise::lt(InstructionContext { interpreter, host }),
        GT => bitwise::gt(InstructionContext { interpreter, host }),
        SLT => bitwise::slt(InstructionContext { interpreter, host }),
        SGT => bitwise::sgt(InstructionContext { interpreter, host }),
        EQ => bitwise::eq(InstructionContext { interpreter, host }),
        ISZERO => bitwise::iszero(InstructionContext { interpreter, host }),
        AND => bitwise::bitand(InstructionContext { interpreter, host }),
        OR => bitwise::bitor(InstructionContext { interpreter, host }),
        XOR => bitwise::bitxor(InstructionContext { interpreter, host }),
        NOT => bitwise::not(InstructionContext { interpreter, host }),
        BYTE => bitwise::byte(InstructionContext { interpreter, host }),
        SHL => bitwise::shl(InstructionContext { interpreter, host }),
        SHR => bitwise::shr(InstructionContext { interpreter, host }),
        SAR => bitwise::sar(InstructionContext { interpreter, host }),
        CLZ => bitwise::clz(InstructionContext { interpreter, host }),

        KECCAK256 => system::keccak256(InstructionContext { interpreter, host }),

        ADDRESS => system::address(InstructionContext { interpreter, host }),
        BALANCE => host::balance(InstructionContext { interpreter, host }),
        ORIGIN => tx_info::origin(InstructionContext { interpreter, host }),
        CALLER => system::caller(InstructionContext { interpreter, host }),
        CALLVALUE => system::callvalue(InstructionContext { interpreter, host }),
        CALLDATALOAD => system::calldataload(InstructionContext { interpreter, host }),
        CALLDATASIZE => system::calldatasize(InstructionContext { interpreter, host }),
        CALLDATACOPY => system::calldatacopy(InstructionContext { interpreter, host }),
        CODESIZE => system::codesize(InstructionContext { interpreter, host }),
        CODECOPY => system::codecopy(InstructionContext { interpreter, host }),

        GASPRICE => tx_info::gasprice(InstructionContext { interpreter, host }),
        EXTCODESIZE => host::extcodesize(InstructionContext { interpreter, host }),
        EXTCODECOPY => host::extcodecopy(InstructionContext { interpreter, host }),
        RETURNDATASIZE => system::returndatasize(InstructionContext { interpreter, host }),
        RETURNDATACOPY => system::returndatacopy(InstructionContext { interpreter, host }),
        EXTCODEHASH => host::extcodehash(InstructionContext { interpreter, host }),
        BLOCKHASH => host::blockhash(InstructionContext { interpreter, host }),
        COINBASE => block_info::coinbase(InstructionContext { interpreter, host }),
        TIMESTAMP => block_info::timestamp(InstructionContext { interpreter, host }),
        NUMBER => block_info::block_number(InstructionContext { interpreter, host }),
        DIFFICULTY => block_info::difficulty(InstructionContext { interpreter, host }),
        GASLIMIT => block_info::gaslimit(InstructionContext { interpreter, host }),
        CHAINID => block_info::chainid(InstructionContext { interpreter, host }),
        SELFBALANCE => host::selfbalance(InstructionContext { interpreter, host }),
        BASEFEE => block_info::basefee(InstructionContext { interpreter, host }),
        BLOBHASH => tx_info::blob_hash(InstructionContext { interpreter, host }),
        BLOBBASEFEE => block_info::blob_basefee(InstructionContext { interpreter, host }),
        SLOTNUM => block_info::slot_num(InstructionContext { interpreter, host }),

        POP => stack::pop(InstructionContext { interpreter, host }),
        MLOAD => memory::mload(InstructionContext { interpreter, host }),
        MSTORE => memory::mstore(InstructionContext { interpreter, host }),
        MSTORE8 => memory::mstore8(InstructionContext { interpreter, host }),
        SLOAD => host::sload(InstructionContext { interpreter, host }),
        SSTORE => host::sstore(InstructionContext { interpreter, host }),
        JUMP => control::jump(InstructionContext { interpreter, host }),
        JUMPI => control::jumpi(InstructionContext { interpreter, host }),
        PC => control::pc(InstructionContext { interpreter, host }),
        MSIZE => memory::msize(InstructionContext { interpreter, host }),
        GAS => system::gas(InstructionContext { interpreter, host }),
        JUMPDEST => control::jumpdest(InstructionContext { interpreter, host }),
        TLOAD => host::tload(InstructionContext { interpreter, host }),
        TSTORE => host::tstore(InstructionContext { interpreter, host }),
        MCOPY => memory::mcopy(InstructionContext { interpreter, host }),

        PUSH0 => stack::push0(InstructionContext { interpreter, host }),
        PUSH1 => stack::push::<1, _, _>(InstructionContext { interpreter, host }),
        PUSH2 => stack::push::<2, _, _>(InstructionContext { interpreter, host }),
        PUSH3 => stack::push::<3, _, _>(InstructionContext { interpreter, host }),
        PUSH4 => stack::push::<4, _, _>(InstructionContext { interpreter, host }),
        PUSH5 => stack::push::<5, _, _>(InstructionContext { interpreter, host }),
        PUSH6 => stack::push::<6, _, _>(InstructionContext { interpreter, host }),
        PUSH7 => stack::push::<7, _, _>(InstructionContext { interpreter, host }),
        PUSH8 => stack::push::<8, _, _>(InstructionContext { interpreter, host }),
        PUSH9 => stack::push::<9, _, _>(InstructionContext { interpreter, host }),
        PUSH10 => stack::push::<10, _, _>(InstructionContext { interpreter, host }),
        PUSH11 => stack::push::<11, _, _>(InstructionContext { interpreter, host }),
        PUSH12 => stack::push::<12, _, _>(InstructionContext { interpreter, host }),
        PUSH13 => stack::push::<13, _, _>(InstructionContext { interpreter, host }),
        PUSH14 => stack::push::<14, _, _>(InstructionContext { interpreter, host }),
        PUSH15 => stack::push::<15, _, _>(InstructionContext { interpreter, host }),
        PUSH16 => stack::push::<16, _, _>(InstructionContext { interpreter, host }),
        PUSH17 => stack::push::<17, _, _>(InstructionContext { interpreter, host }),
        PUSH18 => stack::push::<18, _, _>(InstructionContext { interpreter, host }),
        PUSH19 => stack::push::<19, _, _>(InstructionContext { interpreter, host }),
        PUSH20 => stack::push::<20, _, _>(InstructionContext { interpreter, host }),
        PUSH21 => stack::push::<21, _, _>(InstructionContext { interpreter, host }),
        PUSH22 => stack::push::<22, _, _>(InstructionContext { interpreter, host }),
        PUSH23 => stack::push::<23, _, _>(InstructionContext { interpreter, host }),
        PUSH24 => stack::push::<24, _, _>(InstructionContext { interpreter, host }),
        PUSH25 => stack::push::<25, _, _>(InstructionContext { interpreter, host }),
        PUSH26 => stack::push::<26, _, _>(InstructionContext { interpreter, host }),
        PUSH27 => stack::push::<27, _, _>(InstructionContext { interpreter, host }),
        PUSH28 => stack::push::<28, _, _>(InstructionContext { interpreter, host }),
        PUSH29 => stack::push::<29, _, _>(InstructionContext { interpreter, host }),
        PUSH30 => stack::push::<30, _, _>(InstructionContext { interpreter, host }),
        PUSH31 => stack::push::<31, _, _>(InstructionContext { interpreter, host }),
        PUSH32 => stack::push::<32, _, _>(InstructionContext { interpreter, host }),

        DUP1 => stack::dup::<1, _, _>(InstructionContext { interpreter, host }),
        DUP2 => stack::dup::<2, _, _>(InstructionContext { interpreter, host }),
        DUP3 => stack::dup::<3, _, _>(InstructionContext { interpreter, host }),
        DUP4 => stack::dup::<4, _, _>(InstructionContext { interpreter, host }),
        DUP5 => stack::dup::<5, _, _>(InstructionContext { interpreter, host }),
        DUP6 => stack::dup::<6, _, _>(InstructionContext { interpreter, host }),
        DUP7 => stack::dup::<7, _, _>(InstructionContext { interpreter, host }),
        DUP8 => stack::dup::<8, _, _>(InstructionContext { interpreter, host }),
        DUP9 => stack::dup::<9, _, _>(InstructionContext { interpreter, host }),
        DUP10 => stack::dup::<10, _, _>(InstructionContext { interpreter, host }),
        DUP11 => stack::dup::<11, _, _>(InstructionContext { interpreter, host }),
        DUP12 => stack::dup::<12, _, _>(InstructionContext { interpreter, host }),
        DUP13 => stack::dup::<13, _, _>(InstructionContext { interpreter, host }),
        DUP14 => stack::dup::<14, _, _>(InstructionContext { interpreter, host }),
        DUP15 => stack::dup::<15, _, _>(InstructionContext { interpreter, host }),
        DUP16 => stack::dup::<16, _, _>(InstructionContext { interpreter, host }),

        SWAP1 => stack::swap::<1, _, _>(InstructionContext { interpreter, host }),
        SWAP2 => stack::swap::<2, _, _>(InstructionContext { interpreter, host }),
        SWAP3 => stack::swap::<3, _, _>(InstructionContext { interpreter, host }),
        SWAP4 => stack::swap::<4, _, _>(InstructionContext { interpreter, host }),
        SWAP5 => stack::swap::<5, _, _>(InstructionContext { interpreter, host }),
        SWAP6 => stack::swap::<6, _, _>(InstructionContext { interpreter, host }),
        SWAP7 => stack::swap::<7, _, _>(InstructionContext { interpreter, host }),
        SWAP8 => stack::swap::<8, _, _>(InstructionContext { interpreter, host }),
        SWAP9 => stack::swap::<9, _, _>(InstructionContext { interpreter, host }),
        SWAP10 => stack::swap::<10, _, _>(InstructionContext { interpreter, host }),
        SWAP11 => stack::swap::<11, _, _>(InstructionContext { interpreter, host }),
        SWAP12 => stack::swap::<12, _, _>(InstructionContext { interpreter, host }),
        SWAP13 => stack::swap::<13, _, _>(InstructionContext { interpreter, host }),
        SWAP14 => stack::swap::<14, _, _>(InstructionContext { interpreter, host }),
        SWAP15 => stack::swap::<15, _, _>(InstructionContext { interpreter, host }),
        SWAP16 => stack::swap::<16, _, _>(InstructionContext { interpreter, host }),

        DUPN => stack::dupn(InstructionContext { interpreter, host }),
        SWAPN => stack::swapn(InstructionContext { interpreter, host }),
        EXCHANGE => stack::exchange(InstructionContext { interpreter, host }),

        LOG0 => host::log::<0, _>(InstructionContext { interpreter, host }),
        LOG1 => host::log::<1, _>(InstructionContext { interpreter, host }),
        LOG2 => host::log::<2, _>(InstructionContext { interpreter, host }),
        LOG3 => host::log::<3, _>(InstructionContext { interpreter, host }),
        LOG4 => host::log::<4, _>(InstructionContext { interpreter, host }),

        CREATE => contract::create::<_, false, _>(InstructionContext { interpreter, host }),
        CALL => contract::call(InstructionContext { interpreter, host }),
        CALLCODE => contract::call_code(InstructionContext { interpreter, host }),
        RETURN => control::ret(InstructionContext { interpreter, host }),
        DELEGATECALL => contract::delegate_call(InstructionContext { interpreter, host }),
        CREATE2 => contract::create::<_, true, _>(InstructionContext { interpreter, host }),
        STATICCALL => contract::static_call(InstructionContext { interpreter, host }),
        REVERT => control::revert(InstructionContext { interpreter, host }),
        INVALID => control::invalid(InstructionContext { interpreter, host }),
        SELFDESTRUCT => host::selfdestruct(InstructionContext { interpreter, host }),

        _ => control::unknown(InstructionContext { interpreter, host }),
    }
}

/* used for cargo asm
pub fn asm_step(
    interpreter: &mut Interpreter<EthInterpreter>,
    instruction_table: &InstructionTable<EthInterpreter, DummyHost>,
    host: &mut DummyHost,
) {
    interpreter.step(instruction_table, host);
}

pub fn asm_run(
    interpreter: &mut Interpreter<EthInterpreter>,
    instruction_table: &InstructionTable<EthInterpreter, DummyHost>,
    host: &mut DummyHost,
) {
    interpreter.run_plain(instruction_table, host);
}
*/

/// The result of an interpreter operation.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct InterpreterResult {
    /// The result of the instruction execution.
    pub result: InstructionResult,
    /// The output of the instruction execution.
    pub output: Bytes,
    /// The gas usage information.
    pub gas: Gas,
}

impl InterpreterResult {
    /// Returns a new `InterpreterResult` with the given values.
    pub fn new(result: InstructionResult, output: Bytes, gas: Gas) -> Self {
        Self {
            result,
            output,
            gas,
        }
    }

    /// Returns a new `InterpreterResult` for an out-of-gas error with the given gas limit.
    pub fn new_oog(gas_limit: u64) -> Self {
        Self {
            result: InstructionResult::OutOfGas,
            output: Bytes::default(),
            gas: Gas::new_spent(gas_limit),
        }
    }

    /// Returns whether the instruction result is a success.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        self.result.is_ok()
    }

    /// Returns whether the instruction result is a revert.
    #[inline]
    pub const fn is_revert(&self) -> bool {
        self.result.is_revert()
    }

    /// Returns whether the instruction result is an error.
    #[inline]
    pub const fn is_error(&self) -> bool {
        self.result.is_error()
    }
}

// Special implementation for types where Output can be created from InterpreterAction
impl<IW: InterpreterTypes> Interpreter<IW>
where
    IW::Output: From<InterpreterAction>,
{
    /// Takes the next action from the control and returns it as the specific Output type.
    #[inline]
    pub fn take_next_action_as_output(&mut self) -> IW::Output {
        From::from(self.take_next_action())
    }

    /// Executes the interpreter until it returns or stops, returning the specific Output type.
    #[inline]
    pub fn run_plain_as_output<H: Host + ?Sized>(
        &mut self,
        instruction_table: &InstructionTable<IW, H>,
        host: &mut H,
    ) -> IW::Output {
        From::from(self.run_plain(instruction_table, host))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "serde")]
    fn test_interpreter_serde() {
        use super::*;
        use bytecode::Bytecode;
        use primitives::Bytes;

        let bytecode = Bytecode::new_raw(Bytes::from(&[0x60, 0x00, 0x60, 0x00, 0x01][..]));
        let interpreter = Interpreter::<EthInterpreter>::new(
            SharedMemory::new(),
            ExtBytecode::new(bytecode),
            InputsImpl::default(),
            false,
            SpecId::default(),
            u64::MAX,
        );

        let serialized = serde_json::to_string_pretty(&interpreter).unwrap();
        let deserialized: Interpreter<EthInterpreter> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(
            interpreter.bytecode.pc(),
            deserialized.bytecode.pc(),
            "Program counter should be preserved"
        );
    }
}

#[test]
fn test_mstore_big_offset_memory_oog() {
    use super::*;
    use crate::{host::DummyHost, instructions::instruction_table};
    use bytecode::Bytecode;
    use primitives::Bytes;

    let code = Bytes::from(
        &[
            0x60, 0x00, // PUSH1 0x00
            0x61, 0x27, 0x10, // PUSH2 0x2710  (10,000)
            0x52, // MSTORE
            0x00, // STOP
        ][..],
    );
    let bytecode = Bytecode::new_raw(code);

    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new(),
        ExtBytecode::new(bytecode),
        InputsImpl::default(),
        false,
        SpecId::default(),
        1000,
    );

    let table = instruction_table::<EthInterpreter, DummyHost>();
    let mut host = DummyHost::default();
    let action = interpreter.run_plain(&table, &mut host);

    assert!(action.is_return());
    assert_eq!(
        action.instruction_result(),
        Some(InstructionResult::MemoryOOG)
    );
}

#[test]
#[cfg(feature = "memory_limit")]
fn test_mstore_big_offset_memory_limit_oog() {
    use super::*;
    use crate::{host::DummyHost, instructions::instruction_table};
    use bytecode::Bytecode;
    use primitives::Bytes;

    let code = Bytes::from(
        &[
            0x60, 0x00, // PUSH1 0x00
            0x61, 0x27, 0x10, // PUSH2 0x2710  (10,000)
            0x52, // MSTORE
            0x00, // STOP
        ][..],
    );
    let bytecode = Bytecode::new_raw(code);

    let mut interpreter = Interpreter::<EthInterpreter>::new(
        SharedMemory::new_with_memory_limit(1000),
        ExtBytecode::new(bytecode),
        InputsImpl::default(),
        false,
        SpecId::default(),
        100000,
    );

    let table = instruction_table::<EthInterpreter, DummyHost>();
    let mut host = DummyHost::default();
    let action = interpreter.run_plain(&table, &mut host);

    assert!(action.is_return());
    assert_eq!(
        action.instruction_result(),
        Some(InstructionResult::MemoryLimitOOG)
    );
}
