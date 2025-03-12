use std::convert::Infallible;

use revm::{
    context::{
        result::{EVMError, ExecutionResult},
        BlockEnv, CfgEnv, TxEnv,
    },
    database::State,
    inspector::NoOpInspector,
    primitives::B256,
    Database, DatabaseCommit, ExecuteCommitEvm, Inspector, Journal, JournalEntry, MainBuilder as _,
};

pub type ContextRef<'db> = revm::Context<
    BlockEnv,
    TxEnv,
    CfgEnv,
    &'db mut dyn Database<Error = Infallible>,
    Journal<&'db mut dyn Database<Error = Infallible>>,
>;

pub trait DBSuperTrait: Database<Error = Infallible> + DatabaseCommit {}

impl<T: Database<Error = Infallible> + DatabaseCommit> DBSuperTrait for T {}

fn mine_block<'db, 'inspector, InspectorT>(
    cfg: &CfgEnv,
    db: &'db mut dyn DBSuperTrait,
    transactions: Vec<TxEnv>,
    inspector: &'inspector mut InspectorT,
) -> Result<Vec<ExecutionResult>, EVMError<Infallible>>
where
    'db: 'inspector,
    InspectorT: Inspector<ContextRef<'inspector>>,
{
    let block = BlockEnv {
        prevrandao: Some(B256::random()),
        ..BlockEnv::default()
    };

    let mut results = Vec::new();
    for tx in transactions {
        let result = run_transaction(&block, cfg, &mut *db, tx, &mut *inspector)?;

        results.push(result);
    }

    Ok(results)
}

fn run_transaction<'db, 'inspector, InspectorT>(
    block: &BlockEnv,
    cfg: &CfgEnv,
    db: &'db mut dyn DBSuperTrait,
    tx: TxEnv,
    inspector: &'inspector mut InspectorT,
) -> Result<ExecutionResult, EVMError<Infallible>>
where
    'db: 'inspector,
    InspectorT: Inspector<ContextRef<'inspector>>,
{
    let context = revm::Context {
        block: block.clone(),
        tx,
        cfg: cfg.clone(),
        journaled_state: Journal::<_, JournalEntry>::new(cfg.spec, &mut *db),
        chain: (),
        error: Ok(()),
    };

    let mut evm = context.build_mainnet_with_inspector(&mut *inspector);
    evm.replay_commit()
}

fn main() -> anyhow::Result<()> {
    let cfg = CfgEnv::default();
    let mut db = State::builder().build();
    let transactions = vec![];

    let mut inspector = NoOpInspector;
    let results = mine_block(&cfg, &mut db, transactions, &mut inspector)?;
    println!("results: {results:?}");

    Ok(())
}
