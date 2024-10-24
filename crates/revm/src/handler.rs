// Modules.
pub mod mainnet;
pub mod register;
mod wires;

use context::{
    BlockGetter, CfgGetter, ErrorGetter, FrameResult, JournalCheckpoint, JournalStateGetter,
    JournalStateGetterDBError, TransactionGetter,
};
// Exports.
use mainnet::{EthExecution, EthFrame, EthPostExecution, EthPreExecution, EthValidation};
use primitives::Log;
use state::EvmState;
pub use wires::*;

// Includes.

use crate::{Context, EvmWiring};
use core::mem;
use interpreter::{table::InstructionTables, Host};
use register::{EvmHandler, HandleRegisters};
use specification::{hardfork::Spec, spec_to_generic};
use std::vec::Vec;
use wiring::{
    journaled_state::JournaledState,
    result::{
        EVMError, EVMErrorWiring, HaltReason, InvalidHeader, InvalidTransaction, ResultAndState,
    },
    Transaction,
};

pub mod temp {
    pub use super::*;

    pub trait Handler {
        type Val: ValidationWire;

        fn validation(&self) -> &Self::Val;
    }

    pub struct InspectorHandle<HAL: Handler, INSP> {
        pub handler: HAL,
        pub inspector: INSP,
    }

    // Can be done with custom trait.
    //evm.handler.inspector =

    impl<HAL: Handler, INSP> ValidationWire for InspectorHandle<HAL, INSP> {
        type Context = <<HAL as Handler>::Val as ValidationWire>::Context;
        type Error = <<HAL as Handler>::Val as ValidationWire>::Error;

        fn validate_env(&self, context: &Self::Context) -> Result<(), Self::Error> {
            self.handler.validation().validate_env(context)
        }

        fn validate_tx_against_state(
            &self,
            context: &mut Self::Context,
        ) -> Result<(), Self::Error> {
            self.handler.validation().validate_tx_against_state(context)
        }

        fn validate_initial_tx_gas(&self, context: &Self::Context) -> Result<u64, Self::Error> {
            self.handler.validation().validate_initial_tx_gas(context)
        }
    }

    impl<HAL: Handler, INSP> Handler for InspectorHandle<HAL, INSP> {
        type Val = Self;

        fn validation(&self) -> &Self::Val {
            &self
        }
    }
}

use self::register::{HandleRegister, HandleRegisterBox};

pub trait Hand {
    type Validation: ValidationWire;
    type PreExecution: PreExecutionWire;
    type Execution: ExecutionWire;
    type PostExecution: PostExecutionWire;

    // TODO
    type Precompiles;
    type InstructionTable;

    fn validation(&self) -> &Self::Validation;
    fn pre_execution(&self) -> &Self::PreExecution;
    fn execution(&self) -> &Self::Execution;
    fn post_execution(&self) -> &Self::PostExecution;
}

/// TODO Halt needs to be generalized.
pub struct EthHand<CTX, ERROR> {
    pub validation: EthValidation<CTX, ERROR>,
    pub pre_execution: EthPreExecution<CTX, ERROR>,
    pub execution: EthExecution<CTX, ERROR>,
    pub post_execution: EthPostExecution<CTX, ERROR, HaltReason>,
}

impl<CTX, ERROR> Hand for EthHand<CTX, ERROR>
where
    CTX: TransactionGetter
        + BlockGetter
        + JournalStateGetter
        + CfgGetter
        + ErrorGetter<Error = ERROR>
        + JournalStateGetter<
            Journal: JournaledState<
                FinalOutput = (EvmState, Vec<Log>),
                Checkpoint = JournalCheckpoint,
            >,
        >,
    ERROR: From<InvalidTransaction> + From<InvalidHeader> + From<JournalStateGetterDBError<CTX>>,
{
    type Validation = EthValidation<CTX, ERROR>;
    type PreExecution = EthPreExecution<CTX, ERROR>;
    type Execution = EthExecution<CTX, ERROR>;
    type PostExecution = EthPostExecution<CTX, ERROR, HaltReason>;

    type Precompiles = ();
    type InstructionTable = InstructionTables<'static, CTX>;

    fn validation(&self) -> &Self::Validation {
        &self.validation
    }

    fn pre_execution(&self) -> &Self::PreExecution {
        &self.pre_execution
    }

    fn execution(&self) -> &Self::Execution {
        &self.execution
    }

    fn post_execution(&self) -> &Self::PostExecution {
        &self.post_execution
    }
}

