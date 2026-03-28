use crate::{
    evm::FrameTr, item_or_result::FrameInitOrResult, precompile_provider::PrecompileProvider,
    CallFrame, CreateFrame, FrameData, FrameResult, ItemOrResult,
};
use context::result::FromStringError;
use context_interface::{
    context::{take_error, ContextError},
    journaled_state::{account::JournaledAccountTr, JournalCheckpoint, JournalTr},
    local::{FrameToken, OutFrame},
    Cfg, ContextTr, Database,
};
use core::cmp::min;
use derive_where::derive_where;
use interpreter::{
    interpreter::{EthInterpreter, ExtBytecode},
    interpreter_action::FrameInit,
    interpreter_types::ReturnData,
    CallInput, CallInputs, CallOutcome, CallValue, CreateInputs, CreateOutcome, CreateScheme,
    FrameInput, Gas, InputsImpl, InstructionResult, Interpreter, InterpreterAction,
    InterpreterResult, InterpreterTypes, SharedMemory,
};
use primitives::{
    constants::CALL_STACK_LIMIT,
    hardfork::SpecId::{self, HOMESTEAD, LONDON, SPURIOUS_DRAGON},
    keccak256, Address, Bytes, B256, U256,
};
use state::Bytecode;
use std::{borrow::ToOwned, boxed::Box, string::String, vec::Vec};

// ────────────────────────────────────────────────────────────────────────────────
// Standalone step functions
// ────────────────────────────────────────────────────────────────────────────────

/// Returns `Err(CallTooDeep)` if `depth > CALL_STACK_LIMIT`.
#[inline]
pub fn check_depth(depth: usize) -> Result<(), InstructionResult> {
    if depth > CALL_STACK_LIMIT as usize {
        Err(InstructionResult::CallTooDeep)
    } else {
        Ok(())
    }
}

/// Transfer value from caller to target. Reverts checkpoint on failure.
///
/// Also touches the target account for EIP-158 state clear.
#[inline]
pub fn call_transfer_value<J: JournalTr>(
    journal: &mut J,
    caller: Address,
    target: Address,
    value: &CallValue,
    checkpoint: JournalCheckpoint,
) -> Result<(), InstructionResult> {
    if let CallValue::Transfer(value) = *value {
        if let Some(e) = journal.transfer_loaded(caller, target, value) {
            journal.checkpoint_revert(checkpoint);
            return Err(e.into());
        }
    }
    Ok(())
}

/// Build [`InputsImpl`] from [`CallInputs`].
#[inline]
pub fn call_interpreter_input(inputs: &CallInputs) -> InputsImpl {
    InputsImpl {
        target_address: inputs.target_address,
        caller_address: inputs.caller,
        bytecode_address: Some(inputs.bytecode_address),
        input: inputs.input.clone(),
        call_value: inputs.value.get(),
    }
}

/// Load bytecode from account, or use `known_bytecode` if provided.
///
/// Returns `(Bytecode, B256)` — the bytecode and its hash.
#[inline]
pub fn call_load_bytecode<J: JournalTr>(
    journal: &mut J,
    bytecode_address: Address,
    known_bytecode: Option<(B256, Bytecode)>,
) -> Result<(Bytecode, B256), <J::Database as Database>::Error> {
    if let Some((hash, code)) = known_bytecode {
        Ok((code, hash))
    } else {
        let account = journal.load_account_with_code(bytecode_address)?;
        Ok((
            account.info.code.clone().unwrap_or_default(),
            account.info.code_hash,
        ))
    }
}

/// Check that the caller has sufficient balance for the create value transfer.
///
/// This is a standalone step function for composing custom create flows.
/// The [`FrameBuilder`] inlines this logic to avoid redundant account loads.
#[inline]
pub fn create_check_balance<J: JournalTr>(
    journal: &mut J,
    caller: Address,
    value: U256,
) -> Result<(), CreateCheckError<<J::Database as Database>::Error>> {
    let caller_info = journal.load_account_mut(caller)?;
    if *caller_info.balance() < value {
        return Err(CreateCheckError::Instruction(InstructionResult::OutOfFunds));
    }
    Ok(())
}

/// Bump caller nonce. Returns the old nonce.
///
/// Errors on nonce overflow.
///
/// This is a standalone step function for composing custom create flows.
/// The [`FrameBuilder`] inlines this logic to avoid redundant account loads.
#[inline]
pub fn create_bump_nonce<J: JournalTr>(
    journal: &mut J,
    caller: Address,
) -> Result<u64, CreateCheckError<<J::Database as Database>::Error>> {
    let mut caller_info = journal.load_account_mut(caller)?;
    let old_nonce = caller_info.nonce();
    if !caller_info.bump_nonce() {
        return Err(CreateCheckError::Instruction(InstructionResult::Return));
    }
    Ok(old_nonce)
}

/// Error type for create check/nonce operations that can fail
/// with either a database error or an instruction result.
#[derive(Debug)]
pub enum CreateCheckError<DbError> {
    /// Database error.
    Db(DbError),
    /// Instruction-level failure (e.g. OutOfFunds, nonce overflow).
    Instruction(InstructionResult),
}

impl<DbError> From<DbError> for CreateCheckError<DbError> {
    fn from(e: DbError) -> Self {
        Self::Db(e)
    }
}

/// Compute the created address from the scheme, caller, nonce, and init_code.
///
/// Returns `(address, Option<init_code_hash>)` — the hash is present for CREATE2.
#[inline]
pub fn create_compute_address(
    caller: Address,
    old_nonce: u64,
    scheme: &CreateScheme,
    init_code: &Bytes,
) -> (Address, Option<B256>) {
    match scheme {
        CreateScheme::Create => (caller.create(old_nonce), None),
        CreateScheme::Create2 { salt } => {
            let hash = keccak256(init_code);
            (caller.create2(salt.to_be_bytes(), hash), Some(hash))
        }
        CreateScheme::Custom { address } => (*address, None),
    }
}

