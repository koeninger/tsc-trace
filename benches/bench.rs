use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tsc_trace::*;

#[inline]
fn direct() {
    let tag = 0;
    let start = rdtsc();
    let stop = rdtsc();
    let diff = stop - start;
  
//    eprintln!("{} {} {} {}", tag, start, stop, diff);
}

#[inline]
fn named() {
    let _x = Span::new(1);
}

#[inline]
fn macroed() {
    trace!(2);
}


fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("direct", |b| b.iter(|| black_box(direct())));
    c.bench_function("named", |b| b.iter(|| black_box(named())));
    c.bench_function("macroed", |b| b.iter(|| black_box(named())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
