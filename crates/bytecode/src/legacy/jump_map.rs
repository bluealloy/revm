use bitvec::vec::BitVec;
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};
use primitives::{hex, OnceLock};
use std::{fmt::Debug, sync::Arc};

/// A table of valid `jump` destinations.
///
/// It is immutable, cheap to clone and memory efficient, with one bit per byte in the bytecode.
#[derive(Clone, Eq)]
pub struct JumpTable {
    /// Pointer into `table` to avoid `Arc` overhead on lookup.
    table_ptr: *const u8,
    /// Number of bits in the table.
    len: usize,
    /// Actual bit vec
    table: Arc<BitVec<u8>>,
}

// SAFETY: BitVec data is immutable through Arc, pointer won't be invalidated
unsafe impl Send for JumpTable {}
unsafe impl Sync for JumpTable {}

impl PartialEq for JumpTable {
    fn eq(&self, other: &Self) -> bool {
        self.table.eq(&other.table)
    }
}

impl Hash for JumpTable {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.table.hash(state);
    }
}

impl PartialOrd for JumpTable {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JumpTable {
    fn cmp(&self, other: &Self) -> Ordering {
        self.table.cmp(&other.table)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for JumpTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.table.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for JumpTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bitvec = BitVec::deserialize(deserializer)?;
        Ok(Self::new(bitvec))
    }
}

impl Debug for JumpTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JumpTable")
            .field("map", &hex::encode(self.table.as_raw_slice()))
            .finish()
    }
}

impl Default for JumpTable {
    #[inline]
    fn default() -> Self {
        static DEFAULT: OnceLock<JumpTable> = OnceLock::new();
        DEFAULT.get_or_init(|| Self::new(BitVec::default())).clone()
    }
}

impl JumpTable {
    /// Create new JumpTable directly from an existing BitVec.
    pub fn new(jumps: BitVec<u8>) -> Self {
        let table = Arc::new(jumps);
        let table_ptr = table.as_raw_slice().as_ptr();
        let len = table.len();

        Self {
            table,
            table_ptr,
            len,
        }
    }

