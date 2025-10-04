//! BAL containing writes.

use crate::bal::BalIndex;

/// Use to store values
///
/// If empty it means that this item was read from database.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalWrites<T: PartialEq> {
    /// List of writes with BalIndex.
    pub writes: Vec<(BalIndex, T)>,
}

impl<T: PartialEq> BalWrites<T> {
    /// Extend the builder with another builder.
    pub fn extend(&mut self, other: BalWrites<T>) {
        self.writes.extend(other.writes);
    }

    /// Returns true if the builder is empty.
    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
    }

    /// Force insert a value into the BalWrites.
    ///
    /// Check if last index is same as the index to insert.
    /// If it is, we override the value.
    /// If it is not, we push the value to the end of the vector.
    ///
    /// No checks for original value is done. This is usefull when we know that value is different.
    #[inline]
    pub fn force_insert(&mut self, index: BalIndex, value: T) {
        if let Some(last) = self.writes.last_mut() {
            if last.0 == index {
                last.1 = value;
                return;
            }
        }
        self.writes.push((index, value));
    }

    /// Insert a value into the builder.
    ///
    /// If BalIndex is same as last it will override the value.
    pub fn insert(&mut self, index: BalIndex, original_value: &T, value: T) {
        if let Some(last) = self.writes.last_mut() {
            if last.0 == index {
                // if original value is same as newly written value we pop the last value.
                if original_value == &value {
                    self.writes.pop();
                } else {
                    last.1 = value;
                }
                return;
            }
        } else {
            // if there is no last, we skip insertion if original valus is same as written value.
            if original_value == &value {
                return;
            }
        }
        self.writes.push((index, value));
    }
}
