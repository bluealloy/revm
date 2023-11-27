use revm_interpreter::primitives::SpecId;

use crate::{
    db::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef},
    handler::{MainnetHandle, RegisterHandler},
    primitives::LatestSpec,
    Context, Evm, EvmContext, Handler,
};

/// Evm Builder
pub struct EvmBuilder<'a, EXT, DB: Database> {
    evm: EvmContext<DB>,
    external: EXT,
    handler: Handler<'a, Evm<'a, EXT, DB>, EXT, DB>,
    spec_id: SpecId,
}

impl<'a> Default for EvmBuilder<'a, MainnetHandle, EmptyDB> {
    fn default() -> Self {
        Self {
            evm: EvmContext::new(EmptyDB::default()),
            external: MainnetHandle::default(),
            handler: Handler::mainnet::<LatestSpec>(),
            spec_id: SpecId::LATEST,
        }
    }
}

impl<'a, EXT, DB: Database> EvmBuilder<'a, EXT, DB> {
    pub fn new(evm: Evm<'a, EXT, DB>) -> Self {
        Self {
            evm: evm.context.evm,
            external: evm.context.external,
            handler: evm.handler,
            spec_id: evm.spec_id,
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
            spec_id: self.spec_id,
        }
    }

    /// Sets specification Id , that will mark the version of EVM.
    /// It represent the hard fork of ethereum.
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_spec_id(mut self, spec_id: SpecId) -> Self {
        self.spec_id = spec_id;
        // TODO add match for other spec
        self.handler = Handler::mainnet::<LatestSpec>();
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
            spec_id: self.spec_id,
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
        EvmBuilder {
            evm: EvmContext::new(WrapDatabaseRef(db)),
            external: self.external,
            handler: Handler::mainnet::<LatestSpec>(),
            spec_id: self.spec_id,
        }
    }

    /// Sets the external data that can be used by Handler inside EVM.
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_external<OEXT>(self, external: OEXT) -> EvmBuilder<'a, OEXT, DB> {
        EvmBuilder {
            evm: self.evm,
            external: external,
            handler: Handler::mainnet::<LatestSpec>(),
            spec_id: self.spec_id,
        }
    }

    pub fn register_handler<H: RegisterHandler<'a, DB, EXT>>(
        self,
        handler: H,
    ) -> EvmBuilder<'a, EXT, DB> {
        EvmBuilder {
            evm: self.evm,
            external: self.external,
            handler: handler.register_handler::<LatestSpec>(self.handler),
            spec_id: self.spec_id,
        }
    }
}
