use crate::handler::CustomOpcodeHandler;
use revm::{
    context_interface::{
        result::{EVMError, HaltReason, InvalidTransaction, ResultAndState},
        DatabaseGetter,
    },
    database_interface::Database,
    handler::handler::{EthContext, EthHandler},
};

pub fn transact_custom_opcode<DB: Database, CTX: EthContext + DatabaseGetter<Database = DB>>(
    ctx: &mut CTX,
) -> Result<ResultAndState<HaltReason>, EVMError<<DB as Database>::Error, InvalidTransaction>> {
    CustomOpcodeHandler::<CTX, _>::new().run(ctx)
}
