use crate::{
    handler::mainnet::{EthExecution, EthPostExecution, EthPreExecution, EthValidation},
    //ƒEvmHandler,
};
use context::{Context, JournaledState};
use database_interface::Database;
use interpreter::table::InstructionTables;
use specification::spec_to_generic;
use std::fmt::Debug;
use std::vec::Vec;
use wiring::{
    journaled_state::JournaledState as JournaledStateTrait,
    result::{EVMError, EVMErrorWiring, HaltReason},
    EthereumWiring, EvmWiring as PrimitiveEvmWiring, Transaction,
};

// pub trait EvmWiring: PrimitiveEvmWiring {
//     /// Creates a new handler with the given hardfork.
//     fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>;
// }

// impl<DB: Database, EXT: Debug> EvmWiring for EthereumWiring<DB, EXT> {
//     fn handler<'evm>(hardfork: Self::Hardfork) -> EvmHandler<'evm, Self>
//     where
//         DB: Database,
//     {
//         let spec_id = hardfork;
//         spec_to_generic!(
//             hardfork,
//             EvmHandler {
//                 spec_id: hardfork,
//                 //instruction_table: InstructionTables::new_plain::<SPEC>(),
//                 registers: Vec::new(),
//                 validation: EthValidation::<
//                     Context<Self>,
//                     EVMError<
//                         <<JournaledState<DB> as JournaledStateTrait>::Database as Database>::Error,
//                         <<Self as PrimitiveEvmWiring>::Transaction as Transaction>::TransactionError,
//                     >,
//                 >::new_boxed(spec_id),
//                 pre_execution: EthPreExecution::<
//                 Context<Self>,
//                 EVMError<
//                     <<JournaledState<DB> as JournaledStateTrait>::Database as Database>::Error,
//                     <<Self as PrimitiveEvmWiring>::Transaction as Transaction>::TransactionError,
//                 >>::new_boxed(spec_id),
//                 post_execution: EthPostExecution::<Context<Self>,EVMErrorWiring<Self>, HaltReason>::new_boxed(spec_id),
//                 execution: EthExecution::<Context<Self>, EVMErrorWiring<Self>>::new_boxed(spec_id),
//             }
//         )
//     }
// }
