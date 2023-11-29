//! Evm Builder.

use core::marker::PhantomData;

use crate::{
    db::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef},
    handler::{MainnetHandle, RegisterHandler},
    primitives::{BlockEnv, CfgEnv, Env, Spec, TxEnv},
    primitives::{LatestSpec, SpecId},
    Context, Evm, EvmContext, Handler,
};

/// Evm Builder allows building or modifying EVM.
/// Note that some of the methods that changes underlying structures
///  will reset the registered handler to default mainnet.
pub struct EvmBuilder<'a, STAGE: BuilderStage, EXT: RegisterHandler<'a, DB, EXT>, DB: Database> {
    evm: EvmContext<DB>,
    external: EXT,
    handler: Handler<'a, Evm<'a, EXT, DB>, EXT, DB>,
    phantom: PhantomData<STAGE>,
}

pub trait BuilderStage {}

pub struct SettingDb;
impl BuilderStage for SettingDb {}

pub struct SettingExternal;
impl BuilderStage for SettingExternal {}

impl<'a> Default for EvmBuilder<'a, SettingDb, MainnetHandle, EmptyDB> {
    fn default() -> Self {
        Self {
            evm: EvmContext::new(EmptyDB::default()),
            external: MainnetHandle::default(),
            handler: Handler::mainnet::<LatestSpec>(),
            phantom: PhantomData,
        }
    }
}

impl<'a, EXT: RegisterHandler<'a, DB, EXT>, DB: Database> EvmBuilder<'a, SettingExternal, EXT, DB> {}
/*
impl<'a, EXT: RegisterHandler<'a,DB,EXT>, DB: Database> EvmBuilder<'a, EXT, DB> {
    pub fn new(evm: Evm<'a, EXT, DB>) -> Self {
        Self {
            evm: evm.context.evm,
            external: evm.context.external,
            handler: evm.handler,
        }
    }

    /// Build Evm.
    pub fn build(self) -> Evm<'a, EXT, DB> {
        Evm {
            context: Context {
                evm: self.evm,
                external: self.external,
            },
            handler: self.handler,
        }
    }

    /// Creates the Handler with Generic Spec.
    fn create_handle_generic<SPEC: Spec + 'static>(
        &self,
    ) -> Handler<'a, Evm<'a, EXT, DB>, EXT, DB> {
        self.external.register_handle(Handler::mainnet::<SPEC>())
    }

    /// Creates the Handler with variable SpecId, inside it will call function with Generic Spec.
    fn create_handler(&self, spec_id: SpecId) -> Handler<'a, Evm<'a, EXT, DB>, EXT, DB> {
        use crate::primitives::specification::*;
        match spec_id {
            SpecId::FRONTIER | SpecId::FRONTIER_THAWING => {
                self.create_handle_generic::<FrontierSpec>()
            }
            SpecId::HOMESTEAD | SpecId::DAO_FORK => self.create_handle_generic::<HomesteadSpec>(),
            SpecId::TANGERINE => self.create_handle_generic::<TangerineSpec>(),
            SpecId::SPURIOUS_DRAGON => self.create_handle_generic::<SpuriousDragonSpec>(),
            SpecId::BYZANTIUM => self.create_handle_generic::<ByzantiumSpec>(),
            SpecId::PETERSBURG | SpecId::CONSTANTINOPLE => {
                self.create_handle_generic::<PetersburgSpec>()
            }
            SpecId::ISTANBUL | SpecId::MUIR_GLACIER => self.create_handle_generic::<IstanbulSpec>(),
            SpecId::BERLIN => self.create_handle_generic::<BerlinSpec>(),
            SpecId::LONDON | SpecId::ARROW_GLACIER | SpecId::GRAY_GLACIER => {
                self.create_handle_generic::<LondonSpec>()
            }
            SpecId::MERGE => self.create_handle_generic::<MergeSpec>(),
            SpecId::SHANGHAI => self.create_handle_generic::<ShanghaiSpec>(),
            SpecId::CANCUN => self.create_handle_generic::<CancunSpec>(),
            SpecId::LATEST => self.create_handle_generic::<LatestSpec>(),
            #[cfg(feature = "optimism")]
            SpecId::BEDROCK => self.create_handle_generic::<BedrockSpec>(),
            #[cfg(feature = "optimism")]
            SpecId::REGOLITH => self.create_handle_generic::<RegolithSpec>(),
        }
    }

    /// Sets specification Id , that will mark the version of EVM.
    /// It represent the hard fork of ethereum.
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_spec_id(mut self, spec_id: SpecId) -> Self {
        // TODO add match for other spec
        self.handler = self.create_handler(spec_id);
        self
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

    /// Sets the [`Database`] that will be used by [`Evm`].
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_db<ODB: Database>(self, db: ODB) -> EvmBuilder<'a, EXT, ODB> {
        EvmBuilder {
            evm: EvmContext::new(db),
            external: self.external,
            handler: Handler::mainnet::<LatestSpec>(),
        }
    }
    /// Sets the [`DatabaseRef`] that will be used by [`Evm`].
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_ref_db<RDB: DatabaseRef>(
        self,
        db: RDB,
    ) -> EvmBuilder<'a, EXT, WrapDatabaseRef<RDB>> {
        let present_spec_id = self.handler.spec_id;

        let mut builder = EvmBuilder {
            evm: EvmContext::new(WrapDatabaseRef(db)),
            external: self.external,
            handler: Handler::mainnet::<LatestSpec>(),
        };
        builder.handler = builder.create_handler(present_spec_id);
        builder
    }

    /// Sets the external data that can be used by Handler inside EVM.
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_external<OEXT: RegisterHandler<'a, DB, OEXT>>(
        self,
        external: OEXT,
    ) -> EvmBuilder<'a, OEXT, DB> {
        let handler = external.register_handler::<LatestSpec>(Handler::mainnet::<LatestSpec>());
        EvmBuilder {
            evm: self.evm,
            external: external,
            handler,
        }

        let present_spec_id = self.handler.spec_id;

        let mut builder = EvmBuilder {
            evm: EvmContext::new(WrapDatabaseRef(db)),
            external: self.external,
            handler: Handler::mainnet::<LatestSpec>(),
        };
        builder.handler = builder.create_handler(present_spec_id);
        builder
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    pub fn register_handler<H: RegisterHandler<'a, DB, EXT>>(
        self,
        handler: H,
    ) -> EvmBuilder<'a, EXT, DB> {
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: handler.register_handler::<LatestSpec>(self.handler),
        }
    }
}
*/
