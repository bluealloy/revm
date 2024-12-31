use crate::{
    handler::{
        precompiles::OpPrecompileProvider, OpExecution, OpHandler, OpPreExecution, OpValidation,
    },
    L1BlockInfo, OpSpec, OpTransaction,
};
use inspector::{InspectorContext, InspectorEthFrame};
use revm::{
    context::{block::BlockEnv, tx::TxEnv, CfgEnv, Context},
    context_interface::result::{EVMError, InvalidTransaction},
    database_interface::Database,
    Evm, JournaledState,
};

/// Optimism Error
pub type OpError<DB> = EVMError<<DB as Database>::Error, InvalidTransaction>;

/// Optimism Context
pub type OpContext<DB> = Context<BlockEnv, OpTransaction<TxEnv>, CfgEnv<OpSpec>, DB, L1BlockInfo>;

/// Optimism EVM type
pub type OpEvm<DB> = Evm<OpError<DB>, OpContext<DB>, OpHandler<OpContext<DB>, OpError<DB>>>;

pub type InspCtxType<INSP, DB> = InspectorContext<
    INSP,
    DB,
    Context<BlockEnv, TxEnv, CfgEnv<OpSpec>, DB, JournaledState<DB>, L1BlockInfo>,
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
