use crate::{NotStaticSpec, SpecId};

use super::Spec;

#[derive(Clone)]
pub struct BerlinSpecImpl<const STATIC_CALL: bool>;

pub type BerlinSpec = BerlinSpecImpl<false>;
pub type BerlinSpecStatic = BerlinSpecImpl<true>;

impl NotStaticSpec for BerlinSpec {}

impl<const IS_STATIC_CALL: bool> Spec for BerlinSpecImpl<IS_STATIC_CALL> {
    type STATIC = BerlinSpecImpl<true>;

    //specification id
    const SPEC_ID: SpecId = SpecId::BERLIN;

    const IS_STATIC_CALL: bool = IS_STATIC_CALL;
}