/// Build [`InputsImpl`] for a create frame.
#[inline]
pub fn create_interpreter_input(
    created_address: Address,
    caller: Address,
    value: U256,
) -> InputsImpl {
    InputsImpl {
        target_address: created_address,
        caller_address: caller,
        bytecode_address: None,
        input: CallInput::Bytes(Bytes::new()),
        call_value: value,
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// Kind marker types
// ────────────────────────────────────────────────────────────────────────────────

/// Call frame builder configuration.
#[derive(Debug)]
pub struct CallKind {
    inputs: Box<CallInputs>,
    transfer_value: bool,
    check_precompiles: bool,
    check_empty_bytecode: bool,
    bytecode: Option<Box<(Bytecode, B256)>>,
    is_static: Option<bool>,
}

/// Create frame builder configuration.
#[derive(Debug)]
pub struct CreateKind {
    inputs: Box<CreateInputs>,
    check_balance: bool,
    bump_nonce: bool,
    created_address: Option<Address>,
    bytecode: Option<Box<ExtBytecode>>,
}

// ────────────────────────────────────────────────────────────────────────────────
// Build implementations
// ────────────────────────────────────────────────────────────────────────────────

impl FrameBuilder<CallKind> {
    /// Consume the builder and produce a call frame (or an early result).
    #[inline]
    pub fn build<CTX, ERROR>(
        self,
        mut this: OutFrame<'_, EthFrame>,
        ctx: &mut CTX,
        mut precompile_fn: impl FnMut(
            &mut CTX,
            &CallInputs,
        ) -> Result<Option<InterpreterResult>, String>,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR>
    where
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    {
        let FrameBuilder {
            depth,
            memory,
            check_depth: do_check_depth,
            overrides,
            kind:
                CallKind {
                    inputs,
                    transfer_value,
                    check_precompiles,
                    check_empty_bytecode,
                    bytecode: override_bytecode,
                    is_static: override_is_static,
                },
        } = self;

        let (override_checkpoint, override_input, override_gas_limit) = match overrides {
            Some(o) => (o.checkpoint, o.interpreter_input, o.gas_limit),
            None => (None, None, None),
        };

        let gas = Gas::new(override_gas_limit.unwrap_or(inputs.gas_limit));
        let return_result = |instruction_result: InstructionResult| {
            Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome {
                result: InterpreterResult {
                    result: instruction_result,
                    gas,
                    output: Bytes::new(),
                },
                memory_offset: inputs.return_memory_offset.clone(),
                was_precompile_called: false,
                precompile_call_logs: Vec::new(),
            })))
        };

        // Check depth
        if do_check_depth {
            if let Err(e) = check_depth(depth) {
                return return_result(e);
            }
        }

        // Derive interpreter input and other fields before any moves.
        let interpreter_input = override_input.unwrap_or_else(|| call_interpreter_input(&inputs));
        let is_static = override_is_static.unwrap_or(inputs.is_static);
        let gas_limit = override_gas_limit.unwrap_or(inputs.gas_limit);

        // Create subroutine checkpoint
        let checkpoint = override_checkpoint.unwrap_or_else(|| ctx.journal_mut().checkpoint());

        // Touch address. For "EIP-158 State Clear", this will erase empty accounts.
        if transfer_value {
            if let Err(e) = call_transfer_value(
                ctx.journal_mut(),
                inputs.caller,
                inputs.target_address,
                &inputs.value,
                checkpoint,
            ) {
                return return_result(e);
            }
        }

        // Check precompiles
        if check_precompiles {
            if let Some(result) = precompile_fn(ctx, &inputs).map_err(ERROR::from_string)? {
                let mut logs = Vec::new();
                if result.result.is_ok() {
                    ctx.journal_mut().checkpoint_commit();
                } else {
                    logs = ctx.journal_mut().logs()[checkpoint.log_i..].to_vec();
                    ctx.journal_mut().checkpoint_revert(checkpoint);
                }
                return Ok(ItemOrResult::Result(FrameResult::Call(CallOutcome {
                    result,
                    memory_offset: inputs.return_memory_offset.clone(),
                    was_precompile_called: true,
                    precompile_call_logs: logs,
                })));
            }
        }

        // Get bytecode and hash
        let (bytecode, bytecode_hash) = if let Some(bch) = override_bytecode {
            *bch
        } else {
            call_load_bytecode(
                ctx.journal_mut(),
                inputs.bytecode_address,
                inputs.known_bytecode.clone(),
            )?
        };

        // Returns success if bytecode is empty.
        if check_empty_bytecode && bytecode.is_empty() {
            ctx.journal_mut().checkpoint_commit();
            return return_result(InstructionResult::Stop);
        }

        // Create interpreter and push new CallStackFrame.
        this.get(EthFrame::invalid).clear(
            FrameData::Call(CallFrame {
                return_memory_range: inputs.return_memory_offset.clone(),
            }),
            FrameInput::Call(inputs),
            depth,
            memory,
            ExtBytecode::new_with_hash(bytecode, bytecode_hash),
            interpreter_input,
            is_static,
            ctx.cfg().spec().into(),
            gas_limit,
            checkpoint,
        );
        Ok(ItemOrResult::Item(this.consume()))
    }
}

/// Build an early-return error for a create frame.
///
/// Marked `#[cold]` so the compiler lays out error-path code away from
/// the hot instruction stream, keeping `build()` small enough to inline.
#[cold]
#[inline(never)]
fn create_error_result(
    result: InstructionResult,
    gas_limit: u64,
) -> ItemOrResult<FrameToken, FrameResult> {
    ItemOrResult::Result(FrameResult::Create(CreateOutcome {
        result: InterpreterResult {
            result,
            gas: Gas::new(gas_limit),
            output: Bytes::new(),
        },
        address: None,
    }))
}

