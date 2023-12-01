//! Evm Builder.

use core::marker::PhantomData;

use revm_interpreter::opcode::make_instruction_table;

use crate::{
    db::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef},
    handler::{
        register::{self, RawInstructionTable},
        HandleRegister,
    },
    primitives::{BlockEnv, CfgEnv, Env, Spec, TxEnv},
    primitives::{LatestSpec, SpecId},
    Context, Evm, EvmContext, Handler,
};

/// Evm Builder allows building or modifying EVM.
/// Note that some of the methods that changes underlying structures
///  will reset the registered handler to default mainnet.
pub struct EvmBuilder<'a, Stage: BuilderStage, EXT, DB: Database> {
    evm: EvmContext<DB>,
    external: EXT,
    handler: Handler<'a, Evm<'a, EXT, DB>, EXT, DB>,
    handle_registers: Vec<HandleRegister<'a, EXT, DB>>,
    phantom: PhantomData<Stage>,
}

pub trait BuilderStage {}

pub struct SettingDbStage;
impl BuilderStage for SettingDbStage {}

pub struct SettingExternalStage;
impl BuilderStage for SettingExternalStage {}

pub struct SettingHandlerStage;
impl BuilderStage for SettingHandlerStage {}

impl<'a> Default for EvmBuilder<'a, SettingDbStage, (), EmptyDB> {
    fn default() -> Self {
        Self {
            evm: EvmContext::new(EmptyDB::default()),
            external: (),
            handler: Handler::mainnet::<LatestSpec>(),
            handle_registers: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<'a, EXT, DB: Database> EvmBuilder<'a, SettingDbStage, EXT, DB> {
    /// Sets the [`Database`] that will be used by [`Evm`].
    ///
    /// # Note
    ///
    /// When changed it will reset the handler to default mainnet.
    pub fn with_db<ODB: Database>(self, db: ODB) -> EvmBuilder<'a, SettingExternalStage, EXT, ODB> {
        EvmBuilder {
            evm: EvmContext::new(db),
            external: self.external,
            handler: Handler::mainnet::<LatestSpec>(),
            handle_registers: Vec::new(),
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
        let present_spec_id = self.handler.spec_id;

        let mut builder = EvmBuilder {
            evm: EvmContext::new(WrapDatabaseRef(db)),
            external: self.external,
            handler: Handler::mainnet::<LatestSpec>(),
            handle_registers: Vec::new(),
            phantom: PhantomData,
        };
        builder
    }
}

impl<'a, EXT, DB: Database> EvmBuilder<'a, SettingExternalStage, EXT, DB> {
    pub fn with_empty_external(self) -> EvmBuilder<'a, SettingExternalStage, (), DB> {
        EvmBuilder {
            evm: self.evm,
            external: (),
            handler: Handler::mainnet::<LatestSpec>(),
            handle_registers: Vec::new(),
            phantom: PhantomData,
        }
    }

    pub fn with_external<OEXT>(
        self,
        external: OEXT,
    ) -> EvmBuilder<'a, SettingHandlerStage, OEXT, DB> {
        EvmBuilder {
            evm: self.evm,
            external: external,
            handler: Handler::mainnet::<LatestSpec>(),
            handle_registers: Vec::new(),
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
            // TODO move registers from EVM
            handle_registers: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// Consumes the Builder and build the Build Evm.
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
        let mut handler = Handler::mainnet::<SPEC>();
        let raw_table =
            RawInstructionTable::PlainRaw(make_instruction_table::<Evm<'a, EXT, DB>, SPEC>());
        // apply all registers to default handeler and raw mainnet instruction table.
        for register in self.handle_registers.iter() {
            register(&mut handler, &mut RawInstructionTable::Default);
        }
        handler.instruction_table = raw_table.into_arc();
        handler
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
    /// When changed it will reapply all handle registers.
    pub fn with_spec_id(mut self, spec_id: SpecId) -> Self {
        // TODO add match for other spec
        self.handler = self.create_handler(spec_id);
        self
    }

    /// Register Handler that modifies the behavior of EVM.
    /// Check [`Handler`] for more information.
    pub fn push_handler(mut self, handle_register: register::Register<'a, EXT, DB>) -> Self {
        //self.handle_registers.push(handle_register);
        self
    }
}

// Accessed always.

impl<'a, STAGE: BuilderStage, EXT, DB: Database> EvmBuilder<'a, STAGE, EXT, DB> {
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
}
