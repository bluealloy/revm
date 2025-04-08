use bytecode::{CodeType, Eof};
use context_interface::LocalContextTr;
use primitives::{keccak256, Bytes, HashMap, B256};

/// Local context that is filled by execution.
#[derive(Clone, Debug, Default)]
pub struct LocalContext {
    /// Mapping of initcode hash that contains raw bytes ready for validation or status of validation.
    ///
    /// Used in EIP-7873 EOF - TXCREATE to fetch initcode by hash and cache its validation.
    pub initcode_mapping: HashMap<B256, Initcode>,
}

impl LocalContextTr for LocalContext {
    fn insert_initcodes(&mut self, initcodes: &[Bytes]) {
        self.initcode_mapping = initcodes
            .iter()
            .map(|b| (keccak256(b), Initcode::new(b.clone())))
            .collect();
    }

    fn clear(&mut self) {
        self.initcode_mapping.clear();
    }

    fn get_validated_initcode(&mut self, hash: B256) -> Option<Bytes> {
        let initcode = self.initcode_mapping.get_mut(&hash)?;
        initcode.validate().cloned()
    }
}

impl LocalContext {
    /// Creates a new local context, initcodes are hashes and added to the mapping.
    pub fn new(initcode: &[Bytes]) -> Self {
        let mut s = Self::default();
        s.insert_initcodes(initcode);
        s
    }
}

/// Status of the initcode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitcodeStatus {
    /// Initcode is valid, it was decoded into EOF and validated.
    Valid,
    /// Initcode is invalid this can mean decoding failed or validation failed.
    Invalid,
    /// Initcode is pending validation.
    PendingValidation,
}

/// Initcode with validation status.
#[derive(Clone, Debug)]
pub struct Initcode {
    /// Raw bytes of the initcode.
    bytes: Bytes,
    /// Status of the initcode.
    status: InitcodeStatus,
}

impl Initcode {
    /// Creates a new initcode with validation set to false.
    pub fn new(initcode: Bytes) -> Self {
        Self {
            bytes: initcode,
            status: InitcodeStatus::PendingValidation,
        }
    }

    /// Validates the initcode and sets the status to valid if it is valid.
    ///
    /// If initcode is not pending validation it will return None.
    pub fn validate(&mut self) -> Option<&Bytes> {
        if self.status != InitcodeStatus::PendingValidation {
            return Some(&self.bytes);
        };

        let Ok(eof) = Eof::decode(self.bytes.clone()) else {
            self.status = InitcodeStatus::Invalid;
            return None;
        };

        if eof.validate_mode(CodeType::Initcode).is_err() {
            self.status = InitcodeStatus::Invalid;
            return None;
        }

        self.status = InitcodeStatus::Valid;
        Some(&self.bytes)
    }
}
