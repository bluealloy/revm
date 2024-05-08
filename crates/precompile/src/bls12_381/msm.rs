/// Amount used to calculate the multi-scalar-multiplication discount.
const MSM_MULTIPLIER: u64 = 1000;
/// Table of gas discounts for multi-scalar-multiplication operations.
const MSM_DISCOUNT_TABLE: [u64; 128] = [
    1200, 888, 764, 641, 594, 547, 500, 453, 438, 423, 408, 394, 379, 364, 349, 334, 330, 326, 322,
    318, 314, 310, 306, 302, 298, 294, 289, 285, 281, 277, 273, 269, 268, 266, 265, 263, 262, 260,
    259, 257, 256, 254, 253, 251, 250, 248, 247, 245, 244, 242, 241, 239, 238, 236, 235, 233, 232,
    231, 229, 228, 226, 225, 223, 222, 221, 220, 219, 219, 218, 217, 216, 216, 215, 214, 213, 213,
    212, 211, 211, 210, 209, 208, 208, 207, 206, 205, 205, 204, 203, 202, 202, 201, 200, 199, 199,
    198, 197, 196, 196, 195, 194, 193, 193, 192, 191, 191, 190, 189, 188, 188, 187, 186, 185, 185,
    184, 183, 182, 182, 181, 180, 179, 179, 178, 177, 176, 176, 175, 174,
];

/// Implements the gas schedule for G1/G2 Multiscalar-multiplication assuming 30
/// MGas/second, see also: <https://eips.ethereum.org/EIPS/eip-2537#g1g2-multiexponentiation>
pub(super) fn msm_required_gas(k: usize, multiplication_cost: u64) -> u64 {
    if k == 0 {
        return 0;
    }

    let discount = if k < MSM_DISCOUNT_TABLE.len() {
        MSM_DISCOUNT_TABLE[k - 1]
    } else {
        MSM_DISCOUNT_TABLE[MSM_DISCOUNT_TABLE.len() - 1]
    };

    (k as u64 * discount * multiplication_cost) / MSM_MULTIPLIER
}
