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

Dependency on SDL2 and ttf. Instructions for installation can be found at (https://crates.io/crates/sdl2).

A visual representation of cycles gathered by tsc-trace.

Takes a file that has traces written to it by write_traces_binary through command line arguments, format:
(file path) (span range start) (span range stop) (tag range start) (tag range stop)

e.g. `cargo run --release /Users/Koeninger/Downloads/my_trace 5000 25000 5 18`

will display spans from the my_trace file with starts greater than or equal to 5000 clock cycles and less than or equal to 25000 clock cycles, provided their tags are between 5 and 18.

File path is required, start arguments will default to 0 and stop arguments will default to u64::MAX if not provided.
The default arguments can be changed by editing config.js.

Tag numbers can be replaced with strings (to "name" tags) by editing config.js.

Use Q, W, E to zoom out, in, and reset.
Use A, S, D to move left, right, and reset.

Instances where spans are so small that multiple may be drawn per pixel are represented by a lighter colorset being used.
Clicking on a span will display the tag number (or name) and span length, as well as printing the standard tag csv tag representation to stdout. 
Clicking a non-span or lighter colorset span will instead print what tag would occupy that area and an approximate position (in clock cycles).