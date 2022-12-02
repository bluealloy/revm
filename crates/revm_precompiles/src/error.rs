#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Error {
    /// out of gas is the main error. Other are just here for completness
    OutOfGas,
    // Blake2 erorr
    Blake2WrongLength,
    Blake2WrongFinalIndicatorFlag,
    // Modexp errors
    ModexpExpOverflow,
    ModexpBaseOverflow,
    ModexpModOverflow,
    // Bn128 errors
    Bn128FieldPointNotAMember,
    Bn128AffineGFailedToCreate,
    Bn128PairLength,
}
