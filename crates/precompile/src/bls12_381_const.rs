//! Constants specifying the precompile addresses for each precompile in EIP-2537

use crate::u64_to_address;
use primitives::Address;

/// G1 add precompile address
pub const G1_ADD_ADDRESS: Address = u64_to_address(0x0b);
/// G1 msm precompile address
pub const G1_MSM_ADDRESS: Address = u64_to_address(0x0c);
/// G2 add precompile address
pub const G2_ADD_ADDRESS: Address = u64_to_address(0x0d);
/// G2 msm precompile address
pub const G2_MSM_ADDRESS: Address = u64_to_address(0x0e);
/// Pairing precompile address
pub const PAIRING_ADDRESS: Address = u64_to_address(0x0f);
/// Map fp to g1 precompile address
pub const MAP_FP_TO_G1_ADDRESS: Address = u64_to_address(0x10);
/// Map fp2 to g2 precompile address
pub const MAP_FP2_TO_G2_ADDRESS: Address = u64_to_address(0x11);

/// G1_ADD_BASE_GAS_FEE specifies the amount of gas needed
/// to perform the G1_ADD precompile.
pub const G1_ADD_BASE_GAS_FEE: u64 = 375;
/// G1_MSM_BASE_GAS_FEE specifies the base amount of gas needed to
/// perform the G1_MSM precompile.
///
/// The cost to do an MSM is determined by the formula:
///    (k * G1_MSM_BASE_GAS_FEE * DISCOUNT\[k\]) // MSM_MULTIPLIER
/// where k is the number of point-scalar pairs.
///
/// Note: If one wants to do a G1 scalar multiplication, they would call
/// this precompile with a single point and a scalar.
pub const G1_MSM_BASE_GAS_FEE: u64 = 12000;
/// MSM_MULTIPLIER specifies the division constant that is used to determine the
/// gas needed to compute an MSM.
///
/// The cost to do an MSM is determined by the formula:
///    (k * MSM_BASE_GAS_FEE * DISCOUNT\[k\]) // MSM_MULTIPLIER
/// where k is the number of point-scalar pairs.
///
/// Note: If `k` is more than the size of the discount table, then
/// the last value in the discount table is chosen.
pub const MSM_MULTIPLIER: u64 = 1000;
/// MAP_FP_TO_G1_BASE_GAS_FEE specifies the amount of gas needed
/// to perform the MAP_FP_TO_G1 precompile.
pub const MAP_FP_TO_G1_BASE_GAS_FEE: u64 = 5500;
/// MAP_FP2_TO_G2_BASE_GAS_FEE specifies the amount of gas needed
/// to perform the MAP_FP2_TO_G2 precompile.
pub const MAP_FP2_TO_G2_BASE_GAS_FEE: u64 = 23800;
/// G2_ADD_BASE_GAS_FEE specifies the amount of gas needed
/// to perform the G2_ADD precompile.
pub const G2_ADD_BASE_GAS_FEE: u64 = 600;
/// G2_MSM_BASE_GAS_FEE specifies the base amount of gas needed to
/// perform the G2_MSM precompile.
///
/// The cost to do an MSM is determined by the formula:
///    (k * G2_MSM_BASE_GAS_FEE * DISCOUNT\[k\]) // MSM_MULTIPLIER
/// where k is the number of point-scalar pairs.
///
/// Note: If one wants to do a G2 scalar multiplication, they would call
/// this precompile with a single point and a scalar.
pub const G2_MSM_BASE_GAS_FEE: u64 = 22500;
/// PAIRING_OFFSET_BASE specifies the y-intercept for the linear expression to determine
/// the amount of gas needed to perform a pairing.
///
/// The cost to do a pairing is determined by the formula:
/// cost = PAIRING_MULTIPLIER_BASE * number_of_pairs + PAIRING_OFFSET_BASE
pub const PAIRING_OFFSET_BASE: u64 = 37700;
/// PAIRING_MULTIPLIER_BASE specifies the slope/gradient for the linear expression to determine
/// the amount of gas needed to perform a pairing.
///
/// The cost to do a pairing is determined by the formula:
///   PAIRING_MULTIPLIER_BASE * number_of_pairs + PAIRING_OFFSET_BASE
pub const PAIRING_MULTIPLIER_BASE: u64 = 32600;

