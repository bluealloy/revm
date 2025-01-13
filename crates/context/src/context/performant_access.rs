use super::Context;
use context_interface::{Block, Cfg, Database, Journal, PerformantContextAccess, Transaction};
use primitives::U256;

impl<
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: Journal<Database = DB>,
        CHAIN,
    > PerformantContextAccess for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Error = <DB as Database>::Error;

    fn load_access_list(&mut self) -> Result<(), Self::Error> {
        let Some(access_list) = self.tx.access_list() else {
            return Ok(());
        };
        for access_list in access_list {
            self.journaled_state.warm_account_and_storage(
                *access_list.0,
                access_list.1.iter().map(|i| U256::from_be_bytes(i.0)),
            )?;
        }
        Ok(())
    }
}