impl FrameBuilder<CreateKind> {
    /// Consume the builder and produce a create frame (or an early result).
    #[inline]
    pub fn build<CTX, ERROR>(
        self,
        mut this: OutFrame<'_, EthFrame>,
        ctx: &mut CTX,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR>
    where
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    {
        let FrameBuilder {
            depth,
            memory,
            check_depth: do_check_depth,
            overrides,
            kind:
                CreateKind {
                    inputs,
                    check_balance,
                    bump_nonce,
                    created_address: override_address,
                    bytecode: override_bytecode,
                },
        } = self;

        let spec: SpecId = ctx.cfg().spec().into();
        let gas_limit = overrides
            .as_ref()
            .and_then(|o| o.gas_limit)
            .unwrap_or_else(|| inputs.gas_limit());

        // Check depth
        if do_check_depth && depth > CALL_STACK_LIMIT as usize {
            return Ok(create_error_result(
                InstructionResult::CallTooDeep,
                gas_limit,
            ));
        }

        // Load caller account once for balance check, nonce bump, and/or address computation.
        let needs_caller_load = check_balance || bump_nonce || override_address.is_none();
        let (created_address, init_code_hash) = if needs_caller_load {
            let mut caller_info = ctx.journal_mut().load_account_mut(inputs.caller())?;

            if check_balance && *caller_info.balance() < inputs.value() {
                return Ok(create_error_result(
                    InstructionResult::OutOfFunds,
                    gas_limit,
                ));
            }

            let old_nonce = caller_info.nonce();

            if bump_nonce && !caller_info.bump_nonce() {
                return Ok(create_error_result(InstructionResult::Return, gas_limit));
            }

            drop(caller_info);

            if let Some(addr) = override_address {
                (addr, None)
            } else {
                create_compute_address(
                    inputs.caller(),
                    old_nonce,
                    &inputs.scheme(),
                    inputs.init_code(),
                )
            }
        } else {
            // No balance check, no nonce bump, and override address is provided.
            (
                override_address.expect("override_address must be set when caller load is skipped"),
                None,
            )
        };

        // Warm load account.
        ctx.journal_mut().load_account(created_address)?;

        // Create account, transfer funds and make the journal checkpoint.
        let override_checkpoint = overrides.as_ref().and_then(|o| o.checkpoint);
        let checkpoint = if let Some(cp) = override_checkpoint {
            cp
        } else {
            match ctx.journal_mut().create_account_checkpoint(
                inputs.caller(),
                created_address,
                inputs.value(),
                spec,
            ) {
                Ok(checkpoint) => checkpoint,
                Err(e) => {
                    return Ok(create_error_result(e.into(), gas_limit));
                }
            }
        };

        let bytecode = override_bytecode.map(|b| *b).unwrap_or_else(|| {
            ExtBytecode::new_with_optional_hash(
                Bytecode::new_legacy(inputs.init_code().clone()),
                init_code_hash,
            )
        });

        // Consume `overrides` here to move out interpreter_input without copying.
        let interpreter_input = overrides
            .and_then(|o| o.interpreter_input)
            .unwrap_or_else(|| {
                create_interpreter_input(created_address, inputs.caller(), inputs.value())
            });

        this.get(EthFrame::invalid).clear(
            FrameData::Create(CreateFrame { created_address }),
            FrameInput::Create(inputs),
            depth,
            memory,
            bytecode,
            interpreter_input,
            false,
            spec,
            gas_limit,
            checkpoint,
        );
        Ok(ItemOrResult::Item(this.consume()))
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// FrameBuilder<Kind>
// ────────────────────────────────────────────────────────────────────────────────

/// Rarely-used override fields, heap-allocated only when customization
/// methods are called. On the default (hot) path this is `None` (8 bytes).
#[derive(Debug, Default)]
pub struct FrameOverrides {
    /// Externally-created journal checkpoint.
    pub checkpoint: Option<JournalCheckpoint>,
    /// Override for the interpreter input derived from call/create inputs.
    pub interpreter_input: Option<InputsImpl>,
    /// Override for the gas limit from the call/create inputs.
    pub gas_limit: Option<u64>,
}

/// Configurable builder for constructing EVM call and create frames.
///
/// `FrameBuilder` is parameterized by a `Kind` type ([`CallKind`] or [`CreateKind`])
/// that determines which frame-specific methods and build logic are available.
///
/// # Usage
///
/// **Default (unchanged behavior):**
/// ```ignore
/// FrameBuilder::new_call(depth, memory, inputs)
///     .build(out_frame, ctx, |ctx, inputs| precompiles.run(ctx, inputs))?
/// ```
///
/// **Custom call frame:**
/// ```ignore
/// FrameBuilder::new_call(depth, memory, inputs)
///     .skip_precompile_check()
///     .skip_depth_check()
///     .with_bytecode(my_bytecode, my_hash)
///     .build(out_frame, ctx, |ctx, inputs| precompiles.run(ctx, inputs))?
/// ```
///
/// **Custom create frame:**
/// ```ignore
/// FrameBuilder::new_create(depth, memory, inputs)
///     .skip_balance_check()
///     .skip_nonce_bump()
///     .with_created_address(addr)
///     .build(out_frame, ctx, |_, _| Ok(None))?
/// ```
///
/// # Note on inspector hooks
///
/// The builder operates below the inspector layer. Direct builder usage
/// (outside [`EthFrame::build_frame`]) bypasses inspector hooks
/// (`call`, `create`, `call_end`, `create_end`, `initialize_interp`).
pub struct FrameBuilder<Kind> {
    /// Call depth in the execution stack.
    depth: usize,
    /// Shared memory for the frame.
    memory: SharedMemory,
    check_depth: bool,
    /// Boxed overrides — `None` on the default path (no heap allocation).
    overrides: Option<Box<FrameOverrides>>,
    kind: Kind,
}

impl<Kind: core::fmt::Debug> core::fmt::Debug for FrameBuilder<Kind> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FrameBuilder")
            .field("depth", &self.depth)
            .field("check_depth", &self.check_depth)
            .field("overrides", &self.overrides)
            .field("kind", &self.kind)
            .finish()
    }
}

// Shared methods for any Kind.
impl<K> FrameBuilder<K> {
    /// Returns the call depth.
    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns a reference to the shared memory.
    #[inline]
    pub fn memory(&self) -> &SharedMemory {
        &self.memory
    }

