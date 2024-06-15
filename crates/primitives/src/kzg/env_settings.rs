use super::{
    // trusted_setup_points::{G1_POINTS, G2_POINTS},
    trusted_setup_points::KzgErrors,
    KZGSettings,
};
use core::{
    fmt,
    hash::{Hash, Hasher},
    mem::MaybeUninit,
};
use kzg::eip_4844::{
    load_trusted_setup_rust, BYTES_PER_G1, BYTES_PER_G2, C_KZG_RET_OK, FIELD_ELEMENTS_PER_BLOB,
    TRUSTED_SETUP_NUM_G2_POINTS,
};
use once_cell::{race::OnceBox, unsync::OnceCell};
use rust_kzg_zkcrypto::eip_4844::load_trusted_setup;
use std::{boxed::Box, sync::Arc};

/// KZG Settings that allow us to specify a custom trusted setup.
/// or use hardcoded default settings.
#[derive(Debug, Clone, Default)]
pub enum EnvKzgSettings {
    /// Default mainnet trusted setup
    #[default]
    Default,
    /// Custom trusted setup.
    Custom(Arc<KZGSettings>),
}

// Implement PartialEq and Hash manually because `KZGSettings` does not implement them
impl PartialEq for EnvKzgSettings {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Default, Self::Default) => true,
            (Self::Custom(a), Self::Custom(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for EnvKzgSettings {}

impl Hash for EnvKzgSettings {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            Self::Default => {}
            Self::Custom(settings) => Arc::as_ptr(settings).hash(state),
        }
    }
}

impl EnvKzgSettings {
    /// Return set KZG settings.
    ///
    /// In will initialize the default settings if it is not already loaded.
    pub fn get(&self) -> &KZGSettings {
        match self {
            Self::Default => {
                static DEFAULT: OnceBox<KZGSettings> = OnceBox::new();
                DEFAULT.get_or_init(|| {
                    let settings = load_trusted_setup_helper(
                        include_bytes!("./g1_points.bin"),
                        include_bytes!("./g2_points.bin"),
                    )
                    .unwrap();
                    Box::new(settings)
                })
            }
            Self::Custom(settings) => settings,
        }
    }
}

fn load_trusted_setup_helper(g1_bytes: &[u8], g2_bytes: &[u8]) -> Result<KZGSettings, KzgErrors> {
    if g1_bytes.len() != FIELD_ELEMENTS_PER_BLOB * BYTES_PER_G1 {
        return Err(KzgErrors::ParseError);
    }
    if g2_bytes.len() != TRUSTED_SETUP_NUM_G2_POINTS * BYTES_PER_G2 {
        return Err(KzgErrors::ParseError);
    }
    let mut kzg_settings = MaybeUninit::<KZGSettings>::uninit();
    unsafe {
        if load_trusted_setup(
            kzg_settings.as_mut_ptr(),
            g1_bytes.as_ptr().cast(),
            g1_bytes.len(),
            g2_bytes.as_ptr().cast(),
            g2_bytes.len(),
        ) != C_KZG_RET_OK
        {
            return Err(KzgErrors::NotValidFile);
        }
        Ok(kzg_settings.assume_init())
    }
}
