use crate::{
    db::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef},
    handler::register,
    primitives::{
        BlockEnv, CfgEnv, CfgEnvWithHandlerCfg, Env, EnvWithHandlerCfg, HandlerCfg, SpecId, TxEnv,
    },
    Context, Evm, EvmContext, Handler,
};
use alloc::boxed::Box;
use core::marker::PhantomData;

/// Evm Builder allows building or modifying EVM.
/// Note that some of the methods that changes underlying structures
/// will reset the registered handler to default mainnet.
pub struct EvmBuilder<'a, BuilderStage, EXT, DB: Database> {
    /// Evm context containing database journal, and precompiles.
    evm: EvmContext<DB>,
    /// External context that will be used by EVM.
    external: EXT,
    /// Handler that will be used by EVM. It contains handle registers
    handler: Handler<'a, Evm<'a, EXT, DB>, EXT, DB>,
    /// Phantom data to mark the stage of the builder.
    phantom: PhantomData<BuilderStage>,
}

/// First stage of the builder allows setting generic variables.
/// Generic variables are database and external context.
pub struct SetGenericStage;

/// Second stage of the builder allows appending handler registers.
/// Requires the database and external context to be set.
pub struct HandlerStage;

impl<'a> Default for EvmBuilder<'a, SetGenericStage, (), EmptyDB> {
    fn default() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(all(feature = "optimism_default_handler",
                not(feature = "negate_optimism_default_handler")))] {
                    let mut handler_cfg = HandlerCfg::new(SpecId::LATEST);
                    /// set is_optimism to true by default.
                    handler_cfg.is_optimism = true;

            } else {
                let handler_cfg = HandlerCfg::new(SpecId::LATEST);
            }
        }

        Self {
            evm: EvmContext::new(EmptyDB::default()),
            external: (),
            handler: EvmBuilder::<'a, SetGenericStage, (), EmptyDB>::handler(handler_cfg),
            phantom: PhantomData,
        }
    }
}

impl<'a, EXT, DB: Database> EvmBuilder<'a, SetGenericStage, EXT, DB> {
    /// Sets the [`EmptyDB`] as the [`Database`] that will be used by [`Evm`].
    pub fn with_empty_db(self) -> EvmBuilder<'a, SetGenericStage, EXT, EmptyDB> {
        EvmBuilder {
            evm: self.evm.with_db(EmptyDB::default()),
            external: self.external,
            handler: EvmBuilder::<'a, SetGenericStage, EXT, EmptyDB>::handler(self.handler.cfg()),
            phantom: PhantomData,
        }
    }
    /// Sets the [`Database`] that will be used by [`Evm`].
    pub fn with_db<ODB: Database>(self, db: ODB) -> EvmBuilder<'a, SetGenericStage, EXT, ODB> {
        EvmBuilder {
            evm: self.evm.with_db(db),
            external: self.external,
            handler: EvmBuilder::<'a, SetGenericStage, EXT, ODB>::handler(self.handler.cfg()),
            phantom: PhantomData,
        }
    }
    /// Sets the [`DatabaseRef`] that will be used by [`Evm`].
    pub fn with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SetGenericStage, EXT, WrapDatabaseRef<ODB>> {
        EvmBuilder {
            evm: self.evm.with_db(WrapDatabaseRef(db)),
            external: self.external,
            handler: EvmBuilder::<'a, SetGenericStage, EXT, WrapDatabaseRef<ODB>>::handler(
                self.handler.cfg(),
            ),
            phantom: PhantomData,
        }
    }

    /// Sets the external context that will be used by [`Evm`].
    pub fn with_external_context<OEXT>(
        self,
        external: OEXT,
    ) -> EvmBuilder<'a, SetGenericStage, OEXT, DB> {
        EvmBuilder {
            evm: self.evm,
            external,
            handler: EvmBuilder::<'a, SetGenericStage, OEXT, DB>::handler(self.handler.cfg()),
            phantom: PhantomData,
        }
    }

    /// Sets Builder with [`CfgEnvWithHandlerCfg`].
    pub fn with_env_with_spec_id(
        mut self,
        env_and_spec_id: EnvWithHandlerCfg,
    ) -> EvmBuilder<'a, HandlerStage, EXT, DB> {
        self.evm.env = env_and_spec_id.env;
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: EvmBuilder::<'a, HandlerStage, EXT, DB>::handler(env_and_spec_id.handler_cfg),
            phantom: PhantomData,
        }
    }

    /// Sets Builder with [`CfgEnvWithHandlerCfg`].
    pub fn with_cfg_env_with_spec_id(
        mut self,
        cfg_env_and_spec_id: CfgEnvWithHandlerCfg,
    ) -> EvmBuilder<'a, HandlerStage, EXT, DB> {
        self.evm.env.cfg = cfg_env_and_spec_id.cfg_env;

        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: EvmBuilder::<'a, HandlerStage, EXT, DB>::handler(
                cfg_env_and_spec_id.handler_cfg,
            ),
            phantom: PhantomData,
        }
    }

    /// Sets the Optimism handler with latest spec.
    ///
    /// If `optimism_default_handler` feature is enabled this is not needed.
    #[cfg(feature = "optimism")]
    pub fn optimism(mut self) -> EvmBuilder<'a, HandlerStage, EXT, DB> {
        self.handler = Handler::optimism_with_spec(self.handler.cfg.spec_id);
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: self.handler,
            phantom: PhantomData,
        }
    }

    /// Sets the mainnet handler with latest spec.
    ///
    /// Enabled only with `optimism_default_handler` feature.
    #[cfg(feature = "optimism_default_handler")]
    pub fn mainnet(mut self) -> EvmBuilder<'a, HandlerStage, EXT, DB> {
        self.handler = Handler::mainnet_with_spec(self.handler.cfg.spec_id);
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: self.handler,
            phantom: PhantomData,
        }
    }
}

