use crate::SpecId;

pub(crate) trait NotStaticSpec {}

pub trait Spec {
    /// litle bit of magic. We can have child version of Spec that contains static flag enabled
    type STATIC: Spec;

    #[inline(always)]
    fn enabled(spec_id: SpecId) -> bool {
        Self::SPEC_ID as u8 >= spec_id as u8
    }
    const SPEC_ID: SpecId;
    /// static flag used in STATIC type;
    const IS_STATIC_CALL: bool;
}

macro_rules! spec {
    ($spec_id:tt) => {
        #[allow(non_snake_case)]
        mod $spec_id {
            use super::{NotStaticSpec, Spec};
            use crate::SpecId;

            pub struct SpecInner<const STATIC_CALL: bool>;

            pub type SpecImpl = SpecInner<false>;
            pub type SpecStaticImpl = SpecInner<true>;

            impl NotStaticSpec for SpecImpl {}

            impl<const IS_STATIC_CALL: bool> Spec for SpecInner<IS_STATIC_CALL> {
                type STATIC = SpecInner<true>;

                //specification id
                const SPEC_ID: SpecId = SpecId::$spec_id;

                const IS_STATIC_CALL: bool = IS_STATIC_CALL;
            }
        }
    };
}

spec!(LATEST);
spec!(LONDON);
spec!(BERLIN);
spec!(ISTANBUL);
spec!(BYZANTINE);
spec!(FRONTIER);

pub use BERLIN::SpecImpl as BerlinSpec;
pub use BYZANTINE::SpecImpl as ByzantineSpec;
pub use FRONTIER::SpecImpl as FrontierSpec;
pub use ISTANBUL::SpecImpl as IstanbulSpec;
pub use LATEST::SpecImpl as LatestSpec;
pub use LONDON::SpecImpl as LondonSpec;
