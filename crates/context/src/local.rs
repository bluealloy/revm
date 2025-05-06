//! Local context that is filled by execution.
use bytecode::{CodeType, Eof};
use context_interface::LocalContextTr;
use core::cell::RefCell;
use primitives::{keccak256, Bytes, HashMap, B256};
use std::{rc::Rc, vec::Vec};

/// Local context that is filled by execution.
#[derive(Clone, Debug)]
pub struct LocalContext {
    /// Mapping of initcode hash that contains raw bytes ready for validation or status of validation.
    ///
    /// Used in EIP-7873 EOF - TXCREATE to fetch initcode by hash and cache its validation.
    pub initcode_mapping: HashMap<B256, Initcode>,
    /// Interpreter shared memory buffer. A reused memory buffer for calls.
    pub shared_memory_buffer: Rc<RefCell<Vec<u8>>>,
}

impl Default for LocalContext {
    fn default() -> Self {
        Self {
            initcode_mapping: HashMap::default(),
            shared_memory_buffer: Rc::new(RefCell::new(Vec::with_capacity(1024 * 4))),
        }
    }
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
        // Sets len to 0 but it will not shrink to drop the capacity.
        unsafe { self.shared_memory_buffer.borrow_mut().set_len(0) };
    }

    fn get_validated_initcode(&mut self, hash: B256) -> Option<Bytes> {
        let initcode = self.initcode_mapping.get_mut(&hash)?;
        initcode.validate().cloned()
    }

    fn shared_memory_buffer(&self) -> &Rc<RefCell<Vec<u8>>> {
        &self.shared_memory_buffer
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
        match self.status {
            InitcodeStatus::Valid => return Some(&self.bytes),
            InitcodeStatus::Invalid => return None,
            InitcodeStatus::PendingValidation => (),
        }

        // pending validation
        let Ok(eof) = Eof::decode(self.bytes.clone()) else {
            self.status = InitcodeStatus::Invalid;
            return None;
        };

        // validate in Initcode mode, data section should be filled and it should not contain RETURN or STOP
        if eof.validate_mode(CodeType::Initcode).is_err() {
            self.status = InitcodeStatus::Invalid;
            return None;
        }
        // mark initcode as valid so we can skip this validation next time.
        self.status = InitcodeStatus::Valid;
        Some(&self.bytes)
    }
}
