//! Revm is a Rust EVM implementation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

// reexport dependencies
pub use bytecode;
pub use context;
pub use context_interface;
pub use database;
pub use database_interface;
pub use handler;
pub use inspector;
pub use interpreter;
pub use precompile;
pub use primitives;
pub use state;

// Export items.

pub use context::journal::{Journal, JournalEntry};
pub use context::Context;
pub use database_interface::{Database, DatabaseCommit, DatabaseRef};
pub use handler::{
    ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext, MainnetEvm, SystemCallCommitEvm,
    SystemCallEvm,
};
pub use inspector::{InspectCommitEvm, InspectEvm, Inspector};

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use context::TxEnv;
    use database::{BenchmarkDB, BENCH_CALLER, BENCH_TARGET};
    use inspector::inspectors::TracerEip3155;
    use primitives::{hex, TxKind};
    use state::Bytecode;

    use super::*;

    #[test]
    fn test_modexp_perf() {
        let mut init = hex!("61020060005260066020526102006040527f00000000000000000000000000000000000000000000000000000000000000006060527f00000000000000000000000000000000000000000000000000000000000000006080527f000000000000000000000000000000000000000000000000000000000000000060a0527f000000000000000000000000000000000000000000000000000000000000000060c0527f000000000000000000000000000000000000000000000000000000000000000060e0527f0000000000000000000000000000000000000000000000000000000000000000610100527f00000000000000000000000000000000000000000000000000000000eeeeeeee610120527f0000000000000000000000000000000000000000000000000000000000000000610140527f0000000000000000000000000000000000000000000000000000000000000000610160527f0000000000000000000000000000000000000000000000000000000000000000610180527f00000000000000000000000000000000000000000000000000000000000000006101a0527f00000000000000000000000000000000000000000000000000000000000000006101c0527f00000000000000000000000000000000000000000000000000000000000000006101e0527f0000000000000000000000000000000000000000000000000000000000000000610200523261022052600140610240527f001f21f020ca0000000000000000000000000000000000000000000000000000600260527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610242527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610286527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada6102a6527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada6102c6527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada6102e6527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610306527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610326527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610346527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610366527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610356527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada6103a6527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada6103c6527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada6103e6527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610406527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada610426527fdadadadadadadadadadadadadadadadadadadadadadadadadadadadadadadada61044652").to_vec();
        let mail_loop = hex!("5b610200606061046660006000600561c3505a03f15061049356").to_vec();

        init.extend(mail_loop);

        let mut evm = Context::mainnet()
            .with_db(BenchmarkDB::new_bytecode(Bytecode::new_legacy(init.into())))
            .build_mainnet()
            .with_inspector(TracerEip3155::new_stdout());

        let time = Instant::now();
        let res = evm
            .transact(TxEnv {
                caller: BENCH_CALLER,
                kind: TxKind::Call(BENCH_TARGET),
                gas_limit: 30_000_000,
                ..Default::default()
            })
            .unwrap();
        let elapsed = time.elapsed();
        
        println!("res: {res:?}");
        println!("\ntime: {:?}", elapsed);
    }
}
