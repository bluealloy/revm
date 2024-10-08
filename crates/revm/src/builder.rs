use crate::{handler::register, Context, Evm, EvmContext, EvmWiring, Handler};
use core::marker::PhantomData;
use database_interface::EmptyDB;
use std::boxed::Box;
use transaction::Transaction;
use wiring::{
    default::{CfgEnv, EnvWiring},
    result::InvalidTransaction,
    EthereumWiring,
};

/// Evm Builder allows building or modifying EVM.
/// Note that some of the methods that changes underlying structures
/// will reset the registered handler to default mainnet.
pub struct EvmBuilder<'a, BuilderStage, EvmWiringT: EvmWiring> {
    database: Option<EvmWiringT::Database>,
    external_context: Option<EvmWiringT::ExternalContext>,
    env: Option<Box<EnvWiring<EvmWiringT>>>,
    /// Handler that will be used by EVM. It contains handle registers
    handler: Handler<'a, EvmWiringT, Context<EvmWiringT>>,
    /// Phantom data to mark the stage of the builder.
    phantom: PhantomData<BuilderStage>,
}

/// First stage of the builder allows setting generic variables.
/// Generic variables are database and external context.
pub struct SetGenericStage;

/// Second stage of the builder allows appending handler registers.
/// Requires the database and external context to be set.
pub struct HandlerStage;

impl<'a> Default for EvmBuilder<'a, SetGenericStage, EthereumWiring<EmptyDB, ()>> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, EvmWiringT: EvmWiring> EvmBuilder<'a, SetGenericStage, EvmWiringT>
where
    EvmWiringT::Transaction: Default,
    EvmWiringT::Block: Default,
{
    /// Sets the [`EvmWiring`] that will be used by [`Evm`].
    pub fn new() -> EvmBuilder<'a, SetGenericStage, EvmWiringT> {
        EvmBuilder {
            database: None,
            external_context: None,
            env: Some(Box::new(EnvWiring::<EvmWiringT>::default())),
            handler: EvmWiringT::handler::<'a>(EvmWiringT::Hardfork::default()),
            phantom: PhantomData,
        }
    }
}

impl<'a, EvmWiringT: EvmWiring> EvmBuilder<'a, SetGenericStage, EvmWiringT> {
    pub fn new_with(
        database: EvmWiringT::Database,
        external_context: EvmWiringT::ExternalContext,
        env: Box<EnvWiring<EvmWiringT>>,
        handler: Handler<'a, EvmWiringT, Context<EvmWiringT>>,
    ) -> EvmBuilder<'a, SetGenericStage, EvmWiringT> {
        EvmBuilder {
            database: Some(database),
            external_context: Some(external_context),
            env: Some(env),
            handler,
            phantom: PhantomData,
        }
    }

    pub fn with_wiring<NewEvmWiringT: EvmWiring>(
        self,
    ) -> EvmBuilder<'a, SetGenericStage, NewEvmWiringT>
    where
        NewEvmWiringT::Transaction: Default,
        NewEvmWiringT::Block: Default,
    {
        EvmBuilder {
            database: None,
            external_context: None,
            env: Some(Box::new(EnvWiring::<NewEvmWiringT>::default())),
            handler: NewEvmWiringT::handler::<'a>(NewEvmWiringT::Hardfork::default()),
            phantom: PhantomData,
        }
    }

    pub fn reset_handler_with_external_context<
        NewEvmWiringT: EvmWiring<
            Database = EvmWiringT::Database,
            Block = EvmWiringT::Block,
            Transaction = EvmWiringT::Transaction,
            Hardfork = EvmWiringT::Hardfork,
            HaltReason = EvmWiringT::HaltReason,
        >,
    >(
        self,
    ) -> EvmBuilder<'a, SetGenericStage, NewEvmWiringT> {
        EvmBuilder {
            database: self.database,
            external_context: None,
            env: self.env,
            // Handler that will be used by EVM. It contains handle registers
            handler: NewEvmWiringT::handler::<'a>(NewEvmWiringT::Hardfork::default()),
            phantom: PhantomData,
        }
    }

    pub fn reset_new_database<
        NewEvmWiringT: EvmWiring<
            ExternalContext = EvmWiringT::ExternalContext,
            Block = EvmWiringT::Block,
            Transaction = EvmWiringT::Transaction,
            Hardfork = EvmWiringT::Hardfork,
            HaltReason = EvmWiringT::HaltReason,
        >,
    >(
        self,
    ) -> EvmBuilder<'a, SetGenericStage, NewEvmWiringT> {
        EvmBuilder {
            database: None,
            external_context: self.external_context,
            env: self.env,
            // Handler that will be used by EVM. It contains handle registers
            handler: NewEvmWiringT::handler::<'a>(NewEvmWiringT::Hardfork::default()),
            phantom: PhantomData,
        }
    }
}

