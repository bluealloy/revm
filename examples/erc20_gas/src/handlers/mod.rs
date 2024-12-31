pub mod post_execution;
pub mod pre_execution;
pub mod validation;

pub use post_execution::Erc20PostExecution;
pub use pre_execution::Erc20PreExecution;
pub use validation::Erc20Validation;

use revm::{
    context::{block::BlockEnv, tx::TxEnv, CfgEnv, Context},
    context_interface::result::{EVMError, InvalidTransaction},
    database_interface::Database,
    handler::{EthExecution, EthHandler},
    Evm,
};

pub type Erc20GasError<DB> = EVMError<<DB as Database>::Error, InvalidTransaction>;

pub type Erc20GasContext<DB> = Context<BlockEnv, TxEnv, CfgEnv, DB>;

pub type CustomHandler<
    CTX,
    ERROR,
    VAL = Erc20Validation<CTX, ERROR>,
    PREEXEC = Erc20PreExecution<CTX, ERROR>,
    EXEC = EthExecution<CTX, ERROR>,
    POSTEXEC = Erc20PostExecution<CTX, ERROR>,
> = EthHandler<CTX, ERROR, VAL, PREEXEC, EXEC, POSTEXEC>;

pub type CustomEvm<DB> = Evm<
    Erc20GasError<DB>,
    Erc20GasContext<DB>,
    CustomHandler<Erc20GasContext<DB>, Erc20GasError<DB>>,
>;