/// Discounts table for G1 MSM as a vector of pairs `[k, discount]`.
pub static DISCOUNT_TABLE_G1_MSM: [u16; 128] = [
    1000, 949, 848, 797, 764, 750, 738, 728, 719, 712, 705, 698, 692, 687, 682, 677, 673, 669, 665,
    661, 658, 654, 651, 648, 645, 642, 640, 637, 635, 632, 630, 627, 625, 623, 621, 619, 617, 615,
    613, 611, 609, 608, 606, 604, 603, 601, 599, 598, 596, 595, 593, 592, 591, 589, 588, 586, 585,
    584, 582, 581, 580, 579, 577, 576, 575, 574, 573, 572, 570, 569, 568, 567, 566, 565, 564, 563,
    562, 561, 560, 559, 558, 557, 556, 555, 554, 553, 552, 551, 550, 549, 548, 547, 547, 546, 545,
    544, 543, 542, 541, 540, 540, 539, 538, 537, 536, 536, 535, 534, 533, 532, 532, 531, 530, 529,
    528, 528, 527, 526, 525, 525, 524, 523, 522, 522, 521, 520, 520, 519,
];
/// Discounts table for G2 MSM as a vector of pairs `[k, discount]`:
pub static DISCOUNT_TABLE_G2_MSM: [u16; 128] = [
    1000, 1000, 923, 884, 855, 832, 812, 796, 782, 770, 759, 749, 740, 732, 724, 717, 711, 704,
    699, 693, 688, 683, 679, 674, 670, 666, 663, 659, 655, 652, 649, 646, 643, 640, 637, 634, 632,
    629, 627, 624, 622, 620, 618, 615, 613, 611, 609, 607, 606, 604, 602, 600, 598, 597, 595, 593,
    592, 590, 589, 587, 586, 584, 583, 582, 580, 579, 578, 576, 575, 574, 573, 571, 570, 569, 568,
    567, 566, 565, 563, 562, 561, 560, 559, 558, 557, 556, 555, 554, 553, 552, 552, 551, 550, 549,
    548, 547, 546, 545, 545, 544, 543, 542, 541, 541, 540, 539, 538, 537, 537, 536, 535, 535, 534,
    533, 532, 532, 531, 530, 530, 529, 528, 528, 527, 526, 526, 525, 524, 524,
];

// Constants related to the bls12-381 precompile inputs and outputs

/// FP_LENGTH specifies the number of bytes needed to represent an
/// Fp element. This is an element in the base field of BLS12-381.
///
/// Note: The base field is used to define G1 and G2 elements.
pub const FP_LENGTH: usize = 48;
/// PADDED_FP_LENGTH specifies the number of bytes that the EVM will use
/// to represent an Fp element according to EIP-2537.
///
/// Note: We only need FP_LENGTH number of bytes to represent it,
/// but we pad the byte representation to be 32 byte aligned as specified in EIP 2537.
pub const PADDED_FP_LENGTH: usize = 64;

/// G1_LENGTH specifies the number of bytes needed to represent a G1 element.
///
/// Note: A G1 element contains 2 Fp elements.
pub const G1_LENGTH: usize = 2 * FP_LENGTH;
/// PADDED_G1_LENGTH specifies the number of bytes that the EVM will use to represent
/// a G1 element according to padding rules specified in EIP-2537.
pub const PADDED_G1_LENGTH: usize = 2 * PADDED_FP_LENGTH;

/// PADDED_FP2_LENGTH specifies the number of bytes that the EVM will use to represent
/// a Fp^2 element according to the padding rules specified in EIP-2537.
///
/// Note: This is the quadratic extension of Fp, and by definition
/// means we need 2 Fp elements.
pub const PADDED_FP2_LENGTH: usize = 2 * PADDED_FP_LENGTH;