/// Handler acts as a proxy and allow to define different behavior for different
/// sections of the code. This allows nice integration of different chains or
/// to disable some mainnet behavior.
pub struct Handler<'a, EvmWiringT: EvmWiring, H: Host + 'a> {
    /// Handler hardfork
    pub spec_id: EvmWiringT::Hardfork,
    /// Instruction table type.
    pub instruction_table: InstructionTables<'a, H>,
    /// Registers that will be called on initialization.
    pub registers: Vec<HandleRegisters<'a, EvmWiringT>>,
    /// New Validation
    pub validation: Box<
        dyn ValidationWire<Context = Context<EvmWiringT>, Error = EVMErrorWiring<EvmWiringT>> + 'a,
    >,
    /// Pre execution handle.
    pub pre_execution: Box<
        dyn PreExecutionWire<
                Context = Context<EvmWiringT>,
                Precompiles = (),
                Error = EVMErrorWiring<EvmWiringT>,
            > + 'a,
    >,
    /// Execution loop that handles frames.
    pub execution: Box<
        dyn ExecutionWire<
                Context = Context<EvmWiringT>,
                Error = EVMErrorWiring<EvmWiringT>,
                Frame = EthFrame<Context<EvmWiringT>, EVMErrorWiring<EvmWiringT>>,
                ExecResult = FrameResult,
            > + 'a,
    >,
    /// Post Execution handle.
    pub post_execution: Box<
        dyn PostExecutionWire<
                Context = Context<EvmWiringT>,
                Error = EVMErrorWiring<EvmWiringT>,
                ExecResult = FrameResult,
                Output = ResultAndState<EvmWiringT::HaltReason>,
            > + 'a,
    >,
    //pub execution: ExecutionHandler<'a, EvmWiringT>,
}

impl<'a, EvmWiringT> EvmHandler<'a, EvmWiringT>
where
    EvmWiringT: EvmWiring<Transaction: Transaction<TransactionError: From<InvalidTransaction>>>,
{
    /// Creates a base/vanilla Ethereum handler with the provided spec id.
    pub fn mainnet_with_spec(spec_id: EvmWiringT::Hardfork) -> Self {
        spec_to_generic!(
            spec_id.into(),
            Self {
                spec_id,
                instruction_table: InstructionTables::new_plain::<SPEC>(),
                registers: Vec::new(),
                pre_execution:
                    EthPreExecution::<Context<EvmWiringT>, EVMErrorWiring<EvmWiringT>>::new_boxed(
                        SPEC::SPEC_ID
                    ),
                post_execution: EthPostExecution::<
                    Context<EvmWiringT>,
                    EVMErrorWiring<EvmWiringT>,
                    EvmWiringT::HaltReason,
                >::new_boxed(SPEC::SPEC_ID),
                validation:
                    EthValidation::<Context<EvmWiringT>, EVMErrorWiring<EvmWiringT>>::new_boxed(
                        SPEC::SPEC_ID
                    ),
                execution:
                    EthExecution::<Context<EvmWiringT>, EVMErrorWiring<EvmWiringT>>::new_boxed(
                        SPEC::SPEC_ID
                    ),
            }
        )
    }
}

impl<'a, EvmWiringT: EvmWiring> EvmHandler<'a, EvmWiringT> {
    /// Returns the specification ID.
    pub fn spec_id(&self) -> EvmWiringT::Hardfork {
        self.spec_id
    }

    /// Executes call frame.
    // pub fn execute_frame(
    //     &self,
    //     frame: &mut Frame,
    //     shared_memory: &mut SharedMemory,
    //     context: &mut Context<EvmWiringT>,
    // ) -> EVMResultGeneric<InterpreterAction, EvmWiringT> {
    //     self.execution
    //         .execute_frame(frame, shared_memory, &self.instruction_table, context)
    // }

