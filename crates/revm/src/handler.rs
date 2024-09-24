// Modules.
mod handle_types;
pub mod mainnet;
pub mod register;

// Exports.
pub use handle_types::*;

// Includes.

use crate::{Context, EvmWiring, Frame};
use core::mem;
use interpreter::{table::InstructionTables, Host, InterpreterAction, SharedMemory};
use register::{EvmHandler, HandleRegisters};
use specification::spec_to_generic;
use std::vec::Vec;
use wiring::{
    result::{EVMResultGeneric, InvalidTransaction},
    transaction::TransactionValidation,
};

use self::register::{HandleRegister, HandleRegisterBox};

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
    /// Validity handles.
    pub validation: ValidationHandler<'a, EvmWiringT>,
    /// Pre execution handle.
    pub pre_execution: PreExecutionHandler<'a, EvmWiringT>,
    /// Post Execution handle.
    pub post_execution: PostExecutionHandler<'a, EvmWiringT>,
    /// Execution loop that handles frames.
    pub execution: ExecutionHandler<'a, EvmWiringT>,
}

impl<'a, EvmWiringT> EvmHandler<'a, EvmWiringT>
where
    EvmWiringT:
        EvmWiring<Transaction: TransactionValidation<ValidationError: From<InvalidTransaction>>>,
{
    /// Creates a base/vanilla Ethereum handler with the provided spec id.
    pub fn mainnet_with_spec(spec_id: EvmWiringT::Hardfork) -> Self {
        spec_to_generic!(
            spec_id.into(),
            Self {
                spec_id,
                instruction_table: InstructionTables::new_plain::<SPEC>(),
                registers: Vec::new(),
                validation: ValidationHandler::new::<SPEC>(),
                pre_execution: PreExecutionHandler::new::<SPEC>(),
                post_execution: PostExecutionHandler::mainnet::<SPEC>(),
                execution: ExecutionHandler::new::<SPEC>(),
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
    pub fn execute_frame(
        &self,
        frame: &mut Frame,
        shared_memory: &mut SharedMemory,
        context: &mut Context<EvmWiringT>,
    ) -> EVMResultGeneric<InterpreterAction, EvmWiringT> {
        self.execution
            .execute_frame(frame, shared_memory, &self.instruction_table, context)
    }

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
    pub fn pre_execution(&self) -> &PreExecutionHandler<'a, EvmWiringT> {
        &self.pre_execution
    }

    /// Returns reference to pre execution handler.
    pub fn post_execution(&self) -> &PostExecutionHandler<'a, EvmWiringT> {
        &self.post_execution
    }

    /// Returns reference to frame handler.
    pub fn execution(&self) -> &ExecutionHandler<'a, EvmWiringT> {
        &self.execution
    }

    /// Returns reference to validation handler.
    pub fn validation(&self) -> &ValidationHandler<'a, EvmWiringT> {
        &self.validation
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
                h.post_execution.output = Arc::new(|_, _| Err(EVMError::Custom("test".into())))
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
