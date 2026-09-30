//! BAL containing writes.

use crate::bal::BlockAccessIndex;
use std::vec::Vec;

/// Use to store values
///
/// If empty it means that this item was read from database.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BalWrites<T: PartialEq + Clone> {
    /// List of writes with [`BlockAccessIndex`].
    pub writes: Vec<(BlockAccessIndex, T)>,
}

impl<T: PartialEq + Clone> BalWrites<T> {
    /// Create a new BalWrites.
    pub fn new(mut writes: Vec<(BlockAccessIndex, T)>) -> Self {
        writes.sort_by_key(|(index, _)| *index);
        Self { writes }
    }

    /// Linear search is used for small number of writes. It is faster than binary search.
    #[inline(never)]
    pub fn get_linear_search(&self, bal_index: BlockAccessIndex) -> Option<T> {
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
    pub fn get(&self, bal_index: BlockAccessIndex) -> Option<T> {
        if self.writes.len() < 5 {
            return self.get_linear_search(bal_index);
        }
        // else do binary search.
        let i = match self
            .writes
            .binary_search_by_key(&bal_index, |(index, _)| *index)
        {
            Ok(i) => i,
            Err(i) => i,
        };
        // only if i is not zero, we return the previous value.
        (i != 0).then(|| self.writes[i - 1].1.clone())
    }

    /// Extend the builder with another builder.
    pub fn extend(&mut self, other: BalWrites<T>) {
        self.writes.extend(other.writes);
    }

    /// Returns true if the builder is empty.
    pub const fn is_empty(&self) -> bool {
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
    pub fn force_update(&mut self, index: BlockAccessIndex, value: T) {
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
    /// If [`BlockAccessIndex`] is same as last it will override the value.
    pub fn update(&mut self, index: BlockAccessIndex, original_value: &T, value: T) {
        self.update_with_key(index, original_value, value, |i| i);
    }

    /// Insert a value into the builder.
    ///
    /// If [`BlockAccessIndex`] is same as last it will override the value.
    ///
    /// Assumes that index is always greater than last one and that Writes are updated in proper order.
    #[inline]
    pub fn update_with_key<K: PartialEq, F>(
        &mut self,
        index: BlockAccessIndex,
        original_subvalue: &K,
        value: T,
        f: F,
    ) where
        F: Fn(&T) -> &K,
    {
        // Unchanged value: nothing to record, and a write made earlier at this index must be kept.
        if original_subvalue == f(&value) {
            return;
        }

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
                self.writes.push((index, value));
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

    const fn idx(index: u64) -> BlockAccessIndex {
        BlockAccessIndex::new(index)
    }

    #[test]
    fn test_get() {
        let bal_writes = BalWrites::new(vec![(idx(0), 1), (idx(1), 2), (idx(2), 3)]);
        assert_eq!(bal_writes.get(idx(0)), None);
        assert_eq!(bal_writes.get(idx(1)), Some(1));
        assert_eq!(bal_writes.get(idx(2)), Some(2));
        assert_eq!(bal_writes.get(idx(3)), Some(3));
        assert_eq!(bal_writes.get(idx(4)), Some(3));
    }

    fn get_binary_search(threshold: u64) {
        // Construct test data up to (threshold - 1), skipping one key to simulate a gap.
        let entries: Vec<_> = (0..threshold - 1)
            .map(|i| (idx(i), i + 1))
            .chain(std::iter::once((idx(threshold), threshold + 1)))
            .collect();

        let bal_writes = BalWrites::new(entries);

        // Case 1: lookup before any entries
        assert_eq!(bal_writes.get(idx(0)), None);

        // Case 2: lookups for existing keys before the gap
        for i in 1..threshold - 1 {
            assert_eq!(bal_writes.get(idx(i)), Some(i));
        }

        // Case 3: lookup at the skipped key — should return the previous value
        assert_eq!(bal_writes.get(idx(threshold)), Some(threshold - 1));

        // Case 4: lookup after the skipped key — should return the next valid value
        assert_eq!(bal_writes.get(idx(threshold + 1)), Some(threshold + 1));
    }

    #[test]
    fn test_get_binary_search() {
        get_binary_search(4);
        get_binary_search(5);
        get_binary_search(6);
        get_binary_search(7);
    }

    /// Applies `(index, value)` incorporations in order, each starting from the current value.
    fn apply(start: u64, steps: &[(u64, u64)]) -> BalWrites<u64> {
        let mut writes = BalWrites::default();
        let mut current = start;
        for &(index, value) in steps {
            writes.update(idx(index), &current, value);
            current = value;
        }
        writes
    }

    /// A later incorporation at the same index that only reads the item (e.g. a system call
    /// reading an account a withdrawal credited at index n+1) must keep the earlier write.
    #[test]
    fn test_update_read_after_write_at_same_index_keeps_write() {
        assert_eq!(apply(0, &[(1, 1), (1, 1)]).writes, vec![(idx(1), 1)]);
    }

    /// Every sequence of up to five incorporations over indices 1..=3 and values 0..=2:
    /// unchanged incorporations are no-ops, and `get(i + 1)` returns the value at the end of
    /// index `i`.
    #[test]
    fn test_update_all_short_sequences() {
        fn check(start: u64, seq: &mut Vec<(u64, u64)>) {
            let writes = apply(start, seq);

            let mut current = start;
            let changed: Vec<_> = seq
                .iter()
                .copied()
                .filter(|&(_, value)| core::mem::replace(&mut current, value) != value)
                .collect();
            assert_eq!(
                writes,
                apply(start, &changed),
                "start {start}, steps {seq:?}"
            );

            for (i, &(index, value)) in seq.iter().enumerate() {
                if seq.get(i + 1).is_none_or(|&(next, _)| next != index) {
                    let got = writes.get(idx(index + 1)).unwrap_or(start);
                    assert_eq!(got, value, "start {start}, steps {seq:?}, index {index}");
                }
            }

            if seq.len() < 5 {
                let min_index = seq.last().map_or(1, |&(index, _)| index);
                for index in min_index..=3 {
                    for value in 0..=2 {
                        seq.push((index, value));
                        check(start, seq);
                        seq.pop();
                    }
                }
            }
        }

        for start in 0..=1 {
            check(start, &mut Vec::new());
        }
    }
}
