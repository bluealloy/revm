use super::{
    trusted_setup_points::{G1_POINTS, G2_POINTS},
    KzgSettings,
};
use once_cell::race::OnceBox;
use std::{boxed::Box, sync::Arc};

/// KZG Settings that allow us to specify a custom trusted setup.
/// or use hardcoded default settings.
#[cfg(feature = "kzg-rs")]
pub use kzg_rs::EnvKzgSettings;

#[cfg(not(feature = "kzg-rs"))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub enum EnvKzgSettings {
    /// Default mainnet trusted setup
    #[default]
    Default,
    /// Custom trusted setup.
    Custom(Arc<c_kzg::KzgSettings>),
}

#[cfg(not(feature = "kzg-rs"))]
impl EnvKzgSettings {
    /// Return set KZG settings.
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