    /// Skip the call-depth check (`CALL_STACK_LIMIT`).
    #[inline]
    pub fn skip_depth_check(mut self) -> Self {
        self.check_depth = false;
        self
    }

    /// Provide an externally-created journal checkpoint instead of
    /// letting the builder create one.
    ///
    /// The caller must ensure the checkpoint matches the current journal state.
    #[inline]
    pub fn with_checkpoint(mut self, cp: JournalCheckpoint) -> Self {
        self.overrides_mut().checkpoint = Some(cp);
        self
    }

    /// Override the interpreter input that would normally be derived from the call/create inputs.
    #[inline]
    pub fn with_interpreter_input(mut self, input: InputsImpl) -> Self {
        self.overrides_mut().interpreter_input = Some(input);
        self
    }

    /// Override the gas limit from the call/create inputs.
    #[inline]
    pub fn with_gas_limit(mut self, gas: u64) -> Self {
        self.overrides_mut().gas_limit = Some(gas);
        self
    }

    /// Returns a mutable reference to the overrides, allocating the box on first use.
    #[inline]
    fn overrides_mut(&mut self) -> &mut FrameOverrides {
        self.overrides
            .get_or_insert_with(|| Box::new(FrameOverrides::default()))
    }
}

// Call-specific methods.
impl FrameBuilder<CallKind> {
    /// Create a new call frame builder with default Ethereum behavior.
    #[inline]
    pub fn new_call(depth: usize, memory: SharedMemory, inputs: Box<CallInputs>) -> Self {
        Self {
            depth,
            memory,
            check_depth: true,
            overrides: None,
            kind: CallKind {
                inputs,
                transfer_value: true,
                check_precompiles: true,
                check_empty_bytecode: true,
                bytecode: None,
                is_static: None,
            },
        }
    }

    /// Skip value transfer (and the EIP-158 account touch).
    #[inline]
    pub fn skip_value_transfer(mut self) -> Self {
        self.kind.transfer_value = false;
        self
    }

    /// Skip the precompile dispatch check.
    #[inline]
    pub fn skip_precompile_check(mut self) -> Self {
        self.kind.check_precompiles = false;
        self
    }

    /// Skip the empty-bytecode early-return check.
    #[inline]
    pub fn skip_empty_bytecode_check(mut self) -> Self {
        self.kind.check_empty_bytecode = false;
        self
    }

    /// Provide bytecode directly instead of loading it from the account.
    ///
    /// Also auto-skips the precompile check (custom bytecode implies no precompile dispatch).
    #[inline]
    pub fn with_bytecode(mut self, bytecode: Bytecode, hash: B256) -> Self {
        self.kind.bytecode = Some(Box::new((bytecode, hash)));
        self.kind.check_precompiles = false;
        self
    }

    /// Override the `is_static` flag from the call inputs.
    #[inline]
    pub fn with_is_static(mut self, is_static: bool) -> Self {
        self.kind.is_static = Some(is_static);
        self
    }
}

// Create-specific methods.
impl FrameBuilder<CreateKind> {
    /// Create a new create frame builder with default Ethereum behavior.
    #[inline]
    pub fn new_create(depth: usize, memory: SharedMemory, inputs: Box<CreateInputs>) -> Self {
        Self {
            depth,
            memory,
            check_depth: true,
            overrides: None,
            kind: CreateKind {
                inputs,
                check_balance: true,
                bump_nonce: true,
                created_address: None,
                bytecode: None,
            },
        }
    }

    /// Skip the caller balance check.
    #[inline]
    pub fn skip_balance_check(mut self) -> Self {
        self.kind.check_balance = false;
        self
    }

    /// Skip the caller nonce bump.
    ///
    /// When nonce bump is skipped, a `created_address` should be provided
    /// externally since the CREATE address depends on the old nonce.
    #[inline]
    pub fn skip_nonce_bump(mut self) -> Self {
        self.kind.bump_nonce = false;
        self
    }

    /// Provide a pre-computed created address.
    ///
    /// This only overrides the address; the nonce bump still happens
    /// by default. Chain [`.skip_nonce_bump()`](Self::skip_nonce_bump)
    /// explicitly if the nonce bump should also be skipped.
    #[inline]
    pub fn with_created_address(mut self, addr: Address) -> Self {
        self.kind.created_address = Some(addr);
        self
    }

