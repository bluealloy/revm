# KZG 

With the introduction of EIP4844, this use of blobs for a more efficent short term storage is employed, the validity of this blob stored in the consensus layer is verified 
using the `Point Evaluation` pre-compile, a fancy way of verifing that and evaluation at a given point of a commited polynomial is vaild, in a much more bigger scale, implies that `Data is Available`.

This module houses;

1. `KzgSettings`: Stores the setup and parameters needed for computing KZG proofs.

    ```rust
        pub struct KZGSettings {
            #[doc = " The length of `roots_of_unity`, a power of 2."]
            max_width: u64,
            #[doc = " Powers of the primitive root of unity determined by\n `SCALE2_ROOT_OF_UNITY` in bit-reversal permutation order,\n length `max_width`."]
            roots_of_unity: *mut fr_t,
            #[doc = " G1 group elements from the trusted setup,\n in Lagrange form bit-reversal permutation."]
            g1_values: *mut g1_t,
            #[doc = " G2 group elements from the trusted setup."]
            g2_values: *mut g2_t,
        }
    ```

2. `trusted_setup_points`: This module contains functions and types used for parsing and utilizing the `Trused Setup` for this KZG commitment.

    ```rust
        pub use trusted_setup_points::{
            parse_kzg_trusted_setup, G1Points, G2Points, KzgErrors, BYTES_PER_G1_POINT, BYTES_PER_G2_POINT,
            G1_POINTS, G2_POINTS, NUM_G1_POINTS, NUM_G2_POINTS,
        };
    ```