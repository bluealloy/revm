pub mod post_execution;
pub mod pre_execution;
pub mod validation;

use revm::{
    context::{block::BlockEnv, tx::TxEnv, CfgEnv, Context},
    context_interface::result::{EVMError, InvalidTransaction},
    database_interface::Database,
    handler::{EthExecution, EthHandler},
    Evm,
};

pub use post_execution::Erc20PostExecution;
pub use pre_execution::Erc20PreExecution;
pub use validation::Erc20Validation;

pub type Erc20Error<DB> = EVMError<<DB as Database>::Error, InvalidTransaction>;

pub type Erc20Context<DB> = Context<BlockEnv, TxEnv, CfgEnv, DB>;

pub type Erc20Handler<
    CTX,
    ERROR,
    VAL = Erc20Validation<CTX, ERROR>,
    PREEXEC = Erc20PreExecution<CTX, ERROR>,
    EXEC = EthExecution<CTX, ERROR>,
    POSTEXEC = Erc20PostExecution<CTX, ERROR>,
> = EthHandler<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>;

pub type Erc20Evm<DB> =
    Evm<Erc20Error<DB>, Erc20Context<DB>, Erc20Handler<Erc20Context<DB>, Erc20Error<DB>>>;
