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

    const ASSUME_PRECOMPILE_HAS_BALANCE: bool;
}

macro_rules! spec {
    ($spec_id:tt) => {
        #[allow(non_snake_case)]
        mod $spec_id {
            use super::{NotStaticSpec, Spec};
            use crate::SpecId;

            pub struct SpecInner<const STATIC_CALL: bool, const ASSUME_PRECOMPILE_HAS_BALANCE: bool>;

            pub type SpecImpl = SpecInner<false,true>;
            pub type SpecStaticImpl = SpecInner<true,true>;

            impl NotStaticSpec for SpecImpl {}

            impl<const IS_STATIC_CALL: bool,const ASSUME_PRECOMPILE_HAS_BALANCE: bool> Spec for SpecInner<IS_STATIC_CALL,ASSUME_PRECOMPILE_HAS_BALANCE> {
                type STATIC = SpecInner<true,ASSUME_PRECOMPILE_HAS_BALANCE>;

                //specification id
                const SPEC_ID: SpecId = SpecId::$spec_id;

                const IS_STATIC_CALL: bool = IS_STATIC_CALL;

                const ASSUME_PRECOMPILE_HAS_BALANCE: bool = ASSUME_PRECOMPILE_HAS_BALANCE;
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
