//! A `VecMap` implementation, which is a map based on a vector.
//! This is useful for small maps where the performance of iterating over a vector
//! and can be better than using a hash map, due to better cache locality and branch prediction.

use std::{
    fmt::Debug,
    iter::FromIterator,
    ops::{Index, IndexMut},
};

/// A std::vec::Vec based Map, motivated by the fact that, for some key types,
/// iterating over a vector can be faster than other methods for small maps.
///
/// Most of the operations on this map implementation work in O(n), including
/// some of the ones that are O(1) in HashMap. However, optimizers can work magic with
/// contiguous arrays like Vec, and so for small sets,
/// iterating through a vector actually yields better performance than the
/// less branch- and cache-predictable hash maps.
#[derive(Clone, Eq)]
pub struct VecMap<K, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K, V> VecMap<K, V> {
    /// Creates a new empty `VecMap`.
    pub fn new() -> Self
    where
        K: PartialEq,
    {
        Self::with_capacity(0)
    }

    /// Creates a new `VecMap` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self
    where
        K: PartialEq,
    {
        VecMap {
            keys: Vec::with_capacity(capacity),
            values: Vec::with_capacity(capacity),
        }
    }

    /// Returns the number of key-value pairs in the map.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Returns true if the map contains no key-value pairs.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of key-value pairs in the map.
    pub fn capacity(&self) -> usize {
        self.keys.capacity().min(self.values.capacity())
    }

    /// Clears the map, removing all key-value pairs.
    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }

    #[inline]
    fn position<Q: PartialEq<K> + ?Sized>(&self, key: &Q) -> Option<usize> {
        self.keys.iter().position(|k| key == k)
    }

    /// Checks if the map contains the specified key.
    pub fn contains_key<Q: PartialEq<K> + ?Sized>(&self, key: &Q) -> bool {
        self.position(key).is_some()
    }

    /// Returns a reference to the value associated with the specified key.
    pub fn get<'l, Q: PartialEq<K> + ?Sized>(&'l self, key: &Q) -> Option<&'l V> {
        self.position(key).map(|p| &self.values[p])
    }

    /// Returns a mutable reference to the value associated with the specified key.
    pub fn get_mut<'l, Q: PartialEq<K> + ?Sized>(&'l mut self, key: &Q) -> Option<&'l mut V> {
        self.position(key).map(move |p| &mut self.values[p])
    }

    /// Inserts a key-value pair into the map, replacing the value if the key already exists.
    pub fn insert(&mut self, key: K, mut value: V) -> Option<V>
    where
        K: PartialEq,
    {
        if let Some(position) = self.position(&key) {
            std::mem::swap(&mut value, &mut self.values[position]);
            Some(value)
        } else {
            self.keys.push(key);
            self.values.push(value);
            None
        }
    }

    /// Removes all elements from the map and returns an iterator over the removed key-value pairs.
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        Drain {
            iter: self.keys.drain(..).zip(self.values.drain(..)),
        }
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    pub fn reserve(&mut self, additional: usize) {
        self.keys.reserve(additional);
        self.values.reserve(additional);
    }

    /// Shrinks the capacity of the map to fit its current length.
    pub fn shrink_to_fit(&mut self) {
        self.keys.shrink_to_fit();
        self.values.shrink_to_fit();
    }

    /// Returns a reference to the key-value pair associated with the specified key.
    pub fn get_key_value<'l, Q: PartialEq<K> + ?Sized>(
        &'l self,
        key: &Q,
    ) -> Option<(&'l K, &'l V)> {
        self.position(key).map(|p| (&self.keys[p], &self.values[p]))
    }

    /// Removes the value associated with the specified key and returns it.
    pub fn remove<Q: PartialEq<K> + ?Sized>(&mut self, key: &Q) -> Option<V> {
        if let Some(index) = self.position(key) {
            self.keys.swap_remove(index);
            Some(self.values.swap_remove(index))
        } else {
            None
        }
    }

    /// Returns an entry for the specified key, which can be either occupied or vacant.
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V>
    where
        K: PartialEq,
    {
        match self
            .keys()
            .enumerate()
            .find(|(_, k)| &&key == k)
            .map(|(n, _)| n)
        {
            Some(index) => Entry::Occupied(OccupiedEntry { map: self, index }),
            None => Entry::Vacant(VacantEntry { map: self, key }),
        }
    }

    /// Removes the entry with the specified key and returns it as a tuple of (key, value).
    pub fn remove_entry<Q: PartialEq<K> + ?Sized>(&mut self, key: &Q) -> Option<(K, V)> {
        if let Some(index) = self.position(key) {
            Some((self.keys.swap_remove(index), self.values.swap_remove(index)))
        } else {
            None
        }
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain<F: FnMut(&K, &mut V) -> bool>(&mut self, mut f: F) {
        for i in (0..self.len()).rev() {
            if !f(&self.keys[i], &mut self.values[i]) {
                self.keys.swap_remove(i);
                self.values.swap_remove(i);
            }
        }
    }

    /// Returns an iterator yielding references to the keys and references to the values.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.keys.iter().zip(self.values.iter()),
        }
    }

    /// Returns an iterator yielding mutable references to the keys and mutable references to the values.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.keys.iter().zip(self.values.iter_mut()),
        }
    }

    /// Sorts the map by keys in ascending order.
    pub fn sort(&mut self)
    where
        K: Ord,
    {
        let mut indices: Vec<usize> = (0..self.len()).collect();
        indices.sort_unstable_by_key(|i| &self.keys[*i]);
        reorder_vec(&mut self.keys, indices.iter().copied());
        reorder_vec(&mut self.values, indices.iter().copied());
    }

    /// Much faster than `self == other`, but will return false if the order of the data isn't identical.
    /// # Safety
    /// Note that for the order of data with two `VecMap`s to be identical, they must either have been both sorted,
    /// or they must have undergone the insertion and removal of keys in the same order.
    pub unsafe fn identical(&self, other: &Self) -> bool
    where
        K: PartialEq,
        V: PartialEq,
    {
        self.keys == other.keys && self.values == other.values
    }

    /// Returns an iterator over the keys in arbitrary order.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.keys.iter(),
            _phantom: Default::default(),
        }
    }

    /// Returns an iterator over the values in arbitrary order.
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.values.iter(),
            _phantom: Default::default(),
        }
    }
}