/// SCALAR_LENGTH specifies the number of bytes needed to represent an Fr element.
/// This is an element in the scalar field of BLS12-381.
///
/// Note: Since it is already 32 byte aligned, there is no padded version of this constant.
pub const SCALAR_LENGTH: usize = 32;
/// SCALAR_LENGTH_BITS specifies the number of bits needed to represent an Fr element.
/// This is an element in the scalar field of BLS12-381.
pub const SCALAR_LENGTH_BITS: usize = SCALAR_LENGTH * 8;

/// G1_ADD_INPUT_LENGTH specifies the number of bytes that the input to G1ADD
/// must use.
///
/// Note: The input to the G1 addition precompile is 2 G1 elements.
pub const G1_ADD_INPUT_LENGTH: usize = 2 * PADDED_G1_LENGTH;
/// G1_MSM_INPUT_LENGTH specifies the number of bytes that each MSM input pair should have.
///
/// Note: An MSM pair is a G1 element and a scalar. The input to the MSM precompile will have `n`
/// of these pairs.
pub const G1_MSM_INPUT_LENGTH: usize = PADDED_G1_LENGTH + SCALAR_LENGTH;

/// PADDED_G2_LENGTH specifies the number of bytes that the EVM will use to represent
/// a G2 element.
///
/// Note: A G2 element can be represented using 2 Fp^2 elements.
pub const PADDED_G2_LENGTH: usize = 2 * PADDED_FP2_LENGTH;

/// G2_ADD_INPUT_LENGTH specifies the number of bytes that the input to G2ADD
/// must occupy.
///
/// Note: The input to the G2 addition precompile is 2 G2 elements.
pub const G2_ADD_INPUT_LENGTH: usize = 2 * PADDED_G2_LENGTH;
/// G2_MSM_INPUT_LENGTH specifies the number of bytes that each MSM input pair should have.
///
/// Note: An MSM pair is a G2 element and a scalar. The input to the MSM will have `n`
/// of these pairs.
pub const G2_MSM_INPUT_LENGTH: usize = PADDED_G2_LENGTH + SCALAR_LENGTH;

/// PAIRING_INPUT_LENGTH specifies the number of bytes that each Pairing input pair should have.
///
/// Note: An Pairing input-pair is a G2 element and a G1 element. The input to the Pairing will have `n`
/// of these pairs.
pub const PAIRING_INPUT_LENGTH: usize = PADDED_G1_LENGTH + PADDED_G2_LENGTH;

/// FP_PAD_BY specifies the number of bytes that an FP_ELEMENT is padded by to make it 32 byte aligned.
///
/// Note: This should be equal to PADDED_FP_LENGTH - FP_LENGTH.
pub const FP_PAD_BY: usize = 16;

#[test]
fn check_discount_table_invariant_holds() {
    // Currently EIP-2537 specifies the cost for a G1/G2 scalar multiplication in two places
    // in two different ways.
    //
    // First it explicitly says that G1 Multiplication costs 12000 Gas and G2 Multiplication costs 22500.
    //
    // Then it implies the above constants for G1_MSM and G2_MSM via the MSM formula:
    // MSM_COST = k * MSM_BASE_GAS_FEE * DISCOUNT[k-1] // MSM_MULTIPLIER
    //
    // Note that when the MSM has only one point-scalar pair (scalar multiplication), we get:
    // MSM_COST = MSM_BASE_GAS_FEE * DISCOUNT[0] // MSM_MULTIPLIER
    //  (This is because k==1)
    //
    // The 0th entry in the discount table for G1_MSM and G2_MSM is equal to MSM_MULTIPLIER
    // so for k==1, MSM_COST = MSM_BASE_GAS_FEE
    //
    // For G1, MSM_BASE_GAS_FEE matches 12000 and for G2 MSM_BASE_GAS_FEE matches 22500.
    //
    // In this test, we check that this invariant does not change by asserting that the first value
    // in the discount table is equal to the MULTIPLIER.
    assert_eq!(DISCOUNT_TABLE_G1_MSM[0], MSM_MULTIPLIER as u16);
    assert_eq!(DISCOUNT_TABLE_G2_MSM[0], MSM_MULTIPLIER as u16);
    // Note: We could also more robustly check this by defining the G1/G2 Scalar multiplication constants
    // from the EIP and checking that they equal to the value computed by `msm_required_gas` when k==1
}