impl<'a, EvmWiringT> EvmBuilder<'a, SetGenericStage, EvmWiringT>
where
    EvmWiringT: EvmWiring<Transaction: Transaction<TransactionError: From<InvalidTransaction>>>,
{
    /// Creates the default [EvmWiring]::[crate::Database] that will be used by [`Evm`].
    pub fn with_default_db(mut self) -> EvmBuilder<'a, SetGenericStage, EvmWiringT>
    where
        EvmWiringT::Database: Default,
    {
        self.database = Some(EvmWiringT::Database::default());
        self
    }

    pub fn with_default_ext_ctx(mut self) -> EvmBuilder<'a, SetGenericStage, EvmWiringT>
    where
        EvmWiringT::ExternalContext: Default,
    {
        self.external_context = Some(EvmWiringT::ExternalContext::default());
        self
    }

    /// Sets the [`crate::Database`] that will be used by [`Evm`].
    pub fn with_db(
        mut self,
        db: EvmWiringT::Database,
    ) -> EvmBuilder<'a, SetGenericStage, EvmWiringT> {
        self.database = Some(db);
        self
    }

    /// Sets the external context that will be used by [`Evm`].
    pub fn with_external_context(
        mut self,
        external_context: EvmWiringT::ExternalContext,
    ) -> EvmBuilder<'a, SetGenericStage, EvmWiringT> {
        self.external_context = Some(external_context);
        self
    }

    // /// Sets Builder with [`EnvWithEvmWiring`].
    // pub fn with_env_with_handler_cfg(
    //     mut self,
    //     env_with_handler_cfg: EnvWithEvmWiring<EvmWiringT>,
    // ) -> EvmBuilder<'a, HandlerStage, EvmWiringT> {
    //     let EnvWithEvmWiring { env, spec_id } = env_with_handler_cfg;
    //     self.context.evm.env = env;
    //     EvmBuilder {
    //         context: self.context,
    //         handler: EvmWiringT::handler::<'a, EXT, DB>(spec_id),
    //         phantom: PhantomData,
    //     }
    // }

    // /// Sets Builder with [`ContextWithEvmWiring`].
    // pub fn with_context_with_handler_cfg<OEXT, ODB: Database>(
    //     self,
    //     context_with_handler_cfg: ContextWithEvmWiring<EvmWiringT, OEXT, ODB>,
    // ) -> EvmBuilder<'a, HandlerStage, EvmWiringT, OEXT, ODB> {
    //     EvmBuilder {
    //         context: context_with_handler_cfg.context,
    //         handler: EvmWiringT::handler::<'a, OEXT, ODB>(context_with_handler_cfg.spec_id),
    //         phantom: PhantomData,
    //     }
    // }

    // /// Sets Builder with [`CfgEnvWithEvmWiring`].
    // pub fn with_cfg_env_with_handler_cfg(
    //     mut self,
    //     cfg_env_and_spec_id: CfgEnvWithEvmWiring<EvmWiringT>,
    // ) -> EvmBuilder<'a, HandlerStage, EvmWiringT> {
    //     self.context.evm.env.cfg = cfg_env_and_spec_id.cfg_env;

    //     EvmBuilder {
    //         context: self.context,
    //         handler: EvmWiringT::handler::<'a>(cfg_env_and_spec_id.spec_id),
    //         phantom: PhantomData,
    //     }
    // }
}