impl<'a, EXT, DB: Database> EvmBuilder<'a, HandlerStage, EXT, DB> {
    /// Creates new builder from Evm, Evm is consumed and all field are moved to Builder.
    /// It will preserve set handler and context.
    ///
    /// Builder is in HandlerStage and both database and external are set.
    pub fn new(evm: Evm<'a, EXT, DB>) -> Self {
        Self {
            evm: evm.context.evm,
            external: evm.context.external,
            handler: evm.handler,
            phantom: PhantomData,
        }
    }

    /// Sets the [`EmptyDB`] and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_empty_db(self) -> EvmBuilder<'a, HandlerStage, EXT, EmptyDB> {
        EvmBuilder {
            evm: self.evm.with_db(EmptyDB::default()),
            external: self.external,
            handler: EvmBuilder::<'a, HandlerStage, EXT, EmptyDB>::handler(self.handler.cfg()),
            phantom: PhantomData,
        }
    }

    /// Resets the [`Handler`] and sets base mainnet handler.
    ///
    /// Enabled only with `optimism_default_handler` feature.
    #[cfg(feature = "optimism_default_handler")]
    pub fn reset_handler_with_mainnet(mut self) -> EvmBuilder<'a, HandlerStage, EXT, DB> {
        self.handler = Handler::mainnet_with_spec(self.handler.cfg.spec_id);
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: self.handler,
            phantom: PhantomData,
        }
    }

    /// Sets the [`Database`] that will be used by [`Evm`]
    /// and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_db<ODB: Database>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SetGenericStage, EXT, ODB> {
        EvmBuilder {
            evm: self.evm.with_db(db),
            external: self.external,
            handler: EvmBuilder::<'a, SetGenericStage, EXT, ODB>::handler(self.handler.cfg()),
            phantom: PhantomData,
        }
    }

    /// Resets [`Handler`] and sets the [`DatabaseRef`] that will be used by [`Evm`]
    /// and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SetGenericStage, EXT, WrapDatabaseRef<ODB>> {
        EvmBuilder {
            evm: self.evm.with_db(WrapDatabaseRef(db)),
            external: self.external,
            handler: EvmBuilder::<'a, SetGenericStage, EXT, WrapDatabaseRef<ODB>>::handler(
                self.handler.cfg(),
            ),
            phantom: PhantomData,
        }
    }

    /// Resets [`Handler`] and sets new `ExternalContext` type.
    ///  and resets the [`Handler`] to default mainnet.
    pub fn reset_handler_with_external_context<OEXT>(
        self,
        external: OEXT,
    ) -> EvmBuilder<'a, SetGenericStage, OEXT, DB> {
        EvmBuilder {
            evm: self.evm,
            external,
            handler: EvmBuilder::<'a, SetGenericStage, OEXT, DB>::handler(self.handler.cfg()),
            phantom: PhantomData,
        }
    }
}