    /// Gets the raw bytes of the jump map.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.table.as_raw_slice()
    }

    /// Gets the length of the jump map.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the jump map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Constructs a jump map from raw bytes and length.
    ///
    /// Bit length represents number of used bits inside slice.
    ///
    /// # Panics
    ///
    /// Panics if number of bits in slice is less than bit_len.
    #[inline]
    pub fn from_slice(slice: &[u8], bit_len: usize) -> Self {
        const BYTE_LEN: usize = 8;
        assert!(
            slice.len() * BYTE_LEN >= bit_len,
            "slice bit length {} is less than bit_len {}",
            slice.len() * BYTE_LEN,
            bit_len
        );
        let mut bitvec = BitVec::from_slice(slice);
        unsafe { bitvec.set_len(bit_len) };
        Self::new(bitvec)
    }

    /// Checks if `pc` is a valid jump destination.
    /// Uses cached pointer and bit operations for faster access
    #[inline]
    pub fn is_valid(&self, pc: usize) -> bool {
        pc < self.len && unsafe { *self.table_ptr.add(pc >> 3) & (1 << (pc & 7)) != 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "slice bit length 8 is less than bit_len 10")]
    fn test_jump_table_from_slice_panic() {
        let slice = &[0x00];
        let _ = JumpTable::from_slice(slice, 10);
    }

    #[test]
    fn test_jump_table_from_slice() {
        let slice = &[0x00];
        let jumptable = JumpTable::from_slice(slice, 3);
        assert_eq!(jumptable.len, 3);
    }

    #[test]
    fn test_is_valid() {
        let jump_table = JumpTable::from_slice(&[0x0D, 0x06], 13);

        assert_eq!(jump_table.len, 13);

        assert!(jump_table.is_valid(0)); // valid
        assert!(!jump_table.is_valid(1));
        assert!(jump_table.is_valid(2)); // valid
        assert!(jump_table.is_valid(3)); // valid
        assert!(!jump_table.is_valid(4));
        assert!(!jump_table.is_valid(5));
        assert!(!jump_table.is_valid(6));
        assert!(!jump_table.is_valid(7));
        assert!(!jump_table.is_valid(8));
        assert!(jump_table.is_valid(9)); // valid
        assert!(jump_table.is_valid(10)); // valid
        assert!(!jump_table.is_valid(11));
        assert!(!jump_table.is_valid(12));
    }

    #[test]
    fn test_as_slice() {
        let data = &[0x0D, 0x06];
        let jump_table = JumpTable::from_slice(data, 13);

        let slice = jump_table.as_slice();
        assert_eq!(slice, data);
    }

    #[test]
    fn test_is_empty() {
        // Empty jump table
        let empty_table = JumpTable::default();
        assert!(empty_table.is_empty());
        assert_eq!(empty_table.len(), 0);

        // Non-empty jump table
        let non_empty_table = JumpTable::from_slice(&[0x00], 5);
        assert!(!non_empty_table.is_empty());
        assert_eq!(non_empty_table.len(), 5);
    }

    #[test]
    fn test_partial_cmp() {
        let table1 = JumpTable::from_slice(&[0x00], 5);
        let table2 = JumpTable::from_slice(&[0x01], 5);
        let table3 = JumpTable::from_slice(&[0x00], 5);

        assert_eq!(table1.partial_cmp(&table2), Some(Ordering::Less));
        assert_eq!(table2.partial_cmp(&table1), Some(Ordering::Greater));
        assert_eq!(table1.partial_cmp(&table3), Some(Ordering::Equal));
    }

    #[test]
    fn test_eq() {
        let table1 = JumpTable::from_slice(&[0x0D, 0x06], 13);
        let table2 = JumpTable::from_slice(&[0x0D, 0x06], 13);
        let table3 = JumpTable::from_slice(&[0x0D, 0x07], 13);

        assert_eq!(table1, table2);
        assert_ne!(table1, table3);
    }

    #[test]
    fn test_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let table1 = JumpTable::from_slice(&[0x0D, 0x06], 13);
        let table2 = JumpTable::from_slice(&[0x0D, 0x06], 13);

        let mut hasher1 = DefaultHasher::new();
        table1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        table2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_cmp() {
        let table1 = JumpTable::from_slice(&[0x00], 5);
        let table2 = JumpTable::from_slice(&[0x01], 5);

        assert_eq!(table1.cmp(&table2), Ordering::Less);
        assert_eq!(table2.cmp(&table1), Ordering::Greater);
        assert_eq!(table1.cmp(&table1), Ordering::Equal);
    }

    #[test]
    fn test_clone() {
        let table = JumpTable::from_slice(&[0x0D, 0x06], 13);
        let cloned = table.clone();

        assert_eq!(table, cloned);
        assert_eq!(table.len(), cloned.len());
        assert_eq!(table.as_slice(), cloned.as_slice());
    }

    #[test]
    fn test_debug() {
        let table = JumpTable::from_slice(&[0x0D, 0x06], 13);
        let debug_str = format!("{:?}", table);

        assert!(debug_str.contains("JumpTable"));
        assert!(debug_str.contains("map"));
    }

    #[test]
    fn test_default() {
        let table1 = JumpTable::default();
        let table2 = JumpTable::default();

        // Default tables should be equal
        assert_eq!(table1, table2);
        assert!(table1.is_empty());

        // Should reuse the same default instance (OnceLock)
        assert_eq!(Arc::as_ptr(&table1.table), Arc::as_ptr(&table2.table));
    }

    #[test]
    fn test_new() {
        use bitvec::bitvec;

        let mut bitvec = bitvec![u8, bitvec::order::Lsb0; 0; 10];
        bitvec.set(0, true);
        bitvec.set(5, true);

        let table = JumpTable::new(bitvec);
        assert_eq!(table.len(), 10);
        assert!(table.is_valid(0));
        assert!(table.is_valid(5));
        assert!(!table.is_valid(1));
    }

    #[test]
    fn test_is_valid_out_of_bounds() {
        let table = JumpTable::from_slice(&[0x0D], 8);

        // Out of bounds should return false
        assert!(!table.is_valid(8));
        assert!(!table.is_valid(100));
        assert!(!table.is_valid(usize::MAX));
    }

    #[test]
    fn test_from_slice_exact_bits() {
        // Test when bit_len is exactly the number of bits in the slice
        let table = JumpTable::from_slice(&[0xFF, 0xFF], 16);
        assert_eq!(table.len(), 16);

        // All bits should be valid
        for i in 0..8 {
            assert!(table.is_valid(i));
        }
    }

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<JumpTable>();
        assert_sync::<JumpTable>();
    }
}

