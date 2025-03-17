// Constants related to the bls12-381 gas schedule and precompile address
pub const G1_ADD_ADDRESS: u64 = 0x0b;
pub const G1_ADD_BASE_GAS_FEE: u64 = 375;
pub const G1_MSM_ADDRESS: u64 = 0x0c;
pub const G1_MSM_BASE_GAS_FEE: u64 = 12000;
pub const MAP_FP_TO_G1_ADDRESS: u64 = 0x10;
pub const MAP_FP_TO_G1_BASE_GAS_FEE: u64 = 5500;
pub const MAP_FP2_TO_G2_ADDRESS: u64 = 0x11;
pub const MAP_FP2_TO_G2_BASE_GAS_FEE: u64 = 23800;
pub const G2_ADD_ADDRESS: u64 = 0x0d;
pub const G2_ADD_BASE_GAS_FEE: u64 = 600;
pub const G2_MSM_ADDRESS: u64 = 0x0e;
pub const G2_MSM_BASE_GAS_FEE: u64 = 22500;
pub const PAIRING_ADDRESS: u64 = 0x0f;
// TODO: Why does this have PAIRING twice?
pub const PAIRING_PAIRING_OFFSET_BASE: u64 = 37700;
pub const MSM_MULTIPLIER: u64 = 1000;
// TODO: Why does this have PAIRING twice?
pub const PAIRING_PAIRING_MULTIPLIER_BASE: u64 = 32600;

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
// Discounts table for G2 MSM as a vector of pairs `[k, discount]`:
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
/// TODO: Maybe change all _LENGTH to _LENGTH_BYTES and then NBITS to _LENGTH_BITS
pub const SCALAR_LENGTH: usize = 32;
/// NBITS specifies the number of bits needed to represent an Fr element.
/// This is an element in the scalar field of BLS12-381.
pub const NBITS: usize = 256;

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

/// PADDING_LENGTH specifies the number of bytes that an FP_ELEMENT is padded by.
///
/// Note: This should be equal to PADDED_FP_LENGTH - FP_LENGTH.
/// TODO: Should likely rename this to FP_PADDING_LENGTH, since other objects are also padded
pub const PADDING_LENGTH: usize = 16;

// Big-endian non-Montgomery form.
pub const MODULUS_REPR: [u8; 48] = [
    0x1a, 0x01, 0x11, 0xea, 0x39, 0x7f, 0xe6, 0x9a, 0x4b, 0x1b, 0xa7, 0xb6, 0x43, 0x4b, 0xac, 0xd7,
    0x64, 0x77, 0x4b, 0x84, 0xf3, 0x85, 0x12, 0xbf, 0x67, 0x30, 0xd2, 0xa0, 0xf6, 0xb0, 0xf6, 0x24,
    0x1e, 0xab, 0xff, 0xfe, 0xb1, 0x53, 0xff, 0xff, 0xb9, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xaa, 0xab,
];
