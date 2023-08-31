use c_kzg::KzgSettings;
use once_cell::sync::OnceCell;

#[rustfmt::skip]
mod generated;

/// Note: the type of this is an implementation detail, so we don't expose it in the public API.
static GLOBAL_KZG_SETTINGS: OnceCell<KzgSettings> = OnceCell::new();

/// Returns a reference to the global [`KzgSettings`] instance, initializing it with the default
/// value if it was not previously set.
pub fn get_global_or_default() -> &'static KzgSettings {
    GLOBAL_KZG_SETTINGS.get_or_init(default)
}

/// Returns a reference to the global [`KzgSettings`] instance, initializing it with `f` if it was
/// not previously set.
pub fn get_global_or_init<F: FnOnce() -> KzgSettings>(f: F) -> &'static KzgSettings {
    GLOBAL_KZG_SETTINGS.get_or_init(f)
}

/// Sets the given KZG settings as the global instance.
///
/// Returns `Ok(())` if the cell was empty and `Err(value)` if it was full.
pub fn set_global(value: KzgSettings) -> Result<(), KzgSettings> {
    GLOBAL_KZG_SETTINGS.set(value)
}

/// Creates a new a KZG settings instance by initializing it with the default value.
pub fn default() -> KzgSettings {
    c_kzg::KzgSettings::load_trusted_setup(generated::G1_POINTS, generated::G2_POINTS)
        .expect("failed to load default trusted setup")
}
