use crate::{
    primitives::{specification, EVMError, EVMResult, Env},
    r#impl::{EVMImpl, Transact},
};
use alloc::boxed::Box;
use fluentbase_types::ExitCode;

/// Struct that takes Database and enabled transact to update state directly to database.
/// additionally it allows user to set all environment parameters.
///
/// Parameters that can be set are divided between Config, Block and Transaction(tx)
///
/// For transacting on EVM you can call transact_commit that will automatically apply changes to db.
///
/// You can do a lot with rust and traits. For Database abstractions that we need you can implement,
/// Database, DatabaseRef or Database+DatabaseCommit and they enable functionality depending on what
/// kind of handling of struct you want.
/// * Database trait has mutable self in its functions. It is usefully if on get calls you want to
///   modify
/// your cache or update some statistics. They enable `transact` and `inspect` functions
/// * DatabaseRef takes reference on object, this is useful if you only have reference on state and
///   don't
/// want to update anything on it. It enabled `transact_ref` and `inspect_ref` functions
/// * Database+DatabaseCommit allow directly committing changes of transaction. it enabled
///   `transact_commit`
/// and `inspect_commit`
///
/// /// # Example
///
/// ```
/// # use revm_rwasm::RWASM;        // Assuming this struct is in 'your_crate_name'
/// # struct SomeDatabase;  // Mocking a database type for the purpose of this example
/// # struct Env;           // Assuming the type Env is defined somewhere
///
/// let evm: RWASM = RWASM::new();
/// ```
#[derive(Clone, Debug)]
pub struct RWASM {
    pub env: Env,
}

pub fn new() -> RWASM {
    RWASM::new()
}

impl Default for RWASM {
    fn default() -> Self {
        Self::new()
    }
}

impl RWASM {
    /// Do checks that could make transaction fail before call/create
    pub fn preverify_transaction(&mut self) -> Result<(), EVMError<ExitCode>> {
        evm_inner(&mut self.env).preverify_transaction()
    }

    /// Skip preverification steps and execute transaction without writing to DB, return change
    /// state.
    pub fn transact_preverified(&mut self) -> EVMResult<ExitCode> {
        evm_inner(&mut self.env).transact_preverified()
    }

    /// Execute transaction without writing to DB, return change state.
    pub fn transact(&mut self) -> EVMResult<ExitCode> {
        evm_inner(&mut self.env).transact()
    }
}

impl RWASM {
    /// Creates a new [RWASM] instance with the default environment,
    pub fn new() -> Self {
        Self::with_env(Default::default())
    }

    /// Creates a new [RWASM] instance with the given environment.
    pub fn with_env(env: Env) -> Self {
        Self { env }
    }
}

pub fn evm_inner<'a>(env: &'a mut Env) -> Box<dyn Transact<ExitCode> + 'a> {
    macro_rules! create_evm {
        ($spec:ident) => {
            Box::new(EVMImpl::<'a, $spec>::new(env)) as Box<dyn Transact<ExitCode> + 'a>
        };
    }

    use specification::*;
    match env.cfg.spec_id {
        FRONTIER | FRONTIER_THAWING => create_evm!(FrontierSpec),
        HOMESTEAD | DAO_FORK => create_evm!(HomesteadSpec),
        TANGERINE => create_evm!(TangerineSpec),
        SPURIOUS_DRAGON => create_evm!(SpuriousDragonSpec),
        BYZANTIUM => create_evm!(ByzantiumSpec),
        PETERSBURG | CONSTANTINOPLE => create_evm!(PetersburgSpec),
        ISTANBUL | MUIR_GLACIER => create_evm!(IstanbulSpec),
        BERLIN => create_evm!(BerlinSpec),
        LONDON | ARROW_GLACIER | GRAY_GLACIER => {
            create_evm!(LondonSpec)
        }
        MERGE => create_evm!(MergeSpec),
        SHANGHAI => create_evm!(ShanghaiSpec),
        CANCUN => create_evm!(CancunSpec),
        LATEST => create_evm!(LatestSpec),
        _ => unreachable!("not supported spec id"),
    }
}