    /// Provide init bytecode directly instead of deriving it from the create inputs.
    #[inline]
    pub fn with_bytecode(mut self, bytecode: ExtBytecode) -> Self {
        self.kind.bytecode = Some(Box::new(bytecode));
        self
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// FrameBuilderKind
// ────────────────────────────────────────────────────────────────────────────────

/// Wraps either a [`FrameBuilder<CallKind>`] or [`FrameBuilder<CreateKind>`],
/// returned by [`EthFrame::build_frame`] when the frame type is determined at runtime.
#[derive(Debug)]
pub enum FrameBuilderKind {
    /// Call frame builder.
    Call(FrameBuilder<CallKind>),
    /// Create frame builder.
    Create(FrameBuilder<CreateKind>),
}

impl FrameBuilderKind {
    /// Consume the builder and produce a frame (or an early result).
    ///
    /// Dispatches to [`FrameBuilder::build`] on the inner variant.
    /// The `precompile_fn` closure is only used by the [`Call`](Self::Call) variant.
    #[inline]
    pub fn build<CTX, ERROR>(
        self,
        this: OutFrame<'_, EthFrame>,
        ctx: &mut CTX,
        precompile_fn: impl FnMut(&mut CTX, &CallInputs) -> Result<Option<InterpreterResult>, String>,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR>
    where
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    {
        match self {
            Self::Call(builder) => builder.build(this, ctx, precompile_fn),
            Self::Create(builder) => builder.build(this, ctx),
        }
    }
}

// ────────────────────────────────────────────────────────────────────────────────
// EthFrame
// ────────────────────────────────────────────────────────────────────────────────

/// Frame implementation for Ethereum.
#[derive_where(Clone, Debug; IW,
    <IW as InterpreterTypes>::Stack,
    <IW as InterpreterTypes>::Memory,
    <IW as InterpreterTypes>::Bytecode,
    <IW as InterpreterTypes>::ReturnData,
    <IW as InterpreterTypes>::Input,
    <IW as InterpreterTypes>::RuntimeFlag,
    <IW as InterpreterTypes>::Extend,
)]
pub struct EthFrame<IW: InterpreterTypes = EthInterpreter> {
    /// Frame-specific data (Call, Create, or EOFCreate).
    pub data: FrameData,
    /// Input data for the frame.
    pub input: FrameInput,
    /// Current call depth in the execution stack.
    pub depth: usize,
    /// Journal checkpoint for state reversion.
    pub checkpoint: JournalCheckpoint,
    /// Interpreter instance for executing bytecode.
    pub interpreter: Interpreter<IW>,
    /// Whether the frame has been finished its execution.
    /// Frame is considered finished if it has been called and returned a result.
    pub is_finished: bool,
}

impl<IT: InterpreterTypes> FrameTr for EthFrame<IT> {
    type FrameResult = FrameResult;
    type FrameInit = FrameInit;
}

impl Default for EthFrame<EthInterpreter> {
    fn default() -> Self {
        Self::do_default(Interpreter::default())
    }
}

impl EthFrame<EthInterpreter> {
    /// Creates an new invalid [`EthFrame`].
    pub fn invalid() -> Self {
        Self::do_default(Interpreter::invalid())
    }

    fn do_default(interpreter: Interpreter<EthInterpreter>) -> Self {
        Self {
            data: FrameData::Call(CallFrame {
                return_memory_range: 0..0,
            }),
            input: FrameInput::Empty,
            depth: 0,
            checkpoint: JournalCheckpoint::default(),
            interpreter,
            is_finished: false,
        }
    }

    /// Returns true if the frame has finished execution.
    pub fn is_finished(&self) -> bool {
        self.is_finished
    }

    /// Sets the finished state of the frame.
    pub fn set_finished(&mut self, finished: bool) {
        self.is_finished = finished;
    }
}

/// Type alias for database errors from a context.
pub type ContextTrDbError<CTX> = <<CTX as ContextTr>::Db as Database>::Error;

impl EthFrame<EthInterpreter> {
    /// Clear and initialize a frame.
    #[allow(clippy::too_many_arguments)]
    #[inline(always)]
    pub fn clear(
        &mut self,
        data: FrameData,
        input: FrameInput,
        depth: usize,
        memory: SharedMemory,
        bytecode: ExtBytecode,
        inputs: InputsImpl,
        is_static: bool,
        spec_id: SpecId,
        gas_limit: u64,
        checkpoint: JournalCheckpoint,
    ) {
        let Self {
            data: data_ref,
            input: input_ref,
            depth: depth_ref,
            interpreter,
            checkpoint: checkpoint_ref,
            is_finished: is_finished_ref,
        } = self;
        *data_ref = data;
        *input_ref = input;
        *depth_ref = depth;
        *is_finished_ref = false;
        interpreter.clear(memory, bytecode, inputs, is_static, spec_id, gas_limit);
        *checkpoint_ref = checkpoint;
    }

    /// Returns a [`FrameBuilder<CallKind>`] for constructing a call frame.
    ///
    /// The builder can be customized before calling [`.build()`](FrameBuilder::build).
    #[inline]
    pub fn build_call_frame(
        depth: usize,
        memory: SharedMemory,
        inputs: Box<CallInputs>,
    ) -> FrameBuilder<CallKind> {
        FrameBuilder::new_call(depth, memory, inputs)
    }

    /// Returns a [`FrameBuilder<CreateKind>`] for constructing a create frame.
    ///
    /// The builder can be customized before calling [`.build()`](FrameBuilder::build).
    #[inline]
    pub fn build_create_frame(
        depth: usize,
        memory: SharedMemory,
        inputs: Box<CreateInputs>,
    ) -> FrameBuilder<CreateKind> {
        FrameBuilder::new_create(depth, memory, inputs)
    }

    /// Returns a [`FrameBuilderKind`] for constructing a call or create frame
    /// based on the given [`FrameInput`].
    ///
    /// The builder can be customized before calling [`.build()`](FrameBuilderKind::build).
    pub fn build_frame(frame_init: FrameInit) -> FrameBuilderKind {
        let FrameInit {
            depth,
            memory,
            frame_input,
        } = frame_init;

        match frame_input {
            FrameInput::Call(inputs) => {
                FrameBuilderKind::Call(FrameBuilder::new_call(depth, memory, inputs))
            }
            FrameInput::Create(inputs) => {
                FrameBuilderKind::Create(FrameBuilder::new_create(depth, memory, inputs))
            }
            FrameInput::Empty => unreachable!(),
        }
    }

    /// Make call frame
    #[deprecated(note = "Use `build_call_frame` instead")]
    #[inline]
    pub fn make_call_frame<
        CTX: ContextTr,
        PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        this: OutFrame<'_, Self>,
        ctx: &mut CTX,
        precompiles: &mut PRECOMPILES,
        depth: usize,
        memory: SharedMemory,
        inputs: Box<CallInputs>,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR> {
        Self::build_call_frame(depth, memory, inputs)
            .build(this, ctx, |ctx, inputs| precompiles.run(ctx, inputs))
    }

