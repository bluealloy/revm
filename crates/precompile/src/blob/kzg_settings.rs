use alloc::boxed::Box;
use c_kzg::KzgSettings;
use once_cell::race::OnceBox;

#[rustfmt::skip]
mod generated;

/// Note: the type of this is an implementation detail, so we don't expose it in the public API.
static GLOBAL_KZG_SETTINGS: OnceBox<KzgSettings> = OnceBox::new();

/// Returns a reference to the global [`KzgSettings`] instance, initializing it with the default
/// value if it was not previously set.
pub fn get_global_or_default() -> &'static KzgSettings {
    GLOBAL_KZG_SETTINGS.get_or_init(|| Box::new(default()))
}

/// Returns a reference to the global [`KzgSettings`] instance, initializing it with `f` if it was
/// not previously set.
pub fn get_global_or_init<F: FnOnce() -> Box<KzgSettings>>(f: F) -> &'static KzgSettings {
    GLOBAL_KZG_SETTINGS.get_or_init(f)
}

/// Sets the given [KzgSettings] as the global instance.
///
/// Returns `Ok(())` if the cell was empty and `Err(value)` if it was full.
pub fn set_global(value: Box<KzgSettings>) -> Result<(), Box<KzgSettings>> {
    GLOBAL_KZG_SETTINGS.set(value)
}

/// Creates a new a [KzgSettings] instance by initializing it with the default value.
pub fn default() -> KzgSettings {
    KzgSettings::load_trusted_setup(generated::G1_POINTS, generated::G2_POINTS)
        .expect("failed to load default trusted setup")
}
