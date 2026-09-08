//! Installed with one representation-specific import; fixtures otherwise identical.
EXTENSION_IMPORT
use criterion::{criterion_group, criterion_main, Criterion, BatchSize};
use std::hint::black_box;

fn storage(c: &mut Criterion) {
    eprintln!("extension_size={}", size_of::<Extension>());
    for n in [0, 32] {
        let payload = vec![42u8; n];
        let value = Extension::copy_from_slice(&payload);
        let independent = Extension::copy_from_slice(&payload);
        let shared = value.clone();
        c.bench_function(&format!("storage/construct_slice/{n}"), |b| b.iter(|| black_box(Extension::copy_from_slice(black_box(&payload)))));
        c.bench_function(&format!("storage/construct_vec/{n}"), |b| b.iter_batched(|| payload.clone(), |v| black_box(Extension::from(v)), BatchSize::SmallInput));
        c.bench_function(&format!("storage/clone/{n}"), |b| b.iter(|| black_box(black_box(&value).clone())));
        c.bench_function(&format!("storage/equal_independent/{n}"), |b| b.iter(|| black_box(&value) == black_box(&independent)));
        c.bench_function(&format!("storage/equal_shared/{n}"), |b| b.iter(|| black_box(&value) == black_box(&shared)));
    }
}
criterion_group!(benches, storage);
criterion_main!(benches);
