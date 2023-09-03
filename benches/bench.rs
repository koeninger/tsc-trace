use criterion::*;
use tsc_trace::*;
use std::time::Duration;

#[inline]
fn direct() {
    let tag = 0;
    let start = rdtsc();
    let stop = rdtsc();
    let diff = stop - start;
    black_box(diff);
    black_box(tag);
    //    eprintln!("{} {} {} {}", tag, start, stop, diff);
}

#[inline]
fn macroed() {
    trace!(2);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("tsc");
    let group = group.measurement_time(Duration::from_millis(1000)).warm_up_time(Duration::from_millis(1000));
    group.bench_function("direct", |b| b.iter(|| black_box(direct())));
    group.bench_function("macroed", |b| b.iter(|| black_box(macroed())));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
