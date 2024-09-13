use revm::{
    interpreter::{
        gas,
        opcode::{make_instruction_table, InstructionTable},
        Host, Interpreter,
    },
    primitives::{BlockEnv, EvmWiring, HaltReason, Spec, SpecId, TxEnv},
};
use revm_interpreter::DummyHost;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CustomOpcodeEvmWiring;

impl EvmWiring for CustomOpcodeEvmWiring {
    type Hardfork = CustomOpcodeSpecId;
    type HaltReason = HaltReason;
    type Block = BlockEnv;
    type Transaction = TxEnv;
}

/// Specification IDs for the optimism blockchain.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
pub enum CustomOpcodeSpecId {
    FRONTIER = 0,
    FRONTIER_THAWING = 1,
    HOMESTEAD = 2,
    DAO_FORK = 3,
    TANGERINE = 4,
    SPURIOUS_DRAGON = 5,
    BYZANTIUM = 6,
    CONSTANTINOPLE = 7,
    PETERSBURG = 8,
    ISTANBUL = 9,
    MUIR_GLACIER = 10,
    BERLIN = 11,
    LONDON = 12,
    ARROW_GLACIER = 13,
    GRAY_GLACIER = 14,
    MERGE = 15,
    // Introduces the custom opcode in between the existing hardforks.
    INTRODUCES_OPCODE = 16,
    SHANGHAI = 17,
    CANCUN = 18,
    PRAGUE = 19,
    #[default]
    LATEST = u8::MAX,
}

impl CustomOpcodeSpecId {
    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn enabled(our: Self, other: Self) -> bool {
        our as u8 >= other as u8
    }

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn is_enabled_in(self, other: Self) -> bool {
        Self::enabled(self, other)
    }

    /// Converts the `CustomOpcodeSpecId` into an `SpecId`.
    const fn into_eth_spec_id(self) -> SpecId {
        match self {
            Self::FRONTIER => SpecId::FRONTIER,
            Self::FRONTIER_THAWING => SpecId::FRONTIER_THAWING,
            Self::HOMESTEAD => SpecId::HOMESTEAD,
            Self::DAO_FORK => SpecId::DAO_FORK,
            Self::TANGERINE => SpecId::TANGERINE,
            Self::SPURIOUS_DRAGON => SpecId::SPURIOUS_DRAGON,
            Self::BYZANTIUM => SpecId::BYZANTIUM,
            Self::CONSTANTINOPLE => SpecId::CONSTANTINOPLE,
            Self::PETERSBURG => SpecId::PETERSBURG,
            Self::ISTANBUL => SpecId::ISTANBUL,
            Self::MUIR_GLACIER => SpecId::MUIR_GLACIER,
            Self::BERLIN => SpecId::BERLIN,
            Self::LONDON => SpecId::LONDON,
            Self::ARROW_GLACIER => SpecId::ARROW_GLACIER,
            Self::GRAY_GLACIER => SpecId::GRAY_GLACIER,
            Self::MERGE | Self::INTRODUCES_OPCODE => SpecId::MERGE,
            Self::SHANGHAI => SpecId::SHANGHAI,
            Self::CANCUN => SpecId::CANCUN,
            Self::PRAGUE => SpecId::PRAGUE,
            Self::LATEST => SpecId::LATEST,
        }
    }
}

impl From<CustomOpcodeSpecId> for SpecId {
    fn from(spec_id: CustomOpcodeSpecId) -> Self {
        spec_id.into_eth_spec_id()
    }
}

pub trait CustomOpcodeSpec: Spec + Sized + 'static {
    /// The specification ID for an imaginary chain with custom opcodes.
    const CUSTOM_OPCODE_SPEC_ID: CustomOpcodeSpecId;

    /// Returns whether the provided `CustomOpcodeSpec` is enabled by this spec.
    #[inline]
    fn optimism_enabled(spec_id: CustomOpcodeSpecId) -> bool {
        CustomOpcodeSpecId::enabled(Self::CUSTOM_OPCODE_SPEC_ID, spec_id)
    }
}

macro_rules! spec {
    ($spec_id:ident, $spec_name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $spec_name;

        impl CustomOpcodeSpec for $spec_name {
            const CUSTOM_OPCODE_SPEC_ID: CustomOpcodeSpecId = CustomOpcodeSpecId::$spec_id;
        }

        impl Spec for $spec_name {
            const SPEC_ID: SpecId = $spec_name::CUSTOM_OPCODE_SPEC_ID.into_eth_spec_id();
        }
    };
}

