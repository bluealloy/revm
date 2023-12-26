//! Evm Builder.

use crate::{
    db::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef},
    handler::register,
    primitives::{BlockEnv, CfgEnv, Env, LatestSpec, SpecId, TxEnv},
    Context, Evm, EvmContext, Handler,
};
use core::marker::PhantomData;

/// Evm Builder allows building or modifying EVM.
/// Note that some of the methods that changes underlying structures
/// will reset the registered handler to default mainnet.
pub struct EvmBuilder<'a, Stage: BuilderStage, EXT, DB: Database> {
    evm: EvmContext<DB>,
    external: EXT,
    handler: Handler<'a, Evm<'a, EXT, DB>, EXT, DB>,
    phantom: PhantomData<Stage>,
}

/// Trait that unlocks builder stages.
pub trait BuilderStage {}

/// First stage of the builder allows setting the database.
pub struct SettingDbStage;
impl BuilderStage for SettingDbStage {}

/// Second stage of the builder allows setting the external context.
/// Requires the database to be set.
pub struct SettingExternalStage;
impl BuilderStage for SettingExternalStage {}

/// Third stage of the builder allows setting the handler.
/// Requires the database and external context to be set.
pub struct SettingHandlerStage;
impl BuilderStage for SettingHandlerStage {}

impl<'a> Default for EvmBuilder<'a, SettingDbStage, (), EmptyDB> {
    fn default() -> Self {
        Self {
            evm: EvmContext::new(EmptyDB::default()),
            external: (),
            handler: Handler::mainnet::<LatestSpec>(),
            phantom: PhantomData,
        }
    }
}

impl<'a, EXT, DB: Database> EvmBuilder<'a, SettingDbStage, EXT, DB> {
    /// Sets the [`EmptyDB`] as the [`Database`] that will be used by [`Evm`].
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to the mainnet.
    pub fn with_empty_db(self) -> EvmBuilder<'a, SettingExternalStage, EXT, EmptyDB> {
        EvmBuilder {
            evm: self.evm.with_db(EmptyDB::default()),
            external: self.external,
            handler: Handler::mainnet_with_spec(self.handler.spec_id),

            phantom: PhantomData,
        }
    }
    /// Sets the [`Database`] that will be used by [`Evm`].
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_db<ODB: Database>(self, db: ODB) -> EvmBuilder<'a, SettingExternalStage, EXT, ODB> {
        EvmBuilder {
            evm: self.evm.with_db(db),
            external: self.external,
            handler: Handler::mainnet_with_spec(self.handler.spec_id),

            phantom: PhantomData,
        }
    }
    /// Sets the [`DatabaseRef`] that will be used by [`Evm`].
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SettingExternalStage, EXT, WrapDatabaseRef<ODB>> {
        EvmBuilder {
            evm: self.evm.with_db(WrapDatabaseRef(db)),
            external: self.external,
            handler: Handler::mainnet_with_spec(self.handler.spec_id),

            phantom: PhantomData,
        }
    }
}

impl<'a, EXT, DB: Database> EvmBuilder<'a, SettingExternalStage, EXT, DB> {
    /// Sets empty external context.
    pub fn without_external_context(self) -> EvmBuilder<'a, SettingHandlerStage, (), DB> {
        EvmBuilder {
            evm: self.evm,
            external: (),
            handler: Handler::mainnet_with_spec(self.handler.spec_id),
            phantom: PhantomData,
        }
    }

    /// Sets the external context that will be used by [`Evm`].
    pub fn with_external_context<OEXT>(
        self,
        external: OEXT,
    ) -> EvmBuilder<'a, SettingHandlerStage, OEXT, DB> {
        EvmBuilder {
            evm: self.evm,
            external,
            handler: Handler::mainnet_with_spec(self.handler.spec_id),
            phantom: PhantomData,
        }
    }

    /// Modify Database of EVM.
    pub fn modify_db(mut self, f: impl FnOnce(&mut DB)) -> Self {
        f(&mut self.evm.db);
        self
    }

    /// Appends the handler register to the handler.
    pub fn append_handler_register(
        mut self,
        handle_register: register::HandleRegister<'a, EXT, DB>,
    ) -> EvmBuilder<'_, SettingHandlerStage, EXT, DB> {
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
    pub fn append_handler_register_box(
        mut self,
        handle_register: register::HandleRegisterBox<'a, EXT, DB>,
    ) -> Self {
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
    pub fn with_spec_id(mut self, spec_id: SpecId) -> Self {
        self.handler = self.handler.change_spec_id(spec_id);
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: self.handler,

            phantom: PhantomData,
        }
    }
}

impl<'a, EXT, DB: Database> EvmBuilder<'a, SettingHandlerStage, EXT, DB> {
    /// Creates new build from EVM, evm is consumed and all field are moved to Builder.
    ///
    /// Builder is in SettingHandlerStage and both database and external are set.
    pub fn new(evm: Evm<'a, EXT, DB>) -> Self {
        Self {
            evm: evm.context.evm,
            external: evm.context.external,
            handler: evm.handler,
            phantom: PhantomData,
        }
    }

    /// Modify Database of EVM.
    pub fn modify_db(mut self, f: impl FnOnce(&mut DB)) -> Self {
        f(&mut self.evm.db);
        self
    }

