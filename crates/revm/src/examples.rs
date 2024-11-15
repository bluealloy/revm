/************ CUSTOM ETH HANDLER *****************/

pub struct CustomEthHandler<CTX, ERROR> {
    main_eth_hand: EthHandler<CTX, ERROR>,
    execution: EthExecution<CTX, ERROR>,
}

impl<CTX, ERROR> Handler for CustomEthHandler<CTX, ERROR>
where
    CTX: TransactionGetter
        + BlockGetter
        + JournalStateGetter
        + CfgGetter
        + ErrorGetter<Error = ERROR>
        + JournalStateGetter<Journal: JournaledState<FinalOutput = (EvmState, Vec<Log>)>>
        + Host,
    ERROR: From<InvalidTransaction>
        + From<InvalidHeader>
        + From<JournalStateGetterDBError<CTX>>
        + From<PrecompileErrors>,
{
    type Validation = <EthHandler<CTX, ERROR> as Handler>::Validation;
    type PreExecution = <EthHandler<CTX, ERROR> as Handler>::PreExecution;
    type Execution = <EthHandler<CTX, ERROR> as Handler>::Execution;
    type PostExecution = <EthHandler<CTX, ERROR> as Handler>::PostExecution;

    fn validation(&mut self) -> &mut Self::Validation {
        self.main_eth_hand.validation()
    }

    fn pre_execution(&mut self) -> &mut Self::PreExecution {
        self.main_eth_hand.pre_execution()
    }

    fn execution(&mut self) -> &mut Self::Execution {
        self.main_eth_hand.execution()
    }

    fn post_execution(&mut self) -> &mut Self::PostExecution {
        self.main_eth_hand.post_execution()
    }
}
