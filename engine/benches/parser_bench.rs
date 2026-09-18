use criterion::{Criterion, criterion_group, criterion_main};

fn criterion_benchmark(_c: &mut Criterion) {}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
