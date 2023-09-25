mod env_settings;
#[rustfmt::skip]
mod generated;

#[cfg(feature = "c_kzg")]
pub use c_kzg::KzgSettings;
#[cfg(feature = "c_kzg")]
pub use env_settings::EnvKzgSettings;
#[cfg(feature = "c_kzg")]
pub use generated::{
    BYTES_PER_G1_POINT, BYTES_PER_G2_POINT, G1_POINTS, G2_POINTS, NUM_G1_POINTS, NUM_G2_POINTS,
};