impl<'a, EvmWiringT: EvmWiring> EvmBuilder<'a, HandlerStage, EvmWiringT> {
    //     /// Creates new builder from Evm, Evm is consumed and all field are moved to Builder.
    //     /// It will preserve set handler and context.
    //     ///
    //     /// Builder is in HandlerStage and both database and external are set.
    //     pub fn new(evm: Evm<'a, EvmWiringT>) -> Self {
    //         Self {
    //             context: evm.context,
    //             handler: evm.handler,
    //             phantom: PhantomData,
    //         }
    //     }
    // }

    // impl<'a, EvmWiringT: EvmWiring> EvmBuilder<'a, HandlerStage, EvmWiringT>
    // where
    //     EvmWiringT:
    //         EvmWiring<Transaction: TransactionValidation<ValidationError: From<InvalidTransaction>>>,
    // {
    //     /// Sets the [`EmptyDB`] and resets the [`Handler`] to default mainnet.
    //     pub fn reset_handler_with_empty_db(self) -> EvmBuilder<'a, HandlerStage, EvmWiringT> {
    //         EvmBuilder {
    //             context: Context::new(
    //                 self.context.evm.with_db(EmptyDB::default()),
    //                 self.context.external,
    //             ),
    //             handler: EvmWiringT::handler::<'a>(self.handler.spec_id()),
    //             phantom: PhantomData,
    //         }
    //     }

    //     /// Sets the [`Database`] that will be used by [`Evm`]
    //     /// and resets the [`Handler`] to default mainnet.
    //     pub fn reset_handler_with_db<ODB: Database>(
    //         self,
    //         db: ODB,
    //     ) -> EvmBuilder<'a, SetGenericStage, EvmWiringT, EXT, ODB> {
    //         EvmBuilder {
    //             context: Context::new(self.context.evm.with_db(db), self.context.external),
    //             handler: EvmWiringT::handler::<'a, EXT, ODB>(self.handler.spec_id()),
    //             phantom: PhantomData,
    //         }
    //     }

    //     /// Resets [`Handler`] and sets the [`DatabaseRef`] that will be used by [`Evm`]
    //     /// and resets the [`Handler`] to default mainnet.
    //     pub fn reset_handler_with_ref_db<ODB: DatabaseRef>(
    //         self,
    //         db: ODB,
    //     ) -> EvmBuilder<'a, SetGenericStage, EvmWiringT, EXT, WrapDatabaseRef<ODB>> {
    //         EvmBuilder {
    //             context: Context::new(
    //                 self.context.evm.with_db(WrapDatabaseRef(db)),
    //                 self.context.external,
    //             ),
    //             handler: EvmWiringT::handler::<'a, EXT, WrapDatabaseRef<ODB>>(self.handler.spec_id()),
    //             phantom: PhantomData,
    //         }
    //     }

    //     /// Resets [`Handler`] and sets new `ExternalContext` type.
    //     ///  and resets the [`Handler`] to default mainnet.
    //     pub fn reset_handler_with_external_context<OEXT>(
    //         self,
    //         external: OEXT,
    //     ) -> EvmBuilder<'a, SetGenericStage, EvmWiringT, OEXT, DB> {
    //         EvmBuilder {
    //             context: Context::new(self.context.evm, external),
    //             handler: EvmWiringT::handler::<'a, OEXT, DB>(self.handler.spec_id()),
    //             phantom: PhantomData,
    //         }
    //     }
}