#[cfg(test)]
mod bench_is_valid {
    use super::*;
    use std::time::Instant;

    const ITERATIONS: usize = 1_000_000;
    const TEST_SIZE: usize = 10_000;

    fn create_test_table() -> BitVec<u8> {
        let mut bitvec = BitVec::from_vec(vec![0u8; TEST_SIZE.div_ceil(8)]);
        bitvec.resize(TEST_SIZE, false);
        for i in (0..TEST_SIZE).step_by(3) {
            bitvec.set(i, true);
        }
        bitvec
    }

    #[derive(Clone)]
    pub(super) struct JumpTableWithArcDeref(pub Arc<BitVec<u8>>);

    impl JumpTableWithArcDeref {
        #[inline]
        pub(super) fn is_valid(&self, pc: usize) -> bool {
            pc < self.0.len() && unsafe { *self.0.get_unchecked(pc) }
        }
    }

    fn benchmark_implementation<F>(name: &str, table: &F, test_fn: impl Fn(&F, usize) -> bool)
    where
        F: Clone,
    {
        // Warmup
        for i in 0..10_000 {
            std::hint::black_box(test_fn(table, i % TEST_SIZE));
        }

        let start = Instant::now();
        let mut count = 0;

        for i in 0..ITERATIONS {
            if test_fn(table, i % TEST_SIZE) {
                count += 1;
            }
        }

        let duration = start.elapsed();
        let ns_per_op = duration.as_nanos() as f64 / ITERATIONS as f64;
        let ops_per_sec = ITERATIONS as f64 / duration.as_secs_f64();

        println!("{name} Performance:");
        println!("  Time per op: {ns_per_op:.2} ns");
        println!("  Ops per sec: {ops_per_sec:.0}");
        println!("  True count: {count}");
        println!();

        std::hint::black_box(count);
    }

    #[test]
    fn bench_is_valid() {
        println!("JumpTable is_valid() Benchmark Comparison");
        println!("=========================================");

        let bitvec = create_test_table();

        // Test cached pointer implementation
        let cached_table = JumpTable::new(bitvec.clone());
        benchmark_implementation("JumpTable (Cached Pointer)", &cached_table, |table, pc| {
            table.is_valid(pc)
        });

        // Test Arc deref implementation
        let arc_table = JumpTableWithArcDeref(Arc::new(bitvec));
        benchmark_implementation("JumpTableWithArcDeref (Arc)", &arc_table, |table, pc| {
            table.is_valid(pc)
        });

        println!("Benchmark completed successfully!");
    }

    #[test]
    fn bench_different_access_patterns() {
        let bitvec = create_test_table();
        let cached_table = JumpTable::new(bitvec.clone());
        let arc_table = JumpTableWithArcDeref(Arc::new(bitvec));

        println!("Access Pattern Comparison");
        println!("========================");

        // Sequential access
        let start = Instant::now();
        for i in 0..ITERATIONS {
            std::hint::black_box(cached_table.is_valid(i % TEST_SIZE));
        }
        let cached_sequential = start.elapsed();

        let start = Instant::now();
        for i in 0..ITERATIONS {
            std::hint::black_box(arc_table.is_valid(i % TEST_SIZE));
        }
        let arc_sequential = start.elapsed();

        // Random access
        let start = Instant::now();
        for i in 0..ITERATIONS {
            std::hint::black_box(cached_table.is_valid((i * 17) % TEST_SIZE));
        }
        let cached_random = start.elapsed();

        let start = Instant::now();
        for i in 0..ITERATIONS {
            std::hint::black_box(arc_table.is_valid((i * 17) % TEST_SIZE));
        }
        let arc_random = start.elapsed();

        println!("Sequential Access:");
        println!(
            "  Cached: {:.2} ns/op",
            cached_sequential.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "  Arc:    {:.2} ns/op",
            arc_sequential.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "  Speedup: {:.1}x",
            arc_sequential.as_nanos() as f64 / cached_sequential.as_nanos() as f64
        );

        println!();
        println!("Random Access:");
        println!(
            "  Cached: {:.2} ns/op",
            cached_random.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "  Arc:    {:.2} ns/op",
            arc_random.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "  Speedup: {:.1}x",
            arc_random.as_nanos() as f64 / cached_random.as_nanos() as f64
        );
    }
}
