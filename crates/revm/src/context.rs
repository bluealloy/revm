mod context_precompiles;
pub(crate) mod evm_context;
mod inner_evm_context;

pub use context_precompiles::{
    ContextPrecompile, ContextPrecompiles, ContextStatefulPrecompile, ContextStatefulPrecompileArc,
    ContextStatefulPrecompileBox, ContextStatefulPrecompileMut,
};
use derive_where::derive_where;
pub use evm_context::EvmContext;
pub use inner_evm_context::InnerEvmContext;
use revm_interpreter::as_usize_saturated;

use crate::{
    db::{Database, EmptyDB},
    interpreter::{Host, LoadAccountResult, SStoreResult, SelfDestructResult},
    primitives::{
        Address, Block as _, Bytes, Env, EthereumWiring, Log, B256, BLOCK_HASH_HISTORY, U256,
    },
    EvmWiring,
};
use std::boxed::Box;

/// Main Context structure that contains both EvmContext and External context.
#[derive_where(Clone; EvmWiringT::Block, EvmWiringT::Context, EvmWiringT::Transaction, DB, DB::Error, EXT)]
pub struct Context<EvmWiringT: EvmWiring, EXT, DB: Database> {
    /// Evm Context (internal context).
    pub evm: EvmContext<EvmWiringT, DB>,
    /// External contexts.
    pub external: EXT,
}

impl Default for Context<EthereumWiring, (), EmptyDB> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<EvmWiringT> Context<EvmWiringT, (), EmptyDB>
where
    EvmWiringT: EvmWiring<Block: Default, Transaction: Default>,
{
    /// Creates empty context. This is useful for testing.
    pub fn new_empty() -> Context<EvmWiringT, (), EmptyDB> {
        Context {
            evm: EvmContext::new(EmptyDB::new()),
            external: (),
        }
    }
}

impl<EvmWiringT, DB> Context<EvmWiringT, (), DB>
where
    EvmWiringT: EvmWiring<Block: Default, Transaction: Default>,
    DB: Database,
{
    /// Creates new context with database.
    pub fn new_with_db(db: DB) -> Context<EvmWiringT, (), DB> {
        Context {
            evm: EvmContext::new_with_env(db, Box::default()),
            external: (),
        }
    }
}

impl<EvmWiringT: EvmWiring, EXT, DB: Database> Context<EvmWiringT, EXT, DB> {
    /// Creates new context with external and database.
    pub fn new(evm: EvmContext<EvmWiringT, DB>, external: EXT) -> Context<EvmWiringT, EXT, DB> {
        Context { evm, external }
    }
}

/// Context with handler configuration.
#[derive_where(Clone; EvmWiringT::Block, EvmWiringT::Context, EvmWiringT::Transaction, DB, DB::Error, EXT)]
pub struct ContextWithEvmWiring<EvmWiringT: EvmWiring, EXT, DB: Database> {
    /// Context of execution.
    pub context: Context<EvmWiringT, EXT, DB>,
    /// Handler configuration.
    pub spec_id: EvmWiringT::Hardfork,
}

impl<EvmWiringT: EvmWiring, EXT, DB: Database> ContextWithEvmWiring<EvmWiringT, EXT, DB> {
    /// Creates new context with handler configuration.
    pub fn new(context: Context<EvmWiringT, EXT, DB>, spec_id: EvmWiringT::Hardfork) -> Self {
        Self { spec_id, context }
    }
}

impl<EvmWiringT: EvmWiring, EXT, DB: Database> Host for Context<EvmWiringT, EXT, DB> {
    type EvmWiringT = EvmWiringT;

    /// Returns reference to Environment.
    #[inline]
    fn env(&self) -> &Env<Self::EvmWiringT> {
        &self.evm.env
    }

    fn env_mut(&mut self) -> &mut Env<EvmWiringT> {
        &mut self.evm.env
    }

    fn block_hash(&mut self, number: u64) -> Option<B256> {
        let block_number = as_usize_saturated!(self.env().block.number());
        let requested_number = usize::try_from(number).unwrap_or(usize::MAX);

        let Some(diff) = block_number.checked_sub(requested_number) else {
            return Some(B256::ZERO);
        };

        // blockhash should push zero if number is same as current block number.
        if diff == 0 {
            return Some(B256::ZERO);
        }

        if diff <= BLOCK_HASH_HISTORY {
            return self
                .evm
                .block_hash(number)
                .map_err(|e| self.evm.error = Err(e))
                .ok();
        }

        Some(B256::ZERO)
    }

    fn load_account(&mut self, address: Address) -> Option<LoadAccountResult> {
        self.evm
            .load_account_exist(address)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn balance(&mut self, address: Address) -> Option<(U256, bool)> {
        self.evm
            .balance(address)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn code(&mut self, address: Address) -> Option<(Bytes, bool)> {
        self.evm
            .code(address)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn code_hash(&mut self, address: Address) -> Option<(B256, bool)> {
        self.evm
            .code_hash(address)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<(U256, bool)> {
        self.evm
            .sload(address, index)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn sstore(&mut self, address: Address, index: U256, value: U256) -> Option<SStoreResult> {
        self.evm
            .sstore(address, index, value)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.evm.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.evm.tstore(address, index, value)
    }

    fn log(&mut self, log: Log) {
        self.evm.journaled_state.log(log);
    }

    fn selfdestruct(&mut self, address: Address, target: Address) -> Option<SelfDestructResult> {
        self.evm
            .inner
            .journaled_state
            .selfdestruct(address, target, &mut self.evm.inner.db)
            .map_err(|e| self.evm.error = Err(e))
            .ok()
    }
}
