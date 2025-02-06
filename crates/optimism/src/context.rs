use crate::{
    api::exec_op::transact_op,
    transaction::{abstraction::OpTxGetter, OpTxTrait},
    L1BlockInfo, L1BlockInfoGetter, OpHaltReason, OpSpec, OpSpecId, OpTransaction,
    OpTransactionError,
};
use derive_more::derive::{AsMut, AsRef, Deref, DerefMut};
use inspector::journal::{JournalExt, JournalExtGetter};
use precompile::Log;
use revm::{
    context::{BlockEnv, CfgEnv, TxEnv},
    context_interface::{
        block::BlockSetter,
        result::{EVMError, ExecutionResult, ResultAndState},
        transaction::TransactionSetter,
        Block, BlockGetter, Cfg, CfgGetter, DatabaseGetter, ErrorGetter, Journal, JournalDBError,
        JournalGetter, PerformantContextAccess, Transaction, TransactionGetter,
    },
    database_interface::EmptyDB,
    handler::EthContext,
    interpreter::Host,
    state::EvmState,
    Context, Database, DatabaseCommit, ExecuteCommitEvm, ExecuteEvm, JournaledState,
};
use std::vec::Vec;

#[derive(AsRef, AsMut, Deref, DerefMut)]
pub struct OpContext<
    BLOCK = BlockEnv,
    TX = OpTransaction<TxEnv>,
    CFG = CfgEnv<OpSpec>,
    DB: Database = EmptyDB,
    JOURNAL: Journal<Database = DB> = JournaledState<DB>,
>(pub Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>);

impl Default for OpContext {
    fn default() -> Self {
        Self(
            Context::default()
                .with_tx(OpTransaction::default())
                .with_cfg(CfgEnv::new().with_spec(OpSpec::Op(OpSpecId::BEDROCK)))
                .with_chain(L1BlockInfo::default()),
        )
    }
}

impl OpContext {
    pub fn default_ctx() -> Context<
        BlockEnv,
        OpTransaction<TxEnv>,
        CfgEnv<OpSpec>,
        EmptyDB,
        JournaledState<EmptyDB>,
        L1BlockInfo,
    > {
        Context::default()
            .with_tx(OpTransaction::default())
            .with_cfg(CfgEnv::new().with_spec(OpSpec::Op(OpSpecId::BEDROCK)))
            .with_chain(L1BlockInfo::default())
    }
}

impl<BLOCK: Block, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>> BlockGetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type Block = BLOCK;

    fn block(&self) -> &Self::Block {
        self.0.block()
    }
}

impl<
        BLOCK: Block,
        TX: OpTxTrait,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    > EthContext for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
}

impl<
        BLOCK: Block,
        TX: OpTxTrait,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    > EthContext for &mut OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
}

impl<BLOCK: Block, TX: Transaction, CFG, DB: Database, JOURNAL: Journal<Database = DB>> ErrorGetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type Error = JournalDBError<Self>;

    fn take_error(&mut self) -> Result<(), Self::Error> {
        self.0.take_error()
    }
}

impl<BLOCK: Block, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>> BlockSetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    fn set_block(&mut self, block: Self::Block) {
        self.0.set_block(block)
    }
}

impl<BLOCK: Block, TX: OpTxTrait, CFG, DB: Database, JOURNAL: Journal<Database = DB>> OpTxGetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type OpTransaction = TX;

    fn op_tx(&self) -> &Self::OpTransaction {
        self.0.tx()
    }
}

impl<BLOCK, TX: Transaction, CFG, DB: Database, JOURNAL: Journal<Database = DB>> TransactionGetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type Transaction = TX;

    fn tx(&self) -> &Self::Transaction {
        self.0.tx()
    }
}

impl<BLOCK, TX: Transaction, CFG, DB: Database, JOURNAL: Journal<Database = DB>> TransactionSetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    fn set_tx(&mut self, tx: Self::Transaction) {
        self.0.set_tx(tx)
    }
}

impl<BLOCK, TX: Transaction, CFG, DB: Database, JOURNAL: Journal<Database = DB> + JournalExt>
    JournalExtGetter for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type JournalExt = JOURNAL;

    fn journal_ext(&self) -> &Self::JournalExt {
        self.0.journal_ref()
    }
}

impl<BLOCK, TX, CFG: Cfg, DB: Database, JOURNAL: Journal<Database = DB>> CfgGetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type Cfg = CFG;

    fn cfg(&self) -> &Self::Cfg {
        self.0.cfg()
    }
}

impl<BLOCK, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>> L1BlockInfoGetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    fn l1_block_info(&self) -> &L1BlockInfo {
        self.0.l1_block_info()
    }

    fn l1_block_info_mut(&mut self) -> &mut L1BlockInfo {
        self.0.l1_block_info_mut()
    }
}

impl<BLOCK, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>> DatabaseGetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type Database = DB;

    fn db(&mut self) -> &mut Self::Database {
        self.0.db()
    }

    fn db_ref(&self) -> &Self::Database {
        self.0.db_ref()
    }
}

impl<BLOCK, TX, CFG, DB: Database, JOURNAL: Journal<Database = DB>> JournalGetter
    for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type Journal = JOURNAL;

    fn journal(&mut self) -> &mut Self::Journal {
        self.0.journal()
    }

    fn journal_ref(&self) -> &Self::Journal {
        self.0.journal_ref()
    }
}

impl<BLOCK: Block, TX: Transaction, CFG: Cfg, DB: Database, JOURNAL: Journal<Database = DB>>
    PerformantContextAccess for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
{
    type Error = JournalDBError<Self>;

    fn load_access_list(&mut self) -> Result<(), Self::Error> {
        self.0.load_access_list()
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL> Host for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: Journal<Database = DB>,
{
    fn set_error(&mut self, error: DB::Error) {
        self.0.set_error(error)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL> OpContext<BLOCK, TX, CFG, DB, JOURNAL>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: Journal<Database = DB>,
{
    pub fn new(context: Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>) -> Self {
        Self(context)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL> ExecuteEvm for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
where
    BLOCK: Block,
    TX: OpTxTrait,
    CFG: Cfg<Spec = OpSpec>,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
{
    type Output =
        Result<ResultAndState<OpHaltReason>, EVMError<<DB as Database>::Error, OpTransactionError>>;

    fn exec_previous(&mut self) -> Self::Output {
        transact_op(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL> ExecuteCommitEvm for OpContext<BLOCK, TX, CFG, DB, JOURNAL>
where
    BLOCK: Block,
    TX: OpTxTrait,
    CFG: Cfg<Spec = OpSpec>,
    DB: Database + DatabaseCommit,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
{
    type CommitOutput = Result<
        ExecutionResult<OpHaltReason>,
        EVMError<<DB as Database>::Error, OpTransactionError>,
    >;

    fn exec_commit_previous(&mut self) -> Self::CommitOutput {
        transact_op(self).map(|r| {
            self.db().commit(r.state);
            r.result
        })
    }
}

#[cfg(test)]
mod test {

    use crate::api::into_optimism::{DefaultOp, IntoOptimism};

    use super::*;

    #[test]
    fn test_run() {
        let mut ctx = Context::default();
        // run default tx for mainnet;
        let _ = ctx.exec_previous().unwrap();

        let ctx = Context::default_op();
        // convert to optimism context
        let mut op_ctx = ctx.into_optimism();
        // modify gas limit.
        op_ctx.modify_tx(|tx| {
            tx.base.gas_limit = 1000;
        });
        // run default tx for optimism;
        let _ = op_ctx.exec_previous();
    }
}
