//! BAL containing writes.

use crate::bal::BalIndex;

/// Use to store values
///
/// If empty it means that this item was read from database.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalWrites<T: PartialEq + Clone> {
    /// List of writes with BalIndex.
    pub writes: Vec<(BalIndex, T)>,
}

impl<T: PartialEq + Clone> BalWrites<T> {
    /// Create a new BalWrites.
    pub fn new(mut writes: Vec<(BalIndex, T)>) -> Self {
        writes.sort_by_key(|(index, _)| *index);
        Self { writes }
    }

    /// Get value from BAL.
    pub fn get(&self, bal_index: BalIndex) -> Option<T> {
        let mut last_item = None;
        for (index, item) in self.writes.iter() {
            // if index is greater than bal_index we return the last item.
            if index >= &bal_index {
                return last_item;
            }
            last_item = Some(item.clone());
        }
        last_item
    }

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
    pub fn force_update(&mut self, index: BalIndex, value: T) {
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
    pub fn update(&mut self, index: BalIndex, original_value: &T, value: T) {
        self.update_with_key(index, original_value, value, |i| i);
    }

    /// Insert a value into the builder.
    ///
    /// If BalIndex is same as last it will override the value.
    #[inline]
    pub fn update_with_key<K: PartialEq, F>(
        &mut self,
        index: BalIndex,
        original_key: &K,
        value: T,
        f: F,
    ) where
        F: FnOnce(&T) -> &K,
    {
        if let Some(last) = self.writes.last_mut() {
            if last.0 == index {
                // if original value is same as newly written value we pop the last value.
                if original_key == f(&value) {
                    self.writes.pop();
                } else {
                    last.1 = value;
                }
                return;
            }
        } else {
            // if there is no last, we skip insertion if original valus is same as written value.
            if original_key == f(&value) {
                return;
            }
        }
        self.writes.push((index, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get() {
        let bal_writes = BalWrites::new(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(bal_writes.get(0), None);
        assert_eq!(bal_writes.get(1), Some(1));
        assert_eq!(bal_writes.get(2), Some(2));
        assert_eq!(bal_writes.get(3), Some(3));
        assert_eq!(bal_writes.get(4), Some(3));
    }
}
