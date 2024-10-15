

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EthereumWiring<DB: Database, EXT> {
    phantom: core::marker::PhantomData<(DB, EXT)>,
}

impl<'a, DB: Database, EXT: Debug> EvmWiring for EthereumWiring<DB, EXT> {
    type Database = DB;
    type ExternalContext = EXT;
    type ChainContext = ();
    type Block = crate::default::block::BlockEnv;
    type Transaction = crate::default::TxEnv;
    type Hardfork = SpecId;
    type HaltReason = crate::result::HaltReason;
    type Frame = ();
}

pub type DefaultEthereumWiring = EthereumWiring<EmptyDB, ()>;
