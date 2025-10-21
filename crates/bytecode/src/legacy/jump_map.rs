use bitvec::vec::BitVec;
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};
use primitives::{hex, Bytes, OnceLock};
use std::{fmt::Debug, sync::Arc};

/// A table of valid `jump` destinations.
///
/// It is immutable, cheap to clone and memory efficient, with one bit per byte in the bytecode.
#[derive(Clone, Eq)]
pub struct JumpTable {
    /// Cached pointer to table data to avoid Arc overhead on lookup
    table_ptr: *const u8,
    /// Number of bits in the table.
    len: usize,
    /// Actual bit vec
    table: Arc<Bytes>,
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

impl Debug for JumpTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JumpTable")
            .field("map", &hex::encode(self.table.as_ref()))
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

#[cfg(feature = "serde")]
impl serde::Serialize for JumpTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bitvec = BitVec::<u8>::from_vec(self.table.to_vec());
        bitvec.resize(self.len, false);
        bitvec.serialize(serializer)
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

impl JumpTable {
    /// Create new JumpTable directly from an existing BitVec.
    ///
    /// Uses [`Self::from_bytes`] internally.
    #[inline]
    pub fn new(jumps: BitVec<u8>) -> Self {
        let bit_len = jumps.len();
        let bytes = jumps.into_vec().into();
        Self::from_bytes(bytes, bit_len)
    }

    /// Gets the raw bytes of the jump map.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.table
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
    /// Uses [`Self::from_bytes`] internally.
    ///
    /// # Panics
    ///
    /// Panics if number of bits in slice is less than bit_len.
    #[inline]
    pub fn from_slice(slice: &[u8], bit_len: usize) -> Self {
        Self::from_bytes(Bytes::from(slice.to_vec()), bit_len)
    }

    /// Create new JumpTable directly from an existing Bytes.
    ///
    /// Bit length represents number of used bits inside slice.
    ///
    /// Panics if bytes length is less than bit_len * 8.
    #[inline]
    pub fn from_bytes(bytes: Bytes, bit_len: usize) -> Self {
        Self::from_bytes_arc(Arc::new(bytes), bit_len)
    }

    /// Create new JumpTable directly from an existing Bytes.
    ///
    /// Bit length represents number of used bits inside slice.
    ///
    /// Panics if bytes length is less than bit_len * 8.
    #[inline]
    pub fn from_bytes_arc(table: Arc<Bytes>, bit_len: usize) -> Self {
        const BYTE_LEN: usize = 8;
        assert!(
            table.len() * BYTE_LEN >= bit_len,
            "slice bit length {} is less than bit_len {}",
            table.len() * BYTE_LEN,
            bit_len
        );

        let table_ptr = table.as_ptr();

        Self {
            table_ptr,
            table,
            len: bit_len,
        }
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
    #[cfg(feature = "serde")]
    fn test_serde_legacy_format() {
        let legacy_format = r#"
        {
            "order": "bitvec::order::Lsb0",
            "head": {
                "width": 8,
                "index": 0
            },
            "bits": 4,
            "data": [5]
        }"#;

        let table: JumpTable = serde_json::from_str(legacy_format).expect("Failed to deserialize");
        assert_eq!(table.len, 4);
        assert!(table.is_valid(0));
        assert!(!table.is_valid(1));
        assert!(table.is_valid(2));
        assert!(!table.is_valid(3));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_roundtrip() {
        let original = JumpTable::from_slice(&[0x0D, 0x06], 13);

        // Serialize to JSON
        let serialized = serde_json::to_string(&original).expect("Failed to serialize");

        // Deserialize from JSON
        let deserialized: JumpTable =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Check that the deserialized table matches the original
        assert_eq!(original.len, deserialized.len);
        assert_eq!(original.table, deserialized.table);

        // Verify functionality is preserved
        for i in 0..13 {
            assert_eq!(
                original.is_valid(i),
                deserialized.is_valid(i),
                "Mismatch at index {i}"
            );
        }
    }
}

#[cfg(test)]
mod bench_is_valid {
    use super::*;
    use std::{sync::Arc, time::Instant};

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
