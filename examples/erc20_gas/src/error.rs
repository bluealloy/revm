use revm::{
    context_interface::{result::{EVMError, InvalidTransaction}, JournalStateGetterDBError},
    Context,
};

pub type Erc20Error = EVMError<JournalStateGetterDBError<Context>, InvalidTransaction>; 