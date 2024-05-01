mod context_precompiles;
pub(crate) mod evm_context;
mod inner_evm_context;

pub use context_precompiles::{
    ContextPrecompile, ContextPrecompiles, ContextStatefulPrecompile, ContextStatefulPrecompileArc,
    ContextStatefulPrecompileBox, ContextStatefulPrecompileMut,
};
pub use evm_context::EvmContext;
pub use inner_evm_context::InnerEvmContext;
use revm_interpreter::as_usize_saturated;

use crate::{
    db::{Database, EmptyDB},
    interpreter::{Host, LoadAccountResult, SStoreResult, SelfDestructResult},
    primitives::{
        Address, Block as _, Bytes, ChainSpec, Env, EthChainSpec, Log, B256, BLOCK_HASH_HISTORY,
        U256,
    },
};
use std::boxed::Box;

/// Main Context structure that contains both EvmContext and External context.
pub struct Context<ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Evm Context (internal context).
    pub evm: EvmContext<ChainSpecT, DB>,
    /// External contexts.
    pub external: EXT,
}

impl<ChainSpecT: ChainSpec, EXT: Clone, DB: Database + Clone> Clone for Context<ChainSpecT, EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            evm: self.evm.clone(),
            external: self.external.clone(),
        }
    }
}

impl Default for Context<EthChainSpec, (), EmptyDB> {
    fn default() -> Self {
        Self::new_empty()
    }
}

impl<ChainSpecT: ChainSpec> Context<ChainSpecT, (), EmptyDB> {
    /// Creates empty context. This is useful for testing.
    pub fn new_empty() -> Context<ChainSpecT, (), EmptyDB> {
        Context {
            evm: EvmContext::new(EmptyDB::new()),
            external: (),
        }
    }
}

impl<ChainSpecT: ChainSpec, DB: Database> Context<ChainSpecT, (), DB> {
    /// Creates new context with database.
    pub fn new_with_db(db: DB) -> Context<ChainSpecT, (), DB> {
        Context {
            evm: EvmContext::new_with_env(db, Box::default()),
            external: (),
        }
    }
}

impl<ChainSpecT: ChainSpec, EXT, DB: Database> Context<ChainSpecT, EXT, DB> {
    /// Creates new context with external and database.
    pub fn new(evm: EvmContext<ChainSpecT, DB>, external: EXT) -> Context<ChainSpecT, EXT, DB> {
        Context { evm, external }
    }
}

/// Context with handler configuration.
pub struct ContextWithChainSpec<ChainSpecT: ChainSpec, EXT, DB: Database> {
    /// Context of execution.
    pub context: Context<ChainSpecT, EXT, DB>,
    /// Handler configuration.
    pub spec_id: ChainSpecT::Hardfork,
}

impl<ChainSpecT: ChainSpec, EXT, DB: Database> ContextWithChainSpec<ChainSpecT, EXT, DB> {
    /// Creates new context with handler configuration.
    pub fn new(context: Context<ChainSpecT, EXT, DB>, spec_id: ChainSpecT::Hardfork) -> Self {
        Self { spec_id, context }
    }
}

impl<ChainSpecT: ChainSpec, EXT: Clone, DB: Database + Clone> Clone
    for ContextWithChainSpec<ChainSpecT, EXT, DB>
where
    DB::Error: Clone,
{
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
            spec_id: self.spec_id,
        }
    }
}

impl<ChainSpecT: ChainSpec, EXT, DB: Database> Host for Context<ChainSpecT, EXT, DB> {
    type ChainSpecT = ChainSpecT;

    /// Returns reference to Environment.
    #[inline]
    fn env(&self) -> &Env<ChainSpecT> {
        &self.evm.env
    }

    fn env_mut(&mut self) -> &mut Env<ChainSpecT> {
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
