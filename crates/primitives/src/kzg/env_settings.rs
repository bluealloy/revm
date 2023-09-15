use super::{
    generated::{G1_POINTS, G2_POINTS},
    KzgSettings,
};
use alloc::{boxed::Box, sync::Arc};
use once_cell::race::OnceBox;

/// KZG Settings that allow us to specify a custom trusted setup.
/// or use hardcoded default settings.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum EnvKzgSettings {
    /// Default mainnet trusted setup
    #[default]
    Default,
    /// Custom trusted setup.
    Custom(Arc<c_kzg::KzgSettings>),
}

impl EnvKzgSettings {
    /// Return set KZG settings.
    ///
    /// In will initialize the default settings if it is not already loaded.
    pub fn get(&self) -> &KzgSettings {
        match self {
            Self::Default => {
                static DEFAULT: OnceBox<KzgSettings> = OnceBox::new();
                DEFAULT.get_or_init(|| {
                    let settings = KzgSettings::load_trusted_setup(G1_POINTS, G2_POINTS)
                        .expect("failed to load default trusted setup");
                    Box::new(settings)
                })
            }
            Self::Custom(settings) => settings,
        }
    }
}
