use crate::{
    db::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef},
    evm::Evm,
    handler::register,
    primitives::{
        BlockEnv,
        CfgEnv,
        CfgEnvWithHandlerCfg,
        Env,
        EnvWithHandlerCfg,
        HandlerCfg,
        SpecId,
        TxEnv,
    },
    Context,
    ContextWithHandlerCfg,
    Handler,
};
use core::marker::PhantomData;
use std::boxed::Box;

pub struct EvmFactoryEvm;

/// Evm Builder allows building or modifying EVM.
/// Note that some of the methods that change underlying structures
/// will reset the registered handler to default mainnet.
pub struct UniversalBuilder<'a, BuilderStage, EXT, DB: Database, EVM> {
    context: Context<EXT, DB>,
    /// Handler that EVM will use. It contains handle registers
    handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
    /// Phantom data to mark the stage of the builder.
    phantom1: PhantomData<BuilderStage>,
    phantom2: PhantomData<EVM>,
}

pub type EvmBuilder<'a, BuilderStage, EXT, DB> =
    UniversalBuilder<'a, BuilderStage, EXT, DB, EvmFactoryEvm>;

/// The first stage of the builder allows setting generic variables.
/// Generic variables are database and external context.
pub struct SetGenericStage;

/// The second stage of the builder allows appending handler registers.
/// Requires the database and external context to be set.
pub struct HandlerStage;

impl<'a, EVM> Default for UniversalBuilder<'a, SetGenericStage, (), EmptyDB, EVM> {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "optimism-default-handler",
                not(feature = "negate-optimism-default-handler")))] {
                    let mut handler_cfg = HandlerCfg::new(SpecId::LATEST);
                    // set is_optimism to true by default.
                    handler_cfg.is_optimism = true;
            } else {
                let handler_cfg = HandlerCfg::new(SpecId::LATEST);
            }
        }
        Self {
            context: Context::default(),
            handler: UniversalBuilder::<'a, SetGenericStage, (), EmptyDB, EVM>::handler(
                handler_cfg,
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
}

