# tsc-trace

[![Crates.io](https://img.shields.io/crates/v/tsc-trace.svg)](https://crates.io/crates/tsc-trace)

Trace the number of cycles used by spans of code, via the x86 rdtsc instruction.
This is only usable on x86 / x86_64 architectures.
It will probably give questionable results unless you're pinning threads to cores.

See [main.rs](https://github.com/koeninger/tsc-trace/blob/main/src/main.rs) for example usage.

The features `"capacity_1_million"` ... `"capacity_64_million"` set the capacity (in number of traces, not bytes) used by the thread-local vec to store traces.
Default is 1 million.
That vec is treated as a circular buffer, so it will wrap around and overwrite traces rather than reallocating, OOMing or stopping collection.
Each trace uses 24 bytes (u64 tag, u64 starting rdtsc count, u64 ending rdtsc count).
So total memory overhead is:

(1 usize for index + (capacity * 24 bytes)) * number of threads. 

Alternatively you can use the feature `"off"` to set capacity to 0 and statically disable collection of traces.
This is useful if you want to leave timing markers in place for future use, but not pay any runtime overhead.

The feature `"const_array"` will use a const array rather than a vec for the thread local storage of traces.

The feature `"lfence"` will add an lfence instruction before and after each call to rdtsc.

Run e.g. `cargo bench --features "tsc-trace/capacity_1_million"` to show the runtime overhead difference between using this library, vs directly calling rdtsc twice and subtracting.