spec!(FRONTIER, FrontierSpec);
// FRONTIER_THAWING no EVM spec change
spec!(HOMESTEAD, HomesteadSpec);
// DAO_FORK no EVM spec change
spec!(TANGERINE, TangerineSpec);
spec!(SPURIOUS_DRAGON, SpuriousDragonSpec);
spec!(BYZANTIUM, ByzantiumSpec);
// CONSTANTINOPLE was overridden with PETERSBURG
spec!(PETERSBURG, PetersburgSpec);
spec!(ISTANBUL, IstanbulSpec);
// MUIR_GLACIER no EVM spec change
spec!(BERLIN, BerlinSpec);
spec!(LONDON, LondonSpec);
// ARROW_GLACIER no EVM spec change
// GRAY_GLACIER no EVM spec change
spec!(MERGE, MergeSpec);
spec!(SHANGHAI, ShanghaiSpec);
spec!(CANCUN, CancunSpec);
spec!(PRAGUE, PragueSpec);

spec!(LATEST, LatestSpec);

// Custom Hardforks
spec!(INTRODUCES_OPCODE, IntroducesOpcodeSpec);

macro_rules! custom_opcode_spec_to_generic {
    ($spec_id:expr, $e:expr) => {{
        // We are transitioning from var to generic spec.
        match $spec_id {
            CustomOpcodeSpecId::FRONTIER | CustomOpcodeSpecId::FRONTIER_THAWING => {
                use FrontierSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::HOMESTEAD | CustomOpcodeSpecId::DAO_FORK => {
                use HomesteadSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::TANGERINE => {
                use TangerineSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::SPURIOUS_DRAGON => {
                use SpuriousDragonSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::BYZANTIUM => {
                use ByzantiumSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::PETERSBURG | CustomOpcodeSpecId::CONSTANTINOPLE => {
                use PetersburgSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::ISTANBUL | CustomOpcodeSpecId::MUIR_GLACIER => {
                use IstanbulSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::BERLIN => {
                use BerlinSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::LONDON
            | CustomOpcodeSpecId::ARROW_GLACIER
            | CustomOpcodeSpecId::GRAY_GLACIER => {
                use LondonSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::MERGE => {
                use MergeSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::SHANGHAI => {
                use ShanghaiSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::CANCUN => {
                use CancunSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::LATEST => {
                use LatestSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::PRAGUE => {
                use PragueSpec as SPEC;
                $e
            }
            CustomOpcodeSpecId::INTRODUCES_OPCODE => {
                use IntroducesOpcodeSpec as SPEC;
                $e
            }
        }
    }};
}

// impl<EXT, DB: Database> EvmHandler<'_, CustomOpcodeEvmWiring, EXT, DB> {
//     pub fn custom_opcode_with_spec(spec_id: CustomOpcodeSpecId) -> Self {
//         let mut handler = Self::mainnet_with_spec(spec_id);

//         custom_opcode_spec_to_generic!(spec_id, {
//             let table = make_custom_instruction_table::<_, SPEC>();
//             handler.set_instruction_table(InstructionTables::Plain(table));
//         });

//         handler
//     }
// }

pub fn make_custom_instruction_table<
    EvmWiringT: EvmWiring,
    H: Host + ?Sized,
    SPEC: CustomOpcodeSpec,
>() -> InstructionTable<H> {
    // custom opcode chain can reuse mainnet instructions
    let mut table = make_instruction_table::<H, SPEC>();

    table[0x0c] = custom_opcode_handler::<H, SPEC>;

    table
}

fn custom_opcode_handler<H: Host + ?Sized, SPEC: CustomOpcodeSpec>(
    interpreter: &mut Interpreter,
    _host: &mut H,
) {
    // opcode has access to the chain-specific spec
    if SPEC::optimism_enabled(CustomOpcodeSpecId::INTRODUCES_OPCODE) {
        gas!(interpreter, gas::MID);
    } else {
        gas!(interpreter, gas::HIGH);
    }

    // logic
}

pub fn main() {
    println!("Example is in code compilation");
    let spec_id = CustomOpcodeSpecId::INTRODUCES_OPCODE;

    let _instructions = custom_opcode_spec_to_generic!(
        spec_id,
        make_custom_instruction_table::<
            CustomOpcodeEvmWiring,
            DummyHost<CustomOpcodeEvmWiring>,
            SPEC,
        >()
    );
}
