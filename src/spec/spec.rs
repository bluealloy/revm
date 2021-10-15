use crate::SpecId;
pub(crate) use crate::precompiles::Precompiles;


pub trait NotStaticSpec {}


pub trait Spec {
    /// litle bit of magic. We can have child version of Spec that contains static flag enabled
    type STATIC: Spec;
    
    fn enabled(spec_id: SpecId) -> bool {
        Self::SPEC_ID as u8 <= spec_id as u8
    }

    const SPEC_ID: SpecId;
    /// static flag used in STATIC type;
    const IS_STATIC_CALL: bool;

    // Whether to throw out of gas error when
    // CALL/CALLCODE/DELEGATECALL requires more than maximum amount
    // of gas.
    // TODO check this it was false from ISTANBUL const ERR_ON_CALL_WITH_MORE_GAS: bool;
    
    // Has create2.
    //TODO add it const HAS_CREATE2: bool;
}
