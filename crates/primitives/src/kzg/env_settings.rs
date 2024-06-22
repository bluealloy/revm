use super::{
    trusted_setup_points::{G1_POINTS, G2_POINTS},
    KzgSettings,
};
use once_cell::race::OnceBox;
use std::{boxed::Box, sync::Arc};

/// KZG Settings that allow us to specify a custom trusted setup.
/// or use hardcoded default settings.
#[cfg(not(feature = "c-kzg"))]
pub use kzg_rs::EnvKzgSettings;

#[cfg(feature = "c-kzg")]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum EnvKzgSettings {
    /// Default mainnet trusted setup
    #[default]
    Default,
    /// Custom trusted setup.
    Custom(Arc<c_kzg::KzgSettings>),
}

#[cfg(feature = "c-kzg")]
impl EnvKzgSettings {
    /// Return set KZG settings.
    ///
    /// In will initialize the default settings if it is not already loaded.
    pub fn get(&self) -> &KzgSettings {
        match self {
            Self::Default => {
                static DEFAULT: OnceBox<KzgSettings> = OnceBox::new();
                DEFAULT.get_or_init(|| {
                    let settings =
                        KzgSettings::load_trusted_setup(G1_POINTS.as_ref(), G2_POINTS.as_ref())
                            .expect("failed to load default trusted setup");
                    Box::new(settings)
                })
            }
            Self::Custom(settings) => settings,
        }
    }
}