impl<'a, EXT, DB: Database, EVM> UniversalBuilder<'a, SetGenericStage, EXT, DB, EVM> {
    /// Sets the [`EmptyDB`] as the [`Database`] that will be used by [`Evm`].
    pub fn with_empty_db(self) -> UniversalBuilder<'a, SetGenericStage, EXT, EmptyDB, EVM> {
        UniversalBuilder {
            context: Context::new(
                self.context.evm.with_db(EmptyDB::default()),
                self.context.external,
            ),
            handler: UniversalBuilder::<'a, SetGenericStage, EXT, EmptyDB, EVM>::handler(
                self.handler.cfg(),
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
    /// Sets the [`Database`] that will be used by [`Evm`].
    pub fn with_db<ODB: Database>(
        self,
        db: ODB,
    ) -> UniversalBuilder<'a, SetGenericStage, EXT, ODB, EVM> {
        UniversalBuilder {
            context: Context::new(self.context.evm.with_db(db), self.context.external),
            handler: UniversalBuilder::<'a, SetGenericStage, EXT, ODB, EVM>::handler(
                self.handler.cfg(),
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
    /// Sets the [`DatabaseRef`] that will be used by [`Evm`].
    pub fn with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> UniversalBuilder<'a, SetGenericStage, EXT, WrapDatabaseRef<ODB>, EVM> {
        UniversalBuilder {
            context: Context::new(
                self.context.evm.with_db(WrapDatabaseRef(db)),
                self.context.external,
            ),
            handler:
                UniversalBuilder::<'a, SetGenericStage, EXT, WrapDatabaseRef<ODB>, EVM>::handler(
                    self.handler.cfg(),
                ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets the external context that will be used by [`Evm`].
    pub fn with_external_context<OEXT>(
        self,
        external: OEXT,
    ) -> UniversalBuilder<'a, SetGenericStage, OEXT, DB, EVM> {
        UniversalBuilder {
            context: Context::new(self.context.evm, external),
            handler: UniversalBuilder::<'a, SetGenericStage, OEXT, DB, EVM>::handler(
                self.handler.cfg(),
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets Builder with [`EnvWithHandlerCfg`].
    pub fn with_env_with_handler_cfg(
        mut self,
        env_with_handler_cfg: EnvWithHandlerCfg,
    ) -> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
        let EnvWithHandlerCfg { env, handler_cfg } = env_with_handler_cfg;
        self.context.evm.env = env;
        UniversalBuilder {
            context: self.context,
            handler: UniversalBuilder::<'a, HandlerStage, EXT, DB, EVM>::handler(handler_cfg),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets Builder with [`ContextWithHandlerCfg`].
    pub fn with_context_with_handler_cfg<OEXT, ODB: Database>(
        self,
        context_with_handler_cfg: ContextWithHandlerCfg<OEXT, ODB>,
    ) -> UniversalBuilder<'a, HandlerStage, OEXT, ODB, EVM> {
        UniversalBuilder {
            context: context_with_handler_cfg.context,
            handler: UniversalBuilder::<'a, HandlerStage, OEXT, ODB, EVM>::handler(
                context_with_handler_cfg.cfg,
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets Builder with [`CfgEnvWithHandlerCfg`].
    pub fn with_cfg_env_with_handler_cfg(
        mut self,
        cfg_env_and_spec_id: CfgEnvWithHandlerCfg,
    ) -> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
        self.context.evm.env.cfg = cfg_env_and_spec_id.cfg_env;

        UniversalBuilder {
            context: self.context,
            handler: UniversalBuilder::<'a, HandlerStage, EXT, DB, EVM>::handler(
                cfg_env_and_spec_id.handler_cfg,
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets Builder with [`HandlerCfg`]
    pub fn with_handler_cfg(
        self,
        handler_cfg: HandlerCfg,
    ) -> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
        UniversalBuilder {
            context: self.context,
            handler: UniversalBuilder::<'a, HandlerStage, EXT, DB, EVM>::handler(handler_cfg),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets the Optimism handler with latest spec.
    ///
    /// If `optimism-default-handler` feature is enabled this is not needed.
    #[cfg(feature = "optimism")]
    pub fn optimism(mut self) -> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
        self.handler = Handler::optimism_with_spec(self.handler.cfg.spec_id);
        UniversalBuilder {
            context: self.context,
            handler: self.handler,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets the mainnet handler with latest spec.
    ///
    /// Enabled only with `optimism-default-handler` feature.
    #[cfg(feature = "optimism-default-handler")]
    pub fn mainnet(mut self) -> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
        self.handler = Handler::mainnet_with_spec(self.handler.cfg.spec_id);
        UniversalBuilder {
            context: self.context,
            handler: self.handler,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
}

impl<'a, BuilderStage, EXT, DB: Database>
    UniversalBuilder<'a, BuilderStage, EXT, DB, EvmFactoryEvm>
{
    /// Creates new builder from Evm, Evm is consumed, and all fields are moved to Builder.
    /// It will preserve set handler and context.
    ///
    /// Builder is in HandlerStage and both database and external are set.
    pub fn new(evm: Evm<'a, EXT, DB>) -> Self {
        Self {
            context: evm.context,
            handler: evm.handler,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    pub fn build(self) -> Evm<'a, EXT, DB> {
        Evm::new(self.context, self.handler)
    }
}

impl<'a, EXT, DB: Database, EVM> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
    /// Sets the [`EmptyDB`] and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_empty_db(
        self,
    ) -> UniversalBuilder<'a, HandlerStage, EXT, EmptyDB, EVM> {
        UniversalBuilder {
            context: Context::new(
                self.context.evm.with_db(EmptyDB::default()),
                self.context.external,
            ),
            handler: UniversalBuilder::<'a, HandlerStage, EXT, EmptyDB, EVM>::handler(
                self.handler.cfg(),
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Resets the [`Handler`] and sets base mainnet handler.
    ///
    /// Enabled only with `optimism-default-handler` feature.
    #[cfg(feature = "optimism-default-handler")]
    pub fn reset_handler_with_mainnet(
        mut self,
    ) -> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
        self.handler = Handler::mainnet_with_spec(self.handler.cfg.spec_id);
        UniversalBuilder {
            context: self.context,
            handler: self.handler,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets the [`Database`] that will be used by [`Evm`]
    /// and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_db<ODB: Database>(
        self,
        db: ODB,
    ) -> UniversalBuilder<'a, SetGenericStage, EXT, ODB, EVM> {
        UniversalBuilder {
            context: Context::new(self.context.evm.with_db(db), self.context.external),
            handler: UniversalBuilder::<'a, SetGenericStage, EXT, ODB, EVM>::handler(
                self.handler.cfg(),
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Resets [`Handler`] and sets the [`DatabaseRef`] that will be used by [`Evm`]
    /// and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> UniversalBuilder<'a, SetGenericStage, EXT, WrapDatabaseRef<ODB>, EVM> {
        UniversalBuilder {
            context: Context::new(
                self.context.evm.with_db(WrapDatabaseRef(db)),
                self.context.external,
            ),
            handler:
                UniversalBuilder::<'a, SetGenericStage, EXT, WrapDatabaseRef<ODB>, EVM>::handler(
                    self.handler.cfg(),
                ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Resets [`Handler`] and sets a new `ExternalContext` type,
    /// and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_external_context<OEXT>(
        self,
        external: OEXT,
    ) -> UniversalBuilder<'a, SetGenericStage, OEXT, DB, EVM> {
        UniversalBuilder {
            context: Context::new(self.context.evm, external),
            handler: UniversalBuilder::<'a, SetGenericStage, OEXT, DB, EVM>::handler(
                self.handler.cfg(),
            ),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }
}

impl<'a, BuilderStage, EXT, DB: Database, EVM> UniversalBuilder<'a, BuilderStage, EXT, DB, EVM> {
    /// Creates the default handler.
    ///
    /// This is useful for adding optimism handle register.
    fn handler(handler_cfg: HandlerCfg) -> Handler<'a, Context<EXT, DB>, EXT, DB> {
        Handler::new(handler_cfg)
    }

    /// This modifies the [UniversalBuilder] to make it easy to construct an [`Evm`] with a
    /// _specific_ handler.
    ///
    /// # Example
    /// ```rust
    /// use revm::{EvmBuilder, Handler, primitives::{SpecId, HandlerCfg}};
    /// use revm_interpreter::primitives::CancunSpec;
    /// let builder = EvmBuilder::default();
    ///
    /// // get the desired handler
    /// let mainnet = Handler::mainnet::<CancunSpec>();
    /// let builder = builder.with_handler(mainnet);
    ///
    /// // build the EVM
    /// let evm = builder.build();
    /// ```
    pub fn with_handler(
        self,
        handler: Handler<'a, Context<EXT, DB>, EXT, DB>,
    ) -> UniversalBuilder<'a, BuilderStage, EXT, DB, EVM> {
        UniversalBuilder {
            context: self.context,
            handler,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    ///
    /// When called, EvmBuilder will transition from SetGenericStage to HandlerStage.
    pub fn append_handler_register(
        mut self,
        handle_register: register::HandleRegister<EXT, DB>,
    ) -> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
        self.handler
            .append_handler_register(register::HandleRegisters::Plain(handle_register));
        UniversalBuilder {
            context: self.context,
            handler: self.handler,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    ///
    /// When called, EvmBuilder will transition from SetGenericStage to HandlerStage.
    pub fn append_handler_register_box(
        mut self,
        handle_register: register::HandleRegisterBox<'a, EXT, DB>,
    ) -> UniversalBuilder<'a, HandlerStage, EXT, DB, EVM> {
        self.handler
            .append_handler_register(register::HandleRegisters::Box(handle_register));
        UniversalBuilder {
            context: self.context,
            handler: self.handler,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Sets specification id, that will mark the version of EVM.
    /// It represents the hard fork of ethereum.
    ///
    /// # Note
    ///
    /// When changed it will reapply all handle registers, this can be
    /// an expensive operation depending on registers.
    pub fn with_spec_id(mut self, spec_id: SpecId) -> Self {
        self.handler.modify_spec_id(spec_id);
        UniversalBuilder {
            context: self.context,
            handler: self.handler,
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    /// Allows modification of Evm Database.
    pub fn modify_db(mut self, f: impl FnOnce(&mut DB)) -> Self {
        f(&mut self.context.evm.db);
        self
    }

    /// Allows modification of external context.
    pub fn modify_external_context(mut self, f: impl FnOnce(&mut EXT)) -> Self {
        f(&mut self.context.external);
        self
    }

    /// Allows modification of Evm Environment.
    pub fn modify_env(mut self, f: impl FnOnce(&mut Box<Env>)) -> Self {
        f(&mut self.context.evm.env);
        self
    }

    /// Sets Evm Environment.
    pub fn with_env(mut self, env: Box<Env>) -> Self {
        self.context.evm.env = env;
        self
    }

    /// Allows modification of Evm's Transaction Environment.
    pub fn modify_tx_env(mut self, f: impl FnOnce(&mut TxEnv)) -> Self {
        f(&mut self.context.evm.env.tx);
        self
    }

    /// Sets Evm's Transaction Environment.
    pub fn with_tx_env(mut self, tx_env: TxEnv) -> Self {
        self.context.evm.env.tx = tx_env;
        self
    }

    /// Allows modification of Evm's Block Environment.
    pub fn modify_block_env(mut self, f: impl FnOnce(&mut BlockEnv)) -> Self {
        f(&mut self.context.evm.env.block);
        self
    }

    /// Sets Evm's Block Environment.
    pub fn with_block_env(mut self, block_env: BlockEnv) -> Self {
        self.context.evm.env.block = block_env;
        self
    }

    /// Allows modification of Evm's Config Environment.
    pub fn modify_cfg_env(mut self, f: impl FnOnce(&mut CfgEnv)) -> Self {
        f(&mut self.context.evm.env.cfg);
        self
    }

    /// Clears Environment of EVM.
    pub fn with_clear_env(mut self) -> Self {
        self.context.evm.env.clear();
        self
    }

    /// Clears Transaction environment of EVM.
    pub fn with_clear_tx_env(mut self) -> Self {
        self.context.evm.env.tx.clear();
        self
    }
    /// Clears Block environment of EVM.
    pub fn with_clear_block_env(mut self) -> Self {
        self.context.evm.env.block.clear();
        self
    }

    /// Resets [`Handler`] to default mainnet.
    pub fn reset_handler(mut self) -> Self {
        self.handler = Self::handler(self.handler.cfg());
        self
    }
}

#[cfg(test)]
mod test {
    use super::SpecId;
    use crate::{
        db::EmptyDB,
        inspector::inspector_handle_register,
        inspectors::NoOpInspector,
        primitives::{
            address,
            AccountInfo,
            Address,
            Bytecode,
            Bytes,
            PrecompileResult,
            TxKind,
            U256,
        },
        Context,
        ContextPrecompile,
        ContextStatefulPrecompile,
        Evm,
        InMemoryDB,
        InnerEvmContext,
    };
    use revm_interpreter::{gas, Host, Interpreter};
    use revm_precompile::PrecompileOutput;
    use std::{cell::RefCell, rc::Rc, sync::Arc};

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
        let mut evm = Evm::builder()
            .with_db(InMemoryDB::default())
            .modify_db(|db| {
                db.insert_account_info(to_addr, AccountInfo::new(U256::ZERO, 0, code_hash, code))
            })
            .modify_tx_env(|tx| tx.transact_to = TxKind::Call(to_addr))
            // we need to use handle register box to capture the custom context in the handle
            // register
            .append_handler_register_box(Box::new(move |handler| {
                let custom_context = to_capture.clone();

                // we need to use a box to capture the custom context in the instruction
                let custom_instruction = Box::new(
                    move |_interp: &mut Interpreter, _host: &mut Context<(), InMemoryDB>| {
                        // modify the value
                        let mut inner = custom_context.inner.borrow_mut();
                        *inner += 1;
                    },
                );

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

    #[test]
    fn simple_add_instruction() {
        const CUSTOM_INSTRUCTION_COST: u64 = 133;
        const INITIAL_TX_GAS: u64 = 21000;
        const EXPECTED_RESULT_GAS: u64 = INITIAL_TX_GAS + CUSTOM_INSTRUCTION_COST;

        fn custom_instruction(interp: &mut Interpreter, _host: &mut impl Host) {
            // just spend some gas
            gas!(interp, CUSTOM_INSTRUCTION_COST);
        }

        let code = Bytecode::new_raw([0xED, 0x00].into());
        let code_hash = code.hash_slow();
        let to_addr = address!("ffffffffffffffffffffffffffffffffffffffff");

        let mut evm = Evm::builder()
            .with_db(InMemoryDB::default())
            .modify_db(|db| {
                db.insert_account_info(to_addr, AccountInfo::new(U256::ZERO, 0, code_hash, code))
            })
            .modify_tx_env(|tx| tx.transact_to = TxKind::Call(to_addr))
            .append_handler_register(|handler| {
                handler.instruction_table.insert(0xED, custom_instruction)
            })
            .build();

        let result_and_state = evm.transact().unwrap();
        assert_eq!(result_and_state.result.gas_used(), EXPECTED_RESULT_GAS);
    }

    #[test]
    fn simple_build() {
        // build without external with latest spec
        Evm::builder().build();
        // build with empty db
        Evm::builder().with_empty_db().build();
        // build with_db
        Evm::builder().with_db(EmptyDB::default()).build();
        // build with empty external
        Evm::builder().with_empty_db().build();
        // build with some external
        Evm::builder()
            .with_empty_db()
            .with_external_context(())
            .build();
        // build with spec
        Evm::builder()
            .with_empty_db()
            .with_spec_id(SpecId::HOMESTEAD)
            .build();

        // with with Env change in multiple places
        Evm::builder()
            .with_empty_db()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();
        Evm::builder().modify_tx_env(|tx| tx.gas_limit = 10).build();
        Evm::builder()
            .with_empty_db()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();
        Evm::builder()
            .with_empty_db()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();

        // with inspector handle
        Evm::builder()
            .with_empty_db()
            .with_external_context(NoOpInspector)
            .append_handler_register(inspector_handle_register)
            .build();

        // create the builder
        let evm = Evm::builder()
            .with_db(EmptyDB::default())
            .with_external_context(NoOpInspector)
            .append_handler_register(inspector_handle_register)
            // this would not compile
            // .with_db(..)
            .build();

        let Context { external: _, .. } = evm.into_context();
    }

    #[test]
    fn build_modify_build() {
        // build evm
        let evm = Evm::builder()
            .with_empty_db()
            .with_spec_id(SpecId::HOMESTEAD)
            .build();

        // modify evm
        let evm = evm.modify().with_spec_id(SpecId::FRONTIER).build();
        let _ = evm
            .modify()
            .modify_tx_env(|tx| tx.chain_id = Some(2))
            .build();
    }

    #[test]
    fn build_custom_precompile() {
        struct CustomPrecompile;

        impl ContextStatefulPrecompile<EmptyDB> for CustomPrecompile {
            fn call(
                &self,
                _input: &Bytes,
                _gas_limit: u64,
                _context: &mut InnerEvmContext<EmptyDB>,
            ) -> PrecompileResult {
                Ok(PrecompileOutput::new(10, Bytes::new()))
            }
        }

        let mut evm = Evm::builder()
            .with_empty_db()
            .with_spec_id(SpecId::HOMESTEAD)
            .append_handler_register(|handler| {
                let precompiles = handler.pre_execution.load_precompiles();
                handler.pre_execution.load_precompiles = Arc::new(move || {
                    let mut precompiles = precompiles.clone();
                    precompiles.extend([(
                        Address::ZERO,
                        ContextPrecompile::ContextStateful(Arc::new(CustomPrecompile)),
                    )]);
                    precompiles
                });
            })
            .build();

        evm.transact().unwrap();
    }
}