    /// Make create frame.
    #[deprecated(note = "Use `build_create_frame` instead")]
    #[inline]
    pub fn make_create_frame<
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        this: OutFrame<'_, Self>,
        context: &mut CTX,
        depth: usize,
        memory: SharedMemory,
        inputs: Box<CreateInputs>,
    ) -> Result<ItemOrResult<FrameToken, FrameResult>, ERROR> {
        Self::build_create_frame(depth, memory, inputs).build(this, context)
    }

    /// Initializes a frame with the given context and precompiles.
    #[deprecated(note = "Use `build_frame` instead")]
    pub fn init_with_context<
        CTX: ContextTr,
        PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
    >(
        this: OutFrame<'_, Self>,
        ctx: &mut CTX,
        precompiles: &mut PRECOMPILES,
        frame_init: FrameInit,
    ) -> Result<
        ItemOrResult<FrameToken, FrameResult>,
        ContextError<<<CTX as ContextTr>::Db as Database>::Error>,
    > {
        Self::build_frame(frame_init).build(this, ctx, |ctx, inputs| precompiles.run(ctx, inputs))
    }
}

impl EthFrame<EthInterpreter> {
    /// Processes the next interpreter action, either creating a new frame or returning a result.
    pub fn process_next_action<
        CTX: ContextTr,
        ERROR: From<ContextTrDbError<CTX>> + FromStringError,
    >(
        &mut self,
        context: &mut CTX,
        next_action: InterpreterAction,
    ) -> Result<FrameInitOrResult<Self>, ERROR> {
        // Run interpreter

        let mut interpreter_result = match next_action {
            InterpreterAction::NewFrame(frame_input) => {
                let depth = self.depth + 1;
                return Ok(ItemOrResult::Item(FrameInit {
                    frame_input,
                    depth,
                    memory: self.interpreter.memory.new_child_context(),
                }));
            }
            InterpreterAction::Return(result) => result,
        };

        // Handle return from frame
        let result = match &self.data {
            FrameData::Call(frame) => {
                // return_call
                // Revert changes or not.
                if interpreter_result.result.is_ok() {
                    context.journal_mut().checkpoint_commit();
                } else {
                    context.journal_mut().checkpoint_revert(self.checkpoint);
                }
                ItemOrResult::Result(FrameResult::Call(CallOutcome::new(
                    interpreter_result,
                    frame.return_memory_range.clone(),
                )))
            }
            FrameData::Create(frame) => {
                let (cfg, journal) = context.cfg_journal_mut();
                return_create(
                    journal,
                    cfg,
                    self.checkpoint,
                    &mut interpreter_result,
                    frame.created_address,
                );

                ItemOrResult::Result(FrameResult::Create(CreateOutcome::new(
                    interpreter_result,
                    Some(frame.created_address),
                )))
            }
        };

        Ok(result)
    }

    /// Processes a frame result and updates the interpreter state accordingly.
    pub fn return_result<CTX: ContextTr, ERROR: From<ContextTrDbError<CTX>> + FromStringError>(
        &mut self,
        ctx: &mut CTX,
        result: FrameResult,
    ) -> Result<(), ERROR> {
        self.interpreter.memory.free_child_context();
        take_error::<ERROR, _>(ctx.error())?;

        // Insert result to the top frame.
        match result {
            FrameResult::Call(outcome) => {
                let out_gas = outcome.gas();
                let ins_result = *outcome.instruction_result();
                let returned_len = outcome.result.output.len();

                let interpreter = &mut self.interpreter;
                let mem_length = outcome.memory_length();
                let mem_start = outcome.memory_start();
                interpreter.return_data.set_buffer(outcome.result.output);

                let target_len = min(mem_length, returned_len);

                if ins_result == InstructionResult::FatalExternalError {
                    panic!("Fatal external error in insert_call_outcome");
                }

                let item = if ins_result.is_ok() {
                    U256::from(1)
                } else {
                    U256::ZERO
                };
                // Safe to push without stack limit check
                let _ = interpreter.stack.push(item);

                // Return unspend gas.
                if ins_result.is_ok_or_revert() {
                    interpreter.gas.erase_cost(out_gas.remaining());
                    interpreter
                        .memory
                        .set(mem_start, &interpreter.return_data.buffer()[..target_len]);
                }

                if ins_result.is_ok() {
                    interpreter.gas.record_refund(out_gas.refunded());
                }
            }
            FrameResult::Create(outcome) => {
                let instruction_result = *outcome.instruction_result();
                let interpreter = &mut self.interpreter;

                if instruction_result == InstructionResult::Revert {
                    // Save data to return data buffer if the create reverted
                    interpreter
                        .return_data
                        .set_buffer(outcome.output().to_owned());
                } else {
                    // Otherwise clear it. Note that RETURN opcode should abort.
                    interpreter.return_data.clear();
                };

                assert_ne!(
                    instruction_result,
                    InstructionResult::FatalExternalError,
                    "Fatal external error in insert_eofcreate_outcome"
                );

                let this_gas = &mut interpreter.gas;
                if instruction_result.is_ok_or_revert() {
                    this_gas.erase_cost(outcome.gas().remaining());
                }

                let stack_item = if instruction_result.is_ok() {
                    this_gas.record_refund(outcome.gas().refunded());
                    outcome.address.unwrap_or_default().into_word().into()
                } else {
                    U256::ZERO
                };

                // Safe to push without stack limit check
                let _ = interpreter.stack.push(stack_item);
            }
        }

        Ok(())
    }
}

/// Handles the result of a CREATE operation, including validation and state updates.
pub fn return_create<JOURNAL: JournalTr, CFG: Cfg>(
    journal: &mut JOURNAL,
    cfg: CFG,
    checkpoint: JournalCheckpoint,
    interpreter_result: &mut InterpreterResult,
    address: Address,
) {
    let max_code_size = cfg.max_code_size();
    let is_eip3541_disabled = cfg.is_eip3541_disabled();
    let spec_id = cfg.spec().into();

    // If return is not ok revert and return.
    if !interpreter_result.result.is_ok() {
        journal.checkpoint_revert(checkpoint);
        return;
    }
    // Host error if present on execution
    // If ok, check contract creation limit and calculate gas deduction on output len.
    //
    // EIP-3541: Reject new contract code starting with the 0xEF byte
    if !is_eip3541_disabled
        && spec_id.is_enabled_in(LONDON)
        && interpreter_result.output.first() == Some(&0xEF)
    {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::CreateContractStartingWithEF;
        return;
    }

    // EIP-170: Contract code size limit to 0x6000 (~25kb)
    // EIP-7954 increased this limit to 0x8000 (~32kb).
    if spec_id.is_enabled_in(SPURIOUS_DRAGON) && interpreter_result.output.len() > max_code_size {
        journal.checkpoint_revert(checkpoint);
        interpreter_result.result = InstructionResult::CreateContractSizeLimit;
        return;
    }
    let gas_for_code = cfg
        .gas_params()
        .code_deposit_cost(interpreter_result.output.len());
    if !interpreter_result.gas.record_cost(gas_for_code) {
        // Record code deposit gas cost and check if we are out of gas.
        // EIP-2 point 3: If contract creation does not have enough gas to pay for the
        // final gas fee for adding the contract code to the state, the contract
        // creation fails (i.e. goes out-of-gas) rather than leaving an empty contract.
        if spec_id.is_enabled_in(HOMESTEAD) {
            journal.checkpoint_revert(checkpoint);
            interpreter_result.result = InstructionResult::OutOfGas;
            return;
        } else {
            interpreter_result.output = Bytes::new();
        }
    }
    // If we have enough gas we can commit changes.
    journal.checkpoint_commit();

    // Do analysis of bytecode straight away.
    let bytecode = Bytecode::new_legacy(interpreter_result.output.clone());

    // Set code
    journal.set_code(address, bytecode);

    interpreter_result.result = InstructionResult::Return;
}

#[cfg(test)]
mod tests {
    use super::*;
    use context::Context;
    use context_interface::context::ContextError;
    use context_interface::local::OutFrame;
    use database::InMemoryDB;
    use interpreter::{CallInput, CallScheme};
    use primitives::{address, hardfork::SpecId};
    use state::AccountInfo;
    use std::convert::Infallible;