impl<'a, BuilderStage, EvmWiringT: EvmWiring> EvmBuilder<'a, BuilderStage, EvmWiringT> {
    /// This modifies the [EvmBuilder] to make it easy to construct an [`Evm`] with a _specific_
    /// handler.
    ///
    /// # Example
    /// ```rust
    /// use revm::{EvmBuilder, EvmHandler};
    /// use wiring::EthereumWiring;
    /// use database_interface::EmptyDB;
    /// use specification::hardfork::{SpecId,CancunSpec};
    ///
    /// let builder = EvmBuilder::default().with_default_db().with_default_ext_ctx();
    ///
    /// // get the desired handler
    /// let mainnet = EvmHandler::<'_, EthereumWiring<EmptyDB,()>>::mainnet_with_spec(SpecId::CANCUN);
    /// let builder = builder.with_handler(mainnet);
    ///
    /// // build the EVM
    /// let evm = builder.build();
    /// ```
    pub fn with_handler(
        mut self,
        handler: Handler<'a, EvmWiringT, Context<EvmWiringT>>,
    ) -> EvmBuilder<'a, BuilderStage, EvmWiringT> {
        self.handler = handler;
        self
    }

    /// Builds the [`Evm`].
    pub fn build(self) -> Evm<'a, EvmWiringT> {
        Evm::new(
            Context::new(
                EvmContext::new_with_env(self.database.unwrap(), self.env.unwrap()),
                self.external_context.unwrap(),
            ),
            self.handler,
        )
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    ///
    /// When called, EvmBuilder will transition from SetGenericStage to HandlerStage.
    pub fn append_handler_register(
        mut self,
        handle_register: register::HandleRegister<EvmWiringT>,
    ) -> EvmBuilder<'a, BuilderStage, EvmWiringT> {
        self.handler
            .append_handler_register(register::HandleRegisters::Plain(handle_register));
        self
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    ///
    /// When called, EvmBuilder will transition from SetGenericStage to HandlerStage.
    pub fn append_handler_register_box(
        mut self,
        handle_register: register::HandleRegisterBox<'a, EvmWiringT>,
    ) -> EvmBuilder<'a, BuilderStage, EvmWiringT> {
        self.handler
            .append_handler_register(register::HandleRegisters::Box(handle_register));
        self
    }

    /// Allows modification of Evm Database.
    pub fn modify_db(mut self, f: impl FnOnce(&mut EvmWiringT::Database)) -> Self {
        f(self.database.as_mut().unwrap());
        self
    }

    /// Allows modification of external context.
    pub fn modify_external_context(
        mut self,
        f: impl FnOnce(&mut EvmWiringT::ExternalContext),
    ) -> Self {
        f(self.external_context.as_mut().unwrap());
        self
    }

    /// Allows modification of Evm Environment.
    pub fn modify_env(mut self, f: impl FnOnce(&mut Box<EnvWiring<EvmWiringT>>)) -> Self {
        f(self.env.as_mut().unwrap());
        self
    }

    /// Sets Evm Environment.
    pub fn with_env(mut self, env: Box<EnvWiring<EvmWiringT>>) -> Self {
        self.env = Some(env);
        self
    }

    /// Allows modification of Evm's Transaction Environment.
    pub fn modify_tx_env(mut self, f: impl FnOnce(&mut EvmWiringT::Transaction)) -> Self {
        f(&mut self.env.as_mut().unwrap().tx);
        self
    }

    /// Sets Evm's Transaction Environment.
    pub fn with_tx_env(mut self, tx_env: EvmWiringT::Transaction) -> Self {
        self.env.as_mut().unwrap().tx = tx_env;
        self
    }

    /// Allows modification of Evm's Block Environment.
    pub fn modify_block_env(mut self, f: impl FnOnce(&mut EvmWiringT::Block)) -> Self {
        f(&mut self.env.as_mut().unwrap().block);
        self
    }

    /// Sets Evm's Block Environment.
    pub fn with_block_env(mut self, block_env: EvmWiringT::Block) -> Self {
        self.env.as_mut().unwrap().block = block_env;
        self
    }

    /// Allows modification of Evm's Config Environment.
    pub fn modify_cfg_env(mut self, f: impl FnOnce(&mut CfgEnv)) -> Self {
        f(&mut self.env.as_mut().unwrap().cfg);
        self
    }
}

impl<'a, BuilderStage, EvmWiringT> EvmBuilder<'a, BuilderStage, EvmWiringT>
where
    EvmWiringT: EvmWiring<Block: Default>,
{
    /// Clears Block environment of EVM.
    pub fn with_clear_block_env(mut self) -> Self {
        self.env.as_mut().unwrap().block = EvmWiringT::Block::default();
        self
    }
}

impl<'a, BuilderStage, EvmWiringT> EvmBuilder<'a, BuilderStage, EvmWiringT>
where
    EvmWiringT: EvmWiring<Transaction: Default>,
{
    /// Clears Transaction environment of EVM.
    pub fn with_clear_tx_env(mut self) -> Self {
        self.env.as_mut().unwrap().tx = EvmWiringT::Transaction::default();
        self
    }
}

