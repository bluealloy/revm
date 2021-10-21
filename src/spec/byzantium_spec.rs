use crate::{NotStaticSpec, SpecId};

use super::Spec;

#[derive(Clone)]
pub struct ByzantiumSpecImpl<const STATIC_CALL: bool>;

pub type ByzantiumSpec = ByzantiumSpecImpl<false>;
pub type ByzantiumSpecStatic = ByzantiumSpecImpl<true>;

impl NotStaticSpec for ByzantiumSpec {}

impl<const IS_STATIC_CALL: bool> Spec for ByzantiumSpecImpl<IS_STATIC_CALL> {
    type STATIC = ByzantiumSpecImpl<true>;

    //specification id
    const SPEC_ID: SpecId = SpecId::BYZANTINE;

    const IS_STATIC_CALL: bool = IS_STATIC_CALL;
}
