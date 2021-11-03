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

    const USE_GAS: bool; 
}

macro_rules! spec {
    ($spec_id:tt) => {
        #[allow(non_snake_case)]
        mod $spec_id {
            use super::{NotStaticSpec, Spec};
            use crate::SpecId;

            pub struct SpecInner<const STATIC_CALL: bool, const USE_GAS: bool>;

            pub type SpecImpl<const USE_GAS: bool> = SpecInner<false,USE_GAS>;
            pub type SpecStaticImpl<const USE_GAS: bool> = SpecInner<true,USE_GAS>;

            impl<const USE_GAS:bool> NotStaticSpec for SpecImpl<USE_GAS> {}

            impl<const IS_STATIC_CALL: bool, const USE_GAS: bool> Spec for SpecInner<IS_STATIC_CALL,USE_GAS> {
                type STATIC = SpecInner<true, USE_GAS>;

                //specification id
                const SPEC_ID: SpecId = SpecId::$spec_id;

                const IS_STATIC_CALL: bool = IS_STATIC_CALL;

                const USE_GAS: bool = USE_GAS;
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
pub use LONDON::SpecImpl as LondonSpec;
pub use LATEST::SpecImpl as LatestSpec;
