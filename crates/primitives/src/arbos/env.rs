#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArbOsCfg {
    pub arbos_version: u16,
    pub stylus_version: u16,
    pub max_depth: u32,
    pub ink_price: u32,
    pub debug_mode: bool,
}

impl Default for ArbOsCfg {
    fn default() -> Self {
        Self {
            arbos_version: 32,
            stylus_version: 1,
            max_depth: 4 * 65536,
            ink_price: 10000,
            debug_mode: false,
        }
    }
}