impl<K: PartialEq, V> Default for VecMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Debug, V: Debug> Debug for VecMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

fn reorder_vec<T>(vec: &mut [T], order: impl Iterator<Item = usize>) {
    use std::mem::MaybeUninit;
    let mut buffer: Vec<MaybeUninit<T>> = vec.iter().map(|_| MaybeUninit::uninit()).collect();
    for (from, to) in order.enumerate() {
        std::mem::swap(&mut vec[to], unsafe { &mut *(buffer[from].as_mut_ptr()) });
    }
    for i in 0..vec.len() {
        std::mem::swap(&mut vec[i], unsafe { &mut *(buffer[i].as_mut_ptr()) });
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for VecMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        if self.len() != other.len() {
            return false;
        }
        for (key, value) in self.iter() {
            match other.get(key) {
                Some(v) if value == v => {}
                _ => return false,
            }
        }
        true
    }
}

impl<'a, K: PartialEq + Copy + 'a, V: Copy + 'a> Extend<(&'a K, &'a V)> for VecMap<K, V> {
    fn extend<T: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iter: T) {
        for (key, value) in iter.into_iter() {
            self.insert(*key, *value);
        }
    }
}

impl<K: PartialEq, V> Extend<(K, V)> for VecMap<K, V> {
    fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
        for (key, value) in iter.into_iter() {
            self.insert(key, value);
        }
    }
}

impl<K: PartialEq, V> FromIterator<(K, V)> for VecMap<K, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iterator = iter.into_iter();
        let lower = iterator.size_hint().0;
        let mut this = Self::with_capacity(lower);
        this.extend(iterator);
        this
    }
}