    type TestError = ContextError<Infallible>;

    const CALLER: Address = address!("0x0000000000000000000000000000000000000001");
    const TARGET: Address = address!("0x0000000000000000000000000000000000000002");

    fn test_call_inputs() -> Box<CallInputs> {
        Box::new(CallInputs {
            input: CallInput::Bytes(Bytes::new()),
            return_memory_offset: 0..0,
            gas_limit: 100_000,
            bytecode_address: TARGET,
            known_bytecode: None,
            target_address: TARGET,
            caller: CALLER,
            value: CallValue::Transfer(U256::ZERO),
            scheme: CallScheme::Call,
            is_static: false,
        })
    }

    fn test_create_inputs() -> Box<CreateInputs> {
        Box::new(CreateInputs::new(
            CALLER,
            CreateScheme::Create,
            U256::ZERO,
            Bytes::from_static(&[0x60, 0x00]),
            100_000,
        ))
    }

    fn test_ctx() -> Context<context::BlockEnv, context::TxEnv, context::CfgEnv, InMemoryDB> {
        let mut db = InMemoryDB::default();
        db.insert_account_info(
            CALLER,
            AccountInfo {
                balance: U256::from(1_000_000),
                nonce: 0,
                ..Default::default()
            },
        );
        Context::new(db, SpecId::CANCUN)
    }

    // ─── Standalone function tests ───

    #[test]
    fn test_check_depth_within_limit() {
        assert!(check_depth(0).is_ok());
        assert!(check_depth(CALL_STACK_LIMIT as usize - 1).is_ok());
        assert!(check_depth(CALL_STACK_LIMIT as usize).is_ok());
    }

    #[test]
    fn test_check_depth_exceeds_limit() {
        assert_eq!(
            check_depth(CALL_STACK_LIMIT as usize + 1),
            Err(InstructionResult::CallTooDeep)
        );
    }

    #[test]
    fn test_create_compute_address_create() {
        let (addr, hash) = create_compute_address(CALLER, 0, &CreateScheme::Create, &Bytes::new());
        assert_eq!(addr, CALLER.create(0));
        assert!(hash.is_none());
    }

    #[test]
    fn test_create_compute_address_create2() {
        let init_code = Bytes::from_static(&[0x60, 0x00]);
        let salt = U256::from(42);
        let (addr, hash) =
            create_compute_address(CALLER, 0, &CreateScheme::Create2 { salt }, &init_code);
        let expected_hash = keccak256(&init_code);
        assert_eq!(addr, CALLER.create2(salt.to_be_bytes(), expected_hash));
        assert_eq!(hash, Some(expected_hash));
    }

    // ─── Builder depth check tests ───

    #[test]
    fn test_call_builder_depth_check_returns_early() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);

        let result: Result<_, TestError> = FrameBuilder::new_call(
            CALL_STACK_LIMIT as usize + 1,
            SharedMemory::new(),
            test_call_inputs(),
        )
        .build(out, &mut ctx, |_, _| Ok(None));

