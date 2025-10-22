//! BAL containing writes.

use crate::bal::BalIndex;
use std::vec::Vec;

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

    /// Linear search is used for small number of writes. It is faster than binary search.
    #[inline(never)]
    pub fn get_linear_search(&self, bal_index: BalIndex) -> Option<T> {
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

    /// Get value from BAL.
    pub fn get(&self, bal_index: BalIndex) -> Option<T> {
        if self.writes.len() < 5 {
            return self.get_linear_search(bal_index);
        }
        // else do binary search.
        let index = self
            .writes
            .binary_search_by_key(&bal_index, |(index, _)| *index)
            .ok()?;
        Some(self.writes[index].1.clone())
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
    /// No checks for original value is done. This is useful when we know that value is different.
    #[inline]
    pub fn force_update(&mut self, index: BalIndex, value: T) {
        if let Some(last) = self.writes.last_mut() {
            if index == last.0 {
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
    ///
    /// Assumes that index is always greater than last one and that Writes are updated in proper order.
    #[inline]
    pub fn update_with_key<K: PartialEq, F>(
        &mut self,
        index: BalIndex,
        original_subvalue: &K,
        value: T,
        f: F,
    ) where
        F: Fn(&T) -> &K,
    {
        // if index is different, we push the new value.
        if let Some(last) = self.writes.last_mut() {
            if last.0 != index {
                // we push the new value only if it is changed.
                if f(&last.1) != f(&value) {
                    self.writes.push((index, value));
                }
                return;
            }
        }

        // extract previous (Can be original_subvalue or previous value) and last value.
        let (previous, last) = match self.writes.as_mut_slice() {
            [.., previous, last] => (f(&previous.1), last),
            [last] => (original_subvalue, last),
            [] => {
                // if writes are empty check if original value is same as newly set value.
                if original_subvalue != f(&value) {
                    self.writes.push((index, value));
                }
                return;
            }
        };

        // if previous value is same, we pop the last value.
        if previous == f(&value) {
            self.writes.pop();
            return;
        }

        // if it is different, we update the last value.
        last.1 = value;
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