impl<'a, Q: PartialEq<K> + ?Sized, K, V> Index<&'a Q> for VecMap<K, V> {
    type Output = V;
    fn index(&self, key: &'a Q) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<'a, Q: PartialEq<K> + ?Sized, K, V> IndexMut<&'a Q> for VecMap<K, V> {
    fn index_mut(&mut self, key: &'a Q) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

impl<'a, K, V> IntoIterator for &'a VecMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut VecMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> IntoIterator for VecMap<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.keys.into_iter().zip(self.values.into_iter()),
        }
    }
}

/// An iterator yielding owned keys and values from a `VecMap`.
#[derive(Debug, Clone)]
pub struct IntoIter<K, V> {
    iter: std::iter::Zip<std::vec::IntoIter<K>, std::vec::IntoIter<V>>,
}

impl<K, V> Iterator for IntoIter<K, V> {
    type Item = (K, V);

    fn next(&mut self) -> Option<(K, V)> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
    fn next_back(&mut self) -> Option<(K, V)> {
        self.iter.next_back()
    }
}

impl<K, V> ExactSizeIterator for IntoIter<K, V> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

/// A view into a single occupied location in a `VecMap`.
///
/// See [`VecMap::entry`](struct.VecMap.html#method.entry) for details.
#[derive(Debug)]
pub struct OccupiedEntry<'a, K, V> {
    map: &'a mut VecMap<K, V>,
    index: usize,
}

/// A view into a single vacant location in a `VecMap`.
///
/// See [`VecMap::entry`](struct.VecMap.html#method.entry) for details.
#[derive(Debug)]
pub struct VacantEntry<'a, K, V> {
    map: &'a mut VecMap<K, V>,
    key: K,
}

/// A view into a single entry in a `VecMap`.
///
/// See [`VecMap::entry`](struct.VecMap.html#method.entry) for details.
#[derive(Debug)]
pub enum Entry<'a, K, V> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, K, V>),

    /// A vacant entry.
    Vacant(VacantEntry<'a, K, V>),
}

use Entry::*;
impl<'a, K, V> Entry<'a, K, V> {
    /// Ensures that the entry is occupied by inserting the given value if it is vacant.
    ///
    /// Returns a mutable reference to the entry's value.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures that the entry is occupied by inserting the the result of the given function if it
    /// is vacant.
    ///
    /// Returns a mutable reference to the entry's value.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Occupied(entry) => entry.into_mut(),
            Vacant(entry) => entry.insert(default()),
        }
    }
}

impl<'a, K, V> OccupiedEntry<'a, K, V> {
    /// Returns a reference to the entry's value.
    pub fn get(&self) -> &V {
        &self.map.values[self.index]
    }

    /// Returns a mutable reference to the entry's value.
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.map.values[self.index]
    }

    /// Returns a mutable reference to the entry's value with the same lifetime as the map.
    pub fn into_mut(self) -> &'a mut V {
        &mut self.map.values[self.index]
    }

    /// Replaces the entry's value with the given one and returns the previous value.
    pub fn insert(&mut self, value: V) -> V {
        std::mem::replace(self.get_mut(), value)
    }

    /// Removes the entry from the map and returns its value.
    pub fn remove(self) -> V {
        self.map.keys.swap_remove(self.index);
        self.map.values.swap_remove(self.index)
    }
}

impl<'a, K, V> VacantEntry<'a, K, V> {
    /// Inserts the entry into the map with the given value.
    ///
    /// Returns a mutable reference to the entry's value with the same lifetime as the map.
    pub fn insert(self, value: V) -> &'a mut V {
        self.map.keys.push(self.key);
        self.map.values.push(value);
        self.map.values.last_mut().unwrap()
    }
}

/// A draining iterator over a `VecMap`.
///
/// See [`VecMap::drain`](struct.VecMap.html#method.drain) for details.
#[derive(Debug)]
pub struct Drain<'a, K, V> {
    iter: std::iter::Zip<std::vec::Drain<'a, K>, std::vec::Drain<'a, V>>,
}

/// An iterator yielding references to a `VecMap`'s keys and their corresponding values.
///
/// See [`VecMap::iter`](struct.VecMap.html#method.iter) for details.
#[derive(Debug, Clone)]
pub struct Iter<'a, K, V> {
    iter: std::iter::Zip<std::slice::Iter<'a, K>, std::slice::Iter<'a, V>>,
}