impl<'a, BuilderStage, EvmWiringT> EvmBuilder<'a, BuilderStage, EvmWiringT>
where
    EvmWiringT: EvmWiring<Block: Default, Transaction: Default>,
{
    /// Clears Environment of EVM.
    pub fn with_clear_env(mut self) -> Self {
        self.env.as_mut().unwrap().clear();
        self
    }
}

impl<'a, BuilderStage, EvmWiringT: EvmWiring> EvmBuilder<'a, BuilderStage, EvmWiringT>
where
    EvmWiringT: EvmWiring<Transaction: Transaction<TransactionError: From<InvalidTransaction>>>,
{
    /// Sets specification Id , that will mark the version of EVM.
    /// It represent the hard fork of ethereum.
    ///
    /// # Note
    ///
    /// When changed it will reapply all handle registers, this can be
    /// expensive operation depending on registers.
    pub fn with_spec_id(mut self, spec_id: EvmWiringT::Hardfork) -> Self {
        self.handler.modify_spec_id(spec_id);
        self
    }

    /// Resets [`Handler`] to default mainnet.
    pub fn reset_handler(mut self) -> Self {
        self.handler = EvmWiringT::handler::<'a>(self.handler.spec_id());
        self
    }
}

#[cfg(test)]
mod test {
    use crate::{Context, Evm};
    use bytecode::Bytecode;
    use database::InMemoryDB;
    use interpreter::Interpreter;
    use primitives::{address, TxKind, U256};
    use state::AccountInfo;
    use std::{cell::RefCell, rc::Rc};
    use wiring::EthereumWiring;

    /// Custom evm context
    #[derive(Default, Clone, Debug)]
    pub(crate) struct CustomContext {
        pub(crate) inner: Rc<RefCell<u8>>,
    }

    #[test]
    fn simple_add_stateful_instruction() {
        let code = Bytecode::new_raw([0xED, 0x00].into());
        let code_hash = code.hash_slow();
        let to_addr = address!("ffffffffffffffffffffffffffffffffffffffff");

        // initialize the custom context and make sure it's zero
        let custom_context = CustomContext::default();
        assert_eq!(*custom_context.inner.borrow(), 0);

        let to_capture = custom_context.clone();
        let mut evm = Evm::<EthereumWiring<InMemoryDB, ()>>::builder()
            .with_default_db()
            .with_default_ext_ctx()
            .modify_db(|db| {
                db.insert_account_info(
                    to_addr,
                    AccountInfo::new(U256::from(1_000_000), 0, code_hash, code),
                )
            })
            .modify_tx_env(|tx| {
                tx.transact_to = TxKind::Call(to_addr);
                tx.gas_limit = 100_000;
            })
            // we need to use handle register box to capture the custom context in the handle
            // register
            .append_handler_register_box(Box::new(move |handler| {
                let custom_context = to_capture.clone();

                // we need to use a box to capture the custom context in the instruction
                let custom_instruction =
                    Box::new(move |_interp: &mut Interpreter, _host: &mut Context<_>| {
                        // modify the value
                        let mut inner = custom_context.inner.borrow_mut();
                        *inner += 1;
                    });

                // need to  ensure the instruction table is a boxed instruction table so that we
                // can insert the custom instruction as a boxed instruction
                handler
                    .instruction_table
                    .insert_boxed(0xED, custom_instruction);
            }))
            .build();

        let _result_and_state = evm.transact().unwrap();

        // ensure the custom context was modified
        assert_eq!(*custom_context.inner.borrow(), 1);
    }

    // #[test]
    // fn simple_add_instruction() {
    //     const CUSTOM_INSTRUCTION_COST: u64 = 133;
    //     const INITIAL_TX_GAS: u64 = 21000;
    //     const EXPECTED_RESULT_GAS: u64 = INITIAL_TX_GAS + CUSTOM_INSTRUCTION_COST;

    //     fn custom_instruction(interp: &mut Interpreter, _host: &mut impl Host) {
    //         // just spend some gas
    //         gas!(interp, CUSTOM_INSTRUCTION_COST);
    //     }

    //     let code = Bytecode::new_raw([0xED, 0x00].into());
    //     let code_hash = code.hash_slow();
    //     let to_addr = address!("ffffffffffffffffffffffffffffffffffffffff");

