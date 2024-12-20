pub trait PreExecutionHandler {
    type Context;
    type Error;

    fn load_accounts(&self, context: &mut Self::Context) -> Result<(), Self::Error>;

    fn apply_eip7702_auth_list(&self, context: &mut Self::Context) -> Result<u64, Self::Error>;

    fn deduct_caller(&self, context: &mut Self::Context) -> Result<(), Self::Error>;
}
