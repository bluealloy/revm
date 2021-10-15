use crate::{NotStaticSpec, SpecId};

use super::Spec;

#[derive(Clone)]
pub struct LatestSpecImpl<const STATIC_CALL: bool>;


pub type LatestSpec = LatestSpecImpl<false>;
pub type LatestSpecStatic = LatestSpecImpl<true>;


impl NotStaticSpec for LatestSpec {}


impl<const IS_STATIC_CALL: bool> Spec for LatestSpecImpl<IS_STATIC_CALL> {
    type STATIC = LatestSpecImpl<true>;
    
    //specification id
    const SPEC_ID: SpecId = SpecId::LATEST;

    const IS_STATIC_CALL: bool = IS_STATIC_CALL;
}
