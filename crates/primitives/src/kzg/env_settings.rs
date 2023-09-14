use super::{
    generated::{G1_POINTS, G2_POINTS},
    KzgSettings,
};
use alloc::{boxed::Box, sync::Arc};
use once_cell::race::OnceBox;

/// KZG Settings.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub enum EnvKzgSettings {
    #[default]
    Default,
    Custom(Arc<c_kzg::KzgSettings>),
}

impl EnvKzgSettings {
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