/// An iterator yielding references to a `VecMap`'s keys and mutable references to their
/// corresponding values.
///
/// See [`VecMap::iter_mut`](struct.VecMap.html#method.iter_mut) for details.
#[derive(Debug)]
pub struct IterMut<'a, K, V> {
    iter: std::iter::Zip<std::slice::Iter<'a, K>, std::slice::IterMut<'a, V>>,
}

/// An iterator yielding references to a `VecMap`'s keys in arbitrary order.
///
/// See [`VecMap::keys`](struct.VecMap.html#method.keys) for details.
#[derive(Debug)]
pub struct Keys<'a, K, V> {
    iter: std::slice::Iter<'a, K>,
    _phantom: std::marker::PhantomData<V>,
}

impl<'a, K, V> Clone for Keys<'a, K, V> {
    fn clone(&self) -> Self {
        Keys {
            iter: self.iter.clone(),
            _phantom: Default::default(),
        }
    }
}

/// An iterator yielding references to a `VecMap`'s values in arbitrary order.
///
/// See [`VecMap::values`](struct.VecMap.html#method.values) for details.
#[derive(Debug)]
pub struct Values<'a, K, V> {
    iter: std::slice::Iter<'a, V>,
    _phantom: std::marker::PhantomData<K>,
}

impl<'a, K, V> Clone for Values<'a, K, V> {
    fn clone(&self) -> Self {
        Values {
            iter: self.iter.clone(),
            _phantom: Default::default(),
        }
    }
}

macro_rules! impl_iter {
    ($typ:ty, $item:ty) => {
        impl<'a, K, V> Iterator for $typ {
            type Item = $item;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl<'a, K, V> DoubleEndedIterator for $typ {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next_back()
            }
        }

        impl<'a, K, V> ExactSizeIterator for $typ {
            fn len(&self) -> usize {
                self.iter.len()
            }
        }
    };
}
impl_iter! {Drain<'a,K,V>,  (K,V)}
impl_iter! {Iter<'a,K,V>,  (&'a K, &'a V)}
impl_iter! {IterMut<'a,K,V>,  (&'a K, &'a mut V)}
impl_iter! {Keys<'a,K,V>,  &'a K}
impl_iter! {Values<'a,K,V>,  &'a V}

#[cfg(feature = "serde")]
mod serde_impl {
    use super::VecMap;
    use serde::{
        de::{Error, MapAccess, Visitor},
        ser::SerializeMap,
        Deserialize, Deserializer, Serialize, Serializer,
    };
    use std::fmt;
    use std::marker::PhantomData;

    impl<K, V> Serialize for VecMap<K, V>
    where
        K: Serialize + Eq,
        V: Serialize,
    {
        #[inline]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut state = serializer.serialize_map(Some(self.len()))?;
            for (k, v) in self {
                state.serialize_entry(k, v)?;
            }
            state.end()
        }
    }

    #[allow(missing_docs)]
    #[derive(Default)]
    struct VecMapVisitor<K, V> {
        marker: PhantomData<VecMap<K, V>>,
    }

    impl<K, V> VecMapVisitor<K, V> {
        fn new() -> Self {
            VecMapVisitor {
                marker: PhantomData,
            }
        }
    }

    impl<'de, K, V> Visitor<'de> for VecMapVisitor<K, V>
    where
        K: Deserialize<'de> + Eq,
        V: Deserialize<'de>,
    {
        type Value = VecMap<K, V>;

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "a VecMap")
        }

        #[inline]
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(VecMap::new())
        }

        #[inline]
        fn visit_map<Visitor>(self, mut visitor: Visitor) -> Result<Self::Value, Visitor::Error>
        where
            Visitor: MapAccess<'de>,
        {
            let mut values = VecMap::with_capacity(visitor.size_hint().unwrap_or(0));

            while let Some((key, value)) = visitor.next_entry()? {
                values.insert(key, value);
            }

            Ok(values)
        }
    }

    impl<'de, K, V> Deserialize<'de> for VecMap<K, V>
    where
        K: Deserialize<'de> + Eq,
        V: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<VecMap<K, V>, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_map(VecMapVisitor::new())
        }
    }
}