    /// Reset handler
    pub fn reset_handler_with_external_context<OEXT>(
        self,
        external_context: OEXT,
    ) -> EvmBuilder<'a, SettingHandlerStage, OEXT, DB> {
        EvmBuilder {
            evm: self.evm,
            external: external_context,
            handler: Handler::mainnet_with_spec(self.handler.spec_id),
            phantom: PhantomData,
        }
    }

    /// Appends the handler register to the handler.
    pub fn append_handler_register(
        mut self,
        handle_register: register::HandleRegister<'a, EXT, DB>,
    ) -> Self {
        self.handler
            .append_handle_register(register::HandleRegisters::Plain(handle_register));
        self
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    pub fn append_handler_register_box(
        mut self,
        handle_register: register::HandleRegisterBox<'a, EXT, DB>,
    ) -> Self {
        self.handler
            .append_handle_register(register::HandleRegisters::Box(handle_register));
        self
    }

    /// Sets the [`EmptyDB`] and resets the [`Handler`]
    pub fn reset_handler_with_empty_db(self) -> EvmBuilder<'a, SettingHandlerStage, EXT, EmptyDB> {
        EvmBuilder {
            evm: self.evm.with_db(EmptyDB::default()),
            external: self.external,
            handler: Handler::mainnet_with_spec(self.handler.spec_id),
            phantom: PhantomData,
        }
    }

    /// Sets the [`Database`] that will be used by [`Evm`].
    pub fn reset_handler_with_db<ODB: Database>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SettingExternalStage, EXT, ODB> {
        EvmBuilder {
            evm: self.evm.with_db(db),
            external: self.external,
            handler: Handler::mainnet_with_spec(self.handler.spec_id),

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
    pub fn with_spec_id(mut self, spec_id: SpecId) -> Self {
        self.handler = self.handler.change_spec_id(spec_id);
        self
    }

    /// Resets [`Handler`] and sets the [`DatabaseRef`] that will be used by [`Evm`].
    pub fn reset_handler_with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> EvmBuilder<'a, SettingExternalStage, EXT, WrapDatabaseRef<ODB>> {
        EvmBuilder {
            evm: self.evm.with_db(WrapDatabaseRef(db)),
            external: self.external,
            handler: Handler::mainnet_with_spec(self.handler.spec_id),

            phantom: PhantomData,
        }
    }
}

// Accessed always.
impl<'a, STAGE: BuilderStage, EXT, DB: Database> EvmBuilder<'a, STAGE, EXT, DB> {
    /// Builds the [`Evm`].
    pub fn build(self) -> Evm<'a, EXT, DB> {
        Evm {
            context: Context {
                evm: self.evm,
                external: self.external,
            },
            handler: self.handler,
        }
    }
    /// Modify Environment of EVM.
    pub fn modify_env(mut self, f: impl FnOnce(&mut Env)) -> Self {
        f(&mut self.evm.env);
        self
    }

    /// Modify Transaction Environment of EVM.
    pub fn modify_tx_env(mut self, f: impl FnOnce(&mut TxEnv)) -> Self {
        f(&mut self.evm.env.tx);
        self
    }

    /// Modify Block Environment of EVM.
    pub fn modify_block_env(mut self, f: impl FnOnce(&mut BlockEnv)) -> Self {
        f(&mut self.evm.env.block);
        self
    }

    /// Modify Config Environment of EVM.
    pub fn modify_cfg_env(mut self, f: impl FnOnce(&mut CfgEnv)) -> Self {
        f(&mut self.evm.env.cfg);
        self
    }

    /// Clear Environment of EVM.
    pub fn with_clear_env(mut self) -> Self {
        self.evm.env.clear();
        self
    }

    /// Clear Transaction environment of EVM.
    pub fn with_clear_tx_env(mut self) -> Self {
        self.evm.env.tx.clear();
        self
    }
    /// Clear Block environment of EVM.
    pub fn with_clear_block_env(mut self) -> Self {
        self.evm.env.block.clear();
        self
    }
}

#[cfg(test)]
mod test {
    use super::SpecId;
    use crate::{
        db::EmptyDB, inspector::inspector_handle_register, inspectors::NoOpInspector, Evm,
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
        Evm::builder()
            .with_empty_db()
            .without_external_context()
            .build();
        // build with some external
        Evm::builder()
            .with_empty_db()
            .with_external_context(())
            .build();
        // build with spec
        Evm::builder()
            .with_empty_db()
            .without_external_context()
            .with_spec_id(SpecId::HOMESTEAD)
            .build();

        // with with Env change in multiple places
        Evm::builder()
            .with_empty_db()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .without_external_context()
            .build();
        Evm::builder().modify_tx_env(|tx| tx.gas_limit = 10).build();
        Evm::builder()
            .with_empty_db()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();
        Evm::builder()
            .with_empty_db()
            .without_external_context()
            .modify_tx_env(|tx| tx.gas_limit = 10)
            .build();

        // with inspector handle
        Evm::builder()
            .with_empty_db()
            .with_external_context(NoOpInspector::default())
            .append_handler_register(inspector_handle_register)
            .build();
    }

    #[test]
    fn build_modify_build() {
        let evm = Evm::builder()
            .with_empty_db()
            .without_external_context()
            .with_spec_id(SpecId::HOMESTEAD)
            .build();

        let evm = evm.modify().with_spec_id(SpecId::FRONTIER).build();
        let _ = evm
            .modify()
            .modify_tx_env(|tx| tx.chain_id = Some(2))
            .build();
    }
}
