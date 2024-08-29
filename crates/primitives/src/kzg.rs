cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        pub use c_kzg::KzgSettings;
        /// KZG Settings that allow us to specify a custom trusted setup.
        /// or use hardcoded default settings.
        #[derive(Debug, Clone, Default, PartialEq, Eq )]
        pub enum EnvKzgSettings {
            /// Default mainnet trusted setup
            #[default]
            Default,
            /// Custom trusted setup.
            Custom(std::sync::Arc<c_kzg::KzgSettings>),
        }

        impl EnvKzgSettings {
            /// Return set KZG settings.
            ///
            /// In will initialize the default settings if it is not already loaded.
            pub fn get(&self) -> &c_kzg::KzgSettings {
                match self {
                    Self::Default => {
                        c_kzg::ethereum_kzg_settings()
                    }
                    Self::Custom(settings) => settings,
                }
            }
        }
    } else if #[cfg(feature = "kzg-rs")] {
        pub use kzg_rs::{KzgSettings,EnvKzgSettings};
    }
}