    //     let mut evm = Evm::builder()
    //         .with_wiring::<EthereumWiring<InMemoryDB, ()>>()
    //         .with_db(InMemoryDB::default())
    //         .modify_db(|db| {
    //             db.insert_account_info(to_addr, AccountInfo::new(U256::ZERO, 0, code_hash, code))
    //         })
    //         .modify_tx_env(|tx| {
    //             let transact_to = &mut tx.transact_to;

    //             *transact_to = TxKind::Call(to_addr)
    //         })
    //         .append_handler_register(|handler| {
    //             handler.instruction_table.insert(0xED, custom_instruction)
    //         })
    //         .build();

    //     let result_and_state = evm.transact().unwrap();
    //     assert_eq!(result_and_state.result.gas_used(), EXPECTED_RESULT_GAS);
    // }

    // #[test]
    // fn simple_build() {
    //     // build without external with latest spec
    //     Evm::builder().with_chain_spec::<TestEvmWiring>().build();
    //     // build with empty db
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_empty_db()
    //         .build();
    //     // build with_db
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_db(EmptyDB::default())
    //         .build();
    //     // build with empty external
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_empty_db()
    //         .build();
    //     // build with some external
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_empty_db()
    //         .with_external_context(())
    //         .build();
    //     // build with spec
    //     Evm::builder()
    //         .with_empty_db()
    //         .with_spec_id(SpecId::HOMESTEAD)
    //         .build();

    //     // with with Env change in multiple places
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_empty_db()
    //         .modify_tx_env(|tx| tx.gas_limit = 10)
    //         .build();
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .modify_tx_env(|tx| tx.gas_limit = 10)
    //         .build();
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_empty_db()
    //         .modify_tx_env(|tx| tx.gas_limit = 10)
    //         .build();
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_empty_db()
    //         .modify_tx_env(|tx| tx.gas_limit = 10)
    //         .build();

    //     // with inspector handle
    //     Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_empty_db()
    //         .with_external_context(NoOpInspector)
    //         .append_handler_register(inspector_handle_register)
    //         .build();

    //     // create the builder
    //     let evm = Evm::builder()
    //         .with_db(EmptyDB::default())
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_external_context(NoOpInspector)
    //         .append_handler_register(inspector_handle_register)
    //         // this would not compile
    //         // .with_db(..)
    //         .build();

    //     let Context { external: _, .. } = evm.into_context();
    // }

    // #[test]
    // fn build_modify_build() {
    //     // build evm
    //     let evm = Evm::builder()
    //         .with_empty_db()
    //         .with_spec_id(SpecId::HOMESTEAD)
    //         .build();

    //     // modify evm
    //     let evm = evm.modify().with_spec_id(SpecId::FRONTIER).build();
    //     let _ = evm
    //         .modify()
    //         .modify_tx_env(|tx| tx.chain_id = Some(2))
    //         .build();
    // }

    // #[test]
    // fn build_custom_precompile() {
    //     struct CustomPrecompile;

    //     impl ContextStatefulPrecompile<TestEvmWiring> for CustomPrecompile {
    //         fn call(
    //             &self,
    //             _input: &Bytes,
    //             _gas_limit: u64,
    //             _context: &mut InnerEvmContext<TestEvmWiring>,
    //         ) -> PrecompileResult {
    //             Ok(PrecompileOutput::new(10, Bytes::new()))
    //         }
    //     }

    //     let spec_id = crate::primitives::SpecId::HOMESTEAD;

    //     let mut evm = Evm::builder()
    //         .with_chain_spec::<TestEvmWiring>()
    //         .with_spec_id(spec_id)
    //         .append_handler_register(|handler| {
    //             let precompiles = handler.pre_execution.load_precompiles();
    //             handler.pre_execution.load_precompiles = Arc::new(move || {
    //                 let mut precompiles = precompiles.clone();
    //                 precompiles.extend([(
    //                     Address::ZERO,
    //                     ContextPrecompile::ContextStateful(Arc::new(CustomPrecompile)),
    //                 )]);
    //                 precompiles
    //             });
    //         })
    //         .build();

    //     evm.transact().unwrap();
    // }
}
