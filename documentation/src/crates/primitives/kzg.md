# KZG 

With the introduction of [EIP4844](https://eips.ethereum.org/EIPS/eip-4844), this use of blobs for a more efficent short term storage is employed, the validity of this blob stored in the consensus layer is verified using the `Point Evaluation` pre-compile, a fancy way of verifing that and evaluation at a given point of a commited polynomial is vaild, in a much more bigger scale, implies that `Data is Available`.

This module houses;

1. `KzgSettings`: Stores the setup and parameters needed for computing and verify KZG proofs.

    The `KZG` premitive provides a default `KZGSettings` obtained from [this]( https://ceremony.ethereum.org/) trusted setup ceremony, a provision is also made for using a custom `KZGSettings` if need be, this is available in the `env.cfg`.


2. `trusted_setup_points`: This module contains functions and types used for parsing and utilizing the [Trusted Setup]( https://ceremony.ethereum.org/) for the `KzgSettings`.