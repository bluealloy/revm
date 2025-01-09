use crate::{
    handler::{
        precompiles::OpPrecompileProvider, OpExecution, OpHandler, OpPreExecution, OpValidation,
    },
    OpSpec, OpTransaction,
};
use inspector::{inspector_context::InspectorContext, InspectorEthFrame};
use maili_protocol::L1BlockInfoTx;
use revm::{
    context::{block::BlockEnv, tx::TxEnv, CfgEnv, Context},
    context_interface::result::{EVMError, InvalidTransaction},
    context_interface::Journal,
    database_interface::Database,
    Evm, JournaledState,
};

/// Optimism Error
pub type OpError<DB> = EVMError<<DB as Database>::Error, InvalidTransaction>;

/// Optimism Context
pub type OpContext<DB> = Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpec>, DB, L1BlockInfoTx>;

/// Defines functionality to retrieve the [`L1BlockInfoTx`].
pub trait L1BlockInfoGetter {
    /// Returns the [`L1BlockInfoTx`] of the context.
    fn l1_block_info(&self) -> &L1BlockInfoTx;

    /// Returns the mutable reference to the [`L1BlockInfoTx`] of the context.
    fn l1_block_info_mut(&mut self) -> &mut L1BlockInfoTx;
}

impl<BLOCK, TX, SPEC, DB: Database, JOURNAL: Journal<Database = DB>> L1BlockInfoGetter
    for Context<BLOCK, TX, SPEC, DB, JOURNAL, L1BlockInfoTx>
{
    fn l1_block_info(&self) -> &L1BlockInfoTx {
        &self.chain
    }

    fn l1_block_info_mut(&mut self) -> &mut L1BlockInfoTx {
        &mut self.chain
    }
}

/// Optimism EVM type
pub type OpEvm<DB> = Evm<OpError<DB>, OpContext<DB>, OpHandler<OpContext<DB>, OpError<DB>>>;

pub type InspCtxType<INSP, DB> = InspectorContext<
    INSP,
    DB,
    Context<BlockEnv, TxEnv, CfgEnv<OpSpec>, DB, JournaledState<DB>, L1BlockInfoTx>,
>;

pub type InspectorOpEvm<DB, INSP> = Evm<
    OpError<DB>,
    InspCtxType<INSP, DB>,
    OpHandler<
        InspCtxType<INSP, DB>,
        OpError<DB>,
        OpValidation<InspCtxType<INSP, DB>, OpError<DB>>,
        OpPreExecution<InspCtxType<INSP, DB>, OpError<DB>>,
        OpExecution<
            InspCtxType<INSP, DB>,
            OpError<DB>,
            InspectorEthFrame<
                InspCtxType<INSP, DB>,
                OpError<DB>,
                OpPrecompileProvider<InspCtxType<INSP, DB>, OpError<DB>>,
            >,
        >,
    >,
>;
