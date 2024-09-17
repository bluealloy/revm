cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        pub use c_kzg::KzgSettings;
    } else if #[cfg(feature = "kzg-rs")] {
        pub use kzg_rs::KzgSettings;
    }
}

/// KZG Settings that allow us to specify a custom trusted setup.
/// or use hardcoded default settings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum EnvKzgSettings {
    /// Default mainnet trusted setup
    #[default]
    Default,
    /// Custom trusted setup.
    Custom(std::sync::Arc<KzgSettings>),
}

impl EnvKzgSettings {
    /// Return set KZG settings.
    ///
    /// In will initialize the default settings if it is not already loaded.
    pub fn get(&self) -> &KzgSettings {
        match self {
            Self::Default => {
                cfg_if::cfg_if! {
                    if #[cfg(feature = "c-kzg")] {
                        c_kzg::ethereum_kzg_settings()
                    } else if #[cfg(feature = "kzg-rs")] {
                        static DEFAULT: Once<KzgSettings> = Once::new();
                        DEFAULT.call_once(|| {
                            KzgSettings::load_trusted_setup_file()
                                .expect("failed to load default trusted setup")
                        })
                    } else {
                        unimplemented!()
                    }
                }
            }
            Self::Custom(settings) => settings,
        }
    }
}
