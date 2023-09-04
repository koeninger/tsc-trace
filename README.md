Trace the number of cycles used by spans of code, via the x86 rdtsc instruction.
This is only usable on x86 / x86_64 architectures.
It will probably give questionable results unless you're pinning threads to cores.

See [main.rs](src/main.rs) for example usage.

You must use one of the features "capacity_1_million" ... "capacity_64_million".
This sets the capacity (in number of traces, not bytes) used by the thread-local array to store traces.
That array is treated as a circular buffer, so it will wrap around and overwrite traces rather than OOMing or stopping collection.
Each trace uses 24 bytes (u64 tag, u64 starting rdtsc count, u64 ending rdtsc count).
So total memory overhead is  (one usize index + (capacity * 24 bytes)) * number of threads. 

Alternatively you can use the feature "off" to set capacity to 0 and statically disable collection of traces.
This is useful if you want to leave timing markers in place for future use, but not pay any runtime overhead.

The feature "lfence" will add an lfence instruction before and after each call to rdtsc.

Run `cargo bench` to show the runtime overhead difference between using this library, vs directly calling rdtsc twice and subtracting.
