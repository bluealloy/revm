use crate::{NotStaticSpec, SpecId};

use super::Spec;

#[derive(Clone)]
pub struct IstanbulSpecImpl<const STATIC_CALL: bool>;


pub type IstanbulSpec = IstanbulSpecImpl<false>;
pub type IstanbulSpecStatic = IstanbulSpecImpl<true>;


impl NotStaticSpec for IstanbulSpec {}

impl<const IS_STATIC_CALL: bool> Spec for IstanbulSpecImpl<IS_STATIC_CALL> {
    type STATIC = IstanbulSpecImpl<true>;
    
    //specification id
    const SPEC_ID: SpecId = SpecId::ISTANBUL;

    const IS_STATIC_CALL: bool = IS_STATIC_CALL;
}
