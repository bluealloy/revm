pub trait PostExecutionHandler {
    type Context;
    type Error;
    type ExecResult;
    type Output;

    /// Calculate final refund
    fn refund(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
        eip7702_refund: i64,
    );

    /// Reimburse the caller with balance it didn't spent.
    fn reimburse_caller(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error>;

    /// Reward beneficiary with transaction rewards.
    fn reward_beneficiary(
        &self,
        context: &mut Self::Context,
        exec_result: &mut Self::ExecResult,
    ) -> Result<(), Self::Error>;

    /// Main return handle, takes state from journal and transforms internal result to [`PostExecutionHandler::Output`].
    fn output(
        &self,
        context: &mut Self::Context,
        result: Self::ExecResult,
    ) -> Result<Self::Output, Self::Error>;

    /// Called when execution ends.
    ///
    /// End handle in comparison to output handle will be called every time after execution.
    /// While [`PostExecutionHandler::output`] will be omitted in case of the error.
    fn end(
        &self,
        _context: &mut Self::Context,
        end_output: Result<Self::Output, Self::Error>,
    ) -> Result<Self::Output, Self::Error> {
        end_output
    }

    /// Clean handler. This handle is called every time regardless
    /// of the result of the transaction.
    fn clear(&self, context: &mut Self::Context);
}