    /// Take instruction table.
    pub fn take_instruction_table(&mut self) -> InstructionTables<'a, Context<EvmWiringT>> {
        let spec_id = self.spec_id();
        mem::replace(
            &mut self.instruction_table,
            spec_to_generic!(spec_id.into(), InstructionTables::new_plain::<SPEC>()),
        )
    }

    /// Set instruction table.
    pub fn set_instruction_table(&mut self, table: InstructionTables<'a, Context<EvmWiringT>>) {
        self.instruction_table = table;
    }

    /// Returns reference to pre execution handler.
    pub fn pre_execution(
        &self,
    ) -> &dyn PreExecutionWire<
        Context = Context<EvmWiringT>,
        Precompiles = (),
        Error = EVMErrorWiring<EvmWiringT>,
    > {
        self.pre_execution.as_ref()
    }

    /// Returns reference to pre execution handler.
    pub fn post_execution(
        &self,
    ) -> &dyn PostExecutionWire<
        Context = Context<EvmWiringT>,
        Error = EVMErrorWiring<EvmWiringT>,
        ExecResult = FrameResult,
        Output = ResultAndState<EvmWiringT::HaltReason>,
    > {
        self.post_execution.as_ref()
    }

    /// Returns reference to frame handler.
    pub fn execution(
        &self,
    ) -> &dyn ExecutionWire<
        Context = Context<EvmWiringT>,
        Error = EVMErrorWiring<EvmWiringT>,
        Frame = EthFrame<Context<EvmWiringT>, EVMErrorWiring<EvmWiringT>>,
        ExecResult = FrameResult,
    > {
        self.execution.as_ref()
    }

    /// Returns reference to validation handler.
    pub fn validation(
        &self,
    ) -> &dyn ValidationWire<Context = Context<EvmWiringT>, Error = EVMErrorWiring<EvmWiringT>>
    {
        self.validation.as_ref()
    }

    /// Append handle register.
    pub fn append_handler_register(&mut self, register: HandleRegisters<'a, EvmWiringT>) {
        register.register(self);
        self.registers.push(register);
    }

    /// Append plain handle register.
    pub fn append_handler_register_plain(&mut self, register: HandleRegister<EvmWiringT>) {
        register(self);
        self.registers.push(HandleRegisters::Plain(register));
    }

    /// Append boxed handle register.
    pub fn append_handler_register_box(&mut self, register: HandleRegisterBox<'a, EvmWiringT>) {
        register(self);
        self.registers.push(HandleRegisters::Box(register));
    }
}

impl<'a, EvmWiringT: EvmWiring> EvmHandler<'a, EvmWiringT> {
    /// Pop last handle register and reapply all registers that are left.
    pub fn pop_handle_register(&mut self) -> Option<HandleRegisters<'a, EvmWiringT>> {
        let out = self.registers.pop();
        if out.is_some() {
            let registers = core::mem::take(&mut self.registers);
            let mut base_handler = EvmWiringT::handler::<'a>(self.spec_id);
            // apply all registers to default handler and raw mainnet instruction table.
            for register in registers {
                base_handler.append_handler_register(register)
            }
            *self = base_handler;
        }
        out
    }

    /// Creates the Handler with variable SpecId, inside it will call function with Generic Spec.
    pub fn modify_spec_id(&mut self, spec_id: EvmWiringT::Hardfork) {
        if self.spec_id == spec_id {
            return;
        }

        let registers = core::mem::take(&mut self.registers);
        // register for optimism is added as a register, so we need to create mainnet handler here.
        let mut handler = EvmWiringT::handler::<'a>(spec_id);
        // apply all registers to default handler and raw mainnet instruction table.
        for register in registers {
            handler.append_handler_register(register)
        }
        handler.spec_id = spec_id;
        *self = handler;
    }
}

#[cfg(test)]
mod test {
    use core::cell::RefCell;
    use database_interface::EmptyDB;
    use std::{rc::Rc, sync::Arc};
    use wiring::{result::EVMError, EthereumWiring, EvmWiring};

    use super::*;

    type TestEvmWiring = EthereumWiring<EmptyDB, ()>;

    #[test]
    fn test_handler_register_pop() {
        let register = |inner: &Rc<RefCell<i32>>| -> HandleRegisterBox<'_, TestEvmWiring> {
            let inner = inner.clone();
            Box::new(move |h| {
                *inner.borrow_mut() += 1;
                //h.post_execution.output = Arc::new(|_, _| Err(EVMError::Custom("test".into())))
            })
        };

        let mut handler = EvmHandler::<'_, TestEvmWiring>::mainnet_with_spec(
            <TestEvmWiring as EvmWiring>::Hardfork::default(),
        );
        let test = Rc::new(RefCell::new(0));

        handler.append_handler_register_box(register(&test));
        assert_eq!(*test.borrow(), 1);

        handler.append_handler_register_box(register(&test));
        assert_eq!(*test.borrow(), 2);

        assert!(handler.pop_handle_register().is_some());

        // first handler is reapplied
        assert_eq!(*test.borrow(), 3);
    }
}
