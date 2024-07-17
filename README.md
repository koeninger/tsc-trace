# tsc-trace

[![Crates.io](https://img.shields.io/crates/v/tsc-trace.svg)](https://crates.io/crates/tsc-trace)

Trace the number of cycles used by spans of code, via the x86 rdtsc instruction or reading ARM cntvct_el0 register.
It will probably give questionable results unless you're pinning threads to cores.

See [main.rs](https://github.com/koeninger/tsc-trace/blob/main/src/main.rs) for example usage.

The features `"capacity_1_million"` ... `"capacity_64_million"` set the capacity (in number of traces, not bytes) used by the thread-local vec to store traces.
Default is 1 million.
That vec is treated as a circular buffer, so it will wrap around and overwrite traces rather than reallocating, OOMing or stopping collection.
Each trace uses 24 bytes (u64 tag, u64 starting count, u64 ending count).
So total memory overhead is:

(1 usize for index + (capacity * 24 bytes)) * number of threads. 

Alternatively you can use the feature `"off"` to set capacity to 0 and statically disable collection of traces.
This is useful if you want to leave timing markers in place for future use, but not pay any runtime overhead.

The feature `"const_array"` will use a const array rather than a vec for the thread local storage of traces.

The feature `"lfence"` will add an lfence instruction before and after each call to rdtsc (x86 only).

Run e.g. `cargo bench --features "tsc-trace/capacity_1_million"` to show the runtime overhead difference between using this library, vs directly calling rdtsc twice and subtracting.

## Viewer

Dependency on SDL2 (https://crates.io/crates/sdl2) and bytemuck (https://crates.io/crates/bytemuck).

A visual representation of cycles gathered by tsc-trace.

Takes a file that has traces written to it by write_traces_binary through command line arguments, format:
(file path) (span range start) (span range stop) (tag range start) (tag range stop)

File path is required, start arguments will default to 0 and stop arguments will default to u64::MAX if not provided.

The span start and stop ranges are in number of clock cycles after the start of the first trace in the file.

Use Q, W, E to zoom out, in, and reset.

Use A, S, D to move right, left, and reset.

Clicking on span will display the tag and span length. This will not work in instances where spans are so small that multiple may be drawn per pixel.
(these instances are represented by a lighter colorset being used)

Tag numbers can be replaced with strings by editing config.js.