impl<'a, BuilderStage, EXT, DB: Database> EvmBuilder<'a, BuilderStage, EXT, DB> {
    /// Creates the default handler.
    ///
    /// This is useful for adding optimism handle register.
    fn handler(handler_cfg: HandlerCfg) -> Handler<'a, Evm<'a, EXT, DB>, EXT, DB> {
        Handler::new(handler_cfg)
    }

    /// Builds the [`Evm`].
    pub fn build(self) -> Evm<'a, EXT, DB> {
        Evm::new(
            Context {
                evm: self.evm,
                external: self.external,
            },
            self.handler,
        )
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    ///
    /// When called, EvmBuilder will transition from SetGenericStage to HandlerStage.
    pub fn append_handler_register(
        mut self,
        handle_register: register::HandleRegister<'a, EXT, DB>,
    ) -> EvmBuilder<'_, HandlerStage, EXT, DB> {
        self.handler
            .append_handle_register(register::HandleRegisters::Plain(handle_register));
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: self.handler,

            phantom: PhantomData,
        }
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    ///
    /// When called, EvmBuilder will transition from SetGenericStage to HandlerStage.
    pub fn append_handler_register_box(
        mut self,
        handle_register: register::HandleRegisterBox<'a, EXT, DB>,
    ) -> EvmBuilder<'_, HandlerStage, EXT, DB> {
        self.handler
            .append_handle_register(register::HandleRegisters::Box(handle_register));
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: self.handler,

            phantom: PhantomData,
        }
    }

    /// Sets specification Id , that will mark the version of EVM.
    /// It represent the hard fork of ethereum.
    ///
    /// # Note
    ///
    /// When changed it will reapply all handle registers, this can be
    /// expensive operation depending on registers.
    pub fn spec_id(mut self, spec_id: SpecId) -> Self {
        self.handler = self.handler.change_spec_id(spec_id);
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: self.handler,

            phantom: PhantomData,
        }
    }

    /// Allows modification of Evm Database.
    pub fn modify_db(mut self, f: impl FnOnce(&mut DB)) -> Self {
        f(&mut self.evm.db);
        self
    }

    /// Allows modification of external context.
    pub fn modify_external_context(mut self, f: impl FnOnce(&mut EXT)) -> Self {
        f(&mut self.external);
        self
    }

    /// Allows modification of Evm Environment.
    pub fn modify_env(mut self, f: impl FnOnce(&mut Box<Env>)) -> Self {
        f(&mut self.evm.env);
        self
    }

    /// Allows modification of Evm's Transaction Environment.
    pub fn modify_tx_env(mut self, f: impl FnOnce(&mut TxEnv)) -> Self {
        f(&mut self.evm.env.tx);
        self
    }

    /// Allows modification of Evm's Block Environment.
    pub fn modify_block_env(mut self, f: impl FnOnce(&mut BlockEnv)) -> Self {
        f(&mut self.evm.env.block);
        self
    }

    /// Allows modification of Evm's Config Environment.
    pub fn modify_cfg_env(mut self, f: impl FnOnce(&mut CfgEnv)) -> Self {
        f(&mut self.evm.env.cfg);
        self
    }

    /// Clears Environment of EVM.
    pub fn with_clear_env(mut self) -> Self {
        self.evm.env.clear();
        self
    }

    /// Clears Transaction environment of EVM.
    pub fn with_clear_tx_env(mut self) -> Self {
        self.evm.env.tx.clear();
        self
    }
    /// Clears Block environment of EVM.
    pub fn with_clear_block_env(mut self) -> Self {
        self.evm.env.block.clear();
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
        db::EmptyDB, inspector::inspector_handle_register, inspectors::NoOpInspector, Context, Evm,
        EvmContext,
    };

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
            .spec_id(SpecId::HOMESTEAD)
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

        let Context {
            external,
            evm: EvmContext { db, .. },
        } = evm.into_context();
        let _ = (external, db);
    }

    #[test]
    fn build_modify_build() {
        // build evm
        let evm = Evm::builder()
            .with_empty_db()
            .spec_id(SpecId::HOMESTEAD)
            .build();

        // modify evm
        let evm = evm.modify().spec_id(SpecId::FRONTIER).build();
        let _ = evm
            .modify()
            .modify_tx_env(|tx| tx.chain_id = Some(2))
            .build();
    }
}