        match result.unwrap() {
            ItemOrResult::Result(FrameResult::Call(outcome)) => {
                assert_eq!(outcome.result.result, InstructionResult::CallTooDeep);
            }
            _ => panic!("expected early return with CallTooDeep"),
        }
    }

    #[test]
    fn test_call_builder_skip_depth_check_succeeds() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);

        let bytecode = Bytecode::new_legacy(Bytes::from_static(&[0x00])); // STOP
        let hash = B256::ZERO;

        let result: Result<_, TestError> = FrameBuilder::new_call(
            CALL_STACK_LIMIT as usize + 1,
            SharedMemory::new(),
            test_call_inputs(),
        )
        .skip_depth_check()
        .skip_value_transfer()
        .with_bytecode(bytecode, hash) // also skips precompile check
        .build(out, &mut ctx, |_, _| Ok(None));

        match result.unwrap() {
            ItemOrResult::Item(_) => {} // Frame was created successfully
            ItemOrResult::Result(r) => panic!("expected frame creation, got result: {r:?}"),
        }
    }

    #[test]
    fn test_create_builder_depth_check_returns_early() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);

        let result: Result<_, TestError> = FrameBuilder::new_create(
            CALL_STACK_LIMIT as usize + 1,
            SharedMemory::new(),
            test_create_inputs(),
        )
        .build(out, &mut ctx);

        match result.unwrap() {
            ItemOrResult::Result(FrameResult::Create(outcome)) => {
                assert_eq!(outcome.result.result, InstructionResult::CallTooDeep);
            }
            _ => panic!("expected early return with CallTooDeep"),
        }
    }

    // ─── Builder balance / nonce tests ───

    #[test]
    fn test_create_builder_balance_check_fails() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);

        // Request more value than the caller has
        let mut inputs = test_create_inputs();
        inputs.set_value(U256::from(999_999_999));

        let result: Result<_, TestError> =
            FrameBuilder::new_create(0, SharedMemory::new(), inputs).build(out, &mut ctx);

        match result.unwrap() {
            ItemOrResult::Result(FrameResult::Create(outcome)) => {
                assert_eq!(outcome.result.result, InstructionResult::OutOfFunds);
            }
            _ => panic!("expected OutOfFunds"),
        }
    }

    #[test]
    fn test_create_builder_skip_balance_check_succeeds() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);

        // Request more value than the caller has, but skip balance check
        let mut inputs = test_create_inputs();
        inputs.set_value(U256::from(999_999_999));

        let result: Result<_, TestError> = FrameBuilder::new_create(0, SharedMemory::new(), inputs)
            .skip_balance_check()
            .build(out, &mut ctx);

        match result.unwrap() {
            ItemOrResult::Item(_) => {} // Frame created despite insufficient balance
            ItemOrResult::Result(r) => panic!("expected frame creation, got result: {r:?}"),
        }
    }

    #[test]
    fn test_create_builder_with_created_address() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);
        let custom_addr = address!("0x00000000000000000000000000000000deadbeef");

        let result: Result<_, TestError> =
            FrameBuilder::new_create(0, SharedMemory::new(), test_create_inputs())
                .with_created_address(custom_addr)
                .build(out, &mut ctx);

        match result.unwrap() {
            ItemOrResult::Item(_) => {
                // Verify the frame was created with the custom address
                match &frame.data {
                    FrameData::Create(cf) => assert_eq!(cf.created_address, custom_addr),
                    _ => panic!("expected Create frame data"),
                }
            }
            ItemOrResult::Result(r) => panic!("expected frame creation, got result: {r:?}"),
        }
    }

    // ─── Builder call-specific tests ───

    #[test]
    fn test_call_builder_empty_bytecode_returns_stop() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);

        // Target has no code, so bytecode is empty
        let result: Result<_, TestError> =
            FrameBuilder::new_call(0, SharedMemory::new(), test_call_inputs())
                .skip_value_transfer()
                .build(out, &mut ctx, |_, _| Ok(None));

        match result.unwrap() {
            ItemOrResult::Result(FrameResult::Call(outcome)) => {
                assert_eq!(outcome.result.result, InstructionResult::Stop);
            }
            _ => panic!("expected Stop for empty bytecode"),
        }
    }

    #[test]
    fn test_call_builder_skip_empty_bytecode_check() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);

        // Target has no code, but we skip the empty bytecode check
        let result: Result<_, TestError> =
            FrameBuilder::new_call(0, SharedMemory::new(), test_call_inputs())
                .skip_value_transfer()
                .skip_empty_bytecode_check()
                .build(out, &mut ctx, |_, _| Ok(None));

        match result.unwrap() {
            ItemOrResult::Item(_) => {} // Frame created despite empty bytecode
            ItemOrResult::Result(r) => panic!("expected frame creation, got result: {r:?}"),
        }
    }

    #[test]
    fn test_call_builder_with_bytecode_skips_precompile() {
        let mut ctx = test_ctx();
        let mut frame = EthFrame::invalid();
        let out = OutFrame::new_init(&mut frame);

        let bytecode = Bytecode::new_legacy(Bytes::from_static(&[0x00]));
        let hash = B256::ZERO;

        // Precompile fn that would panic if called
        let result: Result<_, TestError> =
            FrameBuilder::new_call(0, SharedMemory::new(), test_call_inputs())
                .skip_value_transfer()
                .with_bytecode(bytecode, hash)
                .build(out, &mut ctx, |_, _| {
                    panic!("precompile should not be called")
                });

        match result.unwrap() {
            ItemOrResult::Item(_) => {} // Success — precompile was skipped
            ItemOrResult::Result(r) => panic!("expected frame creation, got result: {r:?}"),
        }
    }

    // ─── Builder getter tests ───

    #[test]
    fn test_builder_getters() {
        let builder = FrameBuilder::new_call(5, SharedMemory::new(), test_call_inputs());
        assert_eq!(builder.depth(), 5);
        // memory() returns a reference — just verify it doesn't panic
        let _ = builder.memory();
    }

    #[test]
    fn test_frame_builder_size_reduction() {
        let call_size = std::mem::size_of::<FrameBuilder<CallKind>>();
        let create_size = std::mem::size_of::<FrameBuilder<CreateKind>>();
        // With boxed overrides, builders should be well under 100 bytes on the default path.
        // Before boxing, CallKind builder was ~300 bytes.
        assert!(
            call_size < 100,
            "FrameBuilder<CallKind> is {call_size} bytes, expected < 100"
        );
        assert!(
            create_size < 100,
            "FrameBuilder<CreateKind> is {create_size} bytes, expected < 100"
        );
    }
}
