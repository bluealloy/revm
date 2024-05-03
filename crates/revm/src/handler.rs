// Modules.
pub mod cfg;
mod handle_types;
pub mod mainnet;
pub mod register;

// Exports.
pub use cfg::{CfgEnvWithHandlerCfg, EnvWithHandlerCfg, HandlerCfg};
pub use handle_types::*;

// Includes.
use crate::{
    chain_spec::{ChainSpec, MainnetChainSpec},
    interpreter::{opcode::InstructionTables, Host},
    primitives::{db::Database, spec_to_generic, EthSpecId, Spec},
    Evm, SpecId,
};
use register::{EvmHandler, HandleRegisters};
use std::vec::Vec;

use self::register::{HandleRegister, HandleRegisterBox};

/// Handler acts as a proxy and allow to define different behavior for different
/// sections of the code. This allows nice integration of different chains or
/// to disable some mainnet behavior.
pub struct Handler<'a, ChainSpecT: ChainSpec, H: Host + 'a, EXT, DB: Database> {
    /// Handler hardfork
    pub spec_id: ChainSpecT::Hardfork,
    /// Instruction table type.
    pub instruction_table: Option<InstructionTables<'a, H>>,
    /// Registers that will be called on initialization.
    pub registers: Vec<HandleRegisters<ChainSpecT, EXT, DB>>,
    /// Validity handles.
    pub validation: ValidationHandler<'a, EXT, DB>,
    /// Pre execution handle.
    pub pre_execution: PreExecutionHandler<'a, EXT, DB>,
    /// Post Execution handle.
    pub post_execution: PostExecutionHandler<'a, ChainSpecT, EXT, DB>,
    /// Execution loop that handles frames.
    pub execution: ExecutionHandler<'a, EXT, DB>,
}

impl<EXT, DB: Database> EvmHandler<'_, MainnetChainSpec, EXT, DB> {
    /// Creates handler with variable spec id, inside it will call `mainnet::<SPEC>` for
    /// appropriate spec.
    pub fn mainnet_with_spec(spec_id: EthSpecId) -> Self {
        Self::base_with_spec(spec_id)
    }
}

impl<'a, ChainSpecT: ChainSpec, EXT, DB: Database> EvmHandler<'a, ChainSpecT, EXT, DB> {
    fn base_with_spec(spec_id: ChainSpecT::Hardfork) -> Self {
        spec_to_generic!(
            spec_id.into(),
            Self {
                spec_id,
                instruction_table: Some(InstructionTables::new_plain::<SPEC>()),
                registers: Vec::new(),
                validation: ValidationHandler::new::<SPEC>(),
                pre_execution: PreExecutionHandler::new::<SPEC>(),
                post_execution: PostExecutionHandler::new::<SPEC>(),
                execution: ExecutionHandler::new::<SPEC>(),
            }
        )
    }

    /// Specification ID.
    pub fn spec_id(&self) -> ChainSpecT::Hardfork {
        self.spec_id
    }

    /// Take instruction table.
    pub fn take_instruction_table(
        &mut self,
    ) -> Option<InstructionTables<'a, Evm<'a, ChainSpecT, EXT, DB>>> {
        self.instruction_table.take()
    }

    /// Set instruction table.
    pub fn set_instruction_table(
        &mut self,
        table: InstructionTables<'a, Evm<'a, ChainSpecT, EXT, DB>>,
    ) {
        self.instruction_table = Some(table);
    }

    /// Returns reference to pre execution handler.
    pub fn pre_execution(&self) -> &PreExecutionHandler<'a, EXT, DB> {
        &self.pre_execution
    }

    /// Returns reference to pre execution handler.
    pub fn post_execution(&self) -> &PostExecutionHandler<'a, ChainSpecT, EXT, DB> {
        &self.post_execution
    }

    /// Returns reference to frame handler.
    pub fn execution(&self) -> &ExecutionHandler<'a, EXT, DB> {
        &self.execution
    }

    /// Returns reference to validation handler.
    pub fn validation(&self) -> &ValidationHandler<'a, EXT, DB> {
        &self.validation
    }

    /// Append handle register.
    pub fn append_handler_register(&mut self, register: HandleRegisters<ChainSpecT, EXT, DB>) {
        register.register(self);
        self.registers.push(register);
    }

    /// Append plain handle register.
    pub fn append_handler_register_plain(&mut self, register: HandleRegister<ChainSpecT, EXT, DB>) {
        register(self);
        self.registers.push(HandleRegisters::Plain(register));
    }

    /// Append boxed handle register.
    pub fn append_handler_register_box(
        &mut self,
        register: HandleRegisterBox<ChainSpecT, EXT, DB>,
    ) {
        register(self);
        self.registers.push(HandleRegisters::Box(register));
    }

    /// Pop last handle register and reapply all registers that are left.
    pub fn pop_handle_register(&mut self) -> Option<HandleRegisters<ChainSpecT, EXT, DB>> {
        let out = self.registers.pop();
        if out.is_some() {
            let registers = core::mem::take(&mut self.registers);
            let mut base_handler = Handler::mainnet_with_spec(self.spec_id);
            // apply all registers to default handler and raw mainnet instruction table.
            for register in registers {
                base_handler.append_handler_register(register)
            }
            *self = base_handler;
        }
        out
    }

    /// Creates the Handler with Generic Spec.
    pub fn create_handle_generic<SPEC: Spec>(&mut self) -> EvmHandler<'a, ChainSpecT, EXT, DB> {
        let registers = core::mem::take(&mut self.registers);
        let mut base_handler = Handler::mainnet::<SPEC>();
        // apply all registers to default handeler and raw mainnet instruction table.
        for register in registers {
            base_handler.append_handler_register(register)
        }
        base_handler
    }

    /// Creates the Handler with variable SpecId, inside it will call function with Generic Spec.
    pub fn modify_spec_id(&mut self, spec_id: ChainSpecT::Hardfork) {
        if self.spec_id == spec_id {
            return;
        }

        let registers = core::mem::take(&mut self.registers);
        // register for optimism is added as a register, so we need to create mainnet handler here.
        let mut handler = Handler::mainnet_with_spec(eth_spec_id);
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

    use crate::{db::EmptyDB, primitives::EVMError};
    use std::{rc::Rc, sync::Arc};

    use super::*;

    #[test]
    fn test_handler_register_pop() {
        let register = |inner: &Rc<RefCell<i32>>| -> HandleRegisterBox<(), EmptyDB> {
            let inner = inner.clone();
            Box::new(move |h| {
                *inner.borrow_mut() += 1;
                h.post_execution.output = Arc::new(|_, _| Err(EVMError::Custom("test".to_string())))
            })
        };

        #[cfg(not(feature = "optimism"))]
        let spec_id = SpecId::LATEST;
        #[cfg(feature = "optimism")]
        let spec_id = crate::optimism::OptimismSpecId::LATEST;

        let mut handler = EvmHandler::<(), EmptyDB>::new(HandlerCfg::new(spec_id));
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
