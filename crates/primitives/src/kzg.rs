mod env_settings;
mod trusted_setup_points;

cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        pub use c_kzg::KzgSettings;
    } else if #[cfg(feature = "kzg-rs")] {
        pub use kzg_rs::KzgSettings;
    }
}

pub use env_settings::EnvKzgSettings;
pub use trusted_setup_points::{
    parse_kzg_trusted_setup, G1Points, G2Points, KzgErrors, BYTES_PER_G1_POINT, BYTES_PER_G2_POINT,
    G1_POINTS, G2_POINTS, NUM_G1_POINTS, NUM_G2_POINTS,
};
