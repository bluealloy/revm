use crate::{NotStaticSpec, SpecId};

use super::Spec;

#[derive(Clone)]
pub struct FrontierSpecImpl<const STATIC_CALL: bool>;

pub type FrontierSpec = FrontierSpecImpl<false>;
pub type FrontierSpecStatic = FrontierSpecImpl<true>;

impl NotStaticSpec for FrontierSpec {}

impl<const IS_STATIC_CALL: bool> Spec for FrontierSpecImpl<IS_STATIC_CALL> {
    type STATIC = FrontierSpecImpl<true>;

    //specification id
    const SPEC_ID: SpecId = SpecId::FRONTIER;

    const IS_STATIC_CALL: bool = IS_STATIC_CALL;
}
