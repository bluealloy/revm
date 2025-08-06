//! Storage slot data structures and implementations.

use primitives::StorageValue;

/// This type keeps track of the current value of a storage slot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvmStorageSlot {
    /// Original value of the storage slot
    pub original_value: StorageValue,
    /// Present value of the storage slot
    pub present_value: StorageValue,
    /// Transaction id, used to track when storage slot was made warm.
    pub transaction_id: usize,
    /// Represents if the storage slot is cold
    pub is_cold: bool,
}

impl EvmStorageSlot {
    /// Creates a new _unchanged_ `EvmStorageSlot` for the given value.
    pub fn new(original: StorageValue, transaction_id: usize) -> Self {
        Self {
            original_value: original,
            present_value: original,
            transaction_id,
            is_cold: false,
        }
    }

    /// Creates a new _changed_ `EvmStorageSlot`.
    pub fn new_changed(
        original_value: StorageValue,
        present_value: StorageValue,
        transaction_id: usize,
    ) -> Self {
        Self {
            original_value,
            present_value,
            transaction_id,
            is_cold: false,
        }
    }
    /// Returns true if the present value differs from the original value.
    pub fn is_changed(&self) -> bool {
        self.original_value != self.present_value
    }

    /// Returns the original value of the storage slot.
    #[inline]
    pub fn original_value(&self) -> StorageValue {
        self.original_value
    }

    /// Returns the current value of the storage slot.
    #[inline]
    pub fn present_value(&self) -> StorageValue {
        self.present_value
    }

    /// Marks the storage slot as cold. Does not change transaction_id.
    #[inline]
    pub fn mark_cold(&mut self) {
        self.is_cold = true;
    }

    /// Marks the storage slot as warm and sets transaction_id to the given value
    ///
    ///
    /// Returns false if old transition_id is different from given id or in case they are same return `Self::is_cold` value.
    #[inline]
    pub fn mark_warm_with_transaction_id(&mut self, transaction_id: usize) -> bool {
        let same_id = self.transaction_id == transaction_id;
        self.transaction_id = transaction_id;
        let was_cold = core::mem::replace(&mut self.is_cold, false);

        if same_id {
            // only if transaction id is same we are returning was_cold.
            return was_cold;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::U256;

    #[test]
    fn test_storage_mark_warm_with_transaction_id() {
        let mut slot = EvmStorageSlot::new(U256::ZERO, 0);
        slot.is_cold = true;
        slot.transaction_id = 0;
        assert!(slot.mark_warm_with_transaction_id(1));

        slot.is_cold = false;
        slot.transaction_id = 0;
        assert!(slot.mark_warm_with_transaction_id(1));

        slot.is_cold = true;
        slot.transaction_id = 1;
        assert!(slot.mark_warm_with_transaction_id(1));

        slot.is_cold = false;
        slot.transaction_id = 1;
        // Only if transaction id is same and is_cold is false, return false.
        assert!(!slot.mark_warm_with_transaction_id(1));
    }
}
