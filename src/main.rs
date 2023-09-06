// import wildcard to avoid unused import warnings when the "off" feature is enabled.
use tsc_trace::*;

// anything that `as u64` works on can be used as a tag for traces
#[allow(dead_code)]
enum Traces {
    Main = 0,
    SomeFunction = 1,
    SomeEvent = 2,
}

fn main() -> std::io::Result<()> {
    {
        trace_span!(Traces::Main); // reads rdtsc to get cycle count, stores it in a stack variable
        for _ in 1..10 {
            some_function();
        }
    } // drop impl of that variable reads rdtsc again, stores the tag and both cycle counts in a thread local array

    // immediately store info (normally tag, start, stop, but can be any 3 u64 you want) to the array of traces
    insert_trace!(
        Traces::SomeEvent,
        rdtsc(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos());

    // write the array of traces to stdout in comma-separated format
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    write_traces_csv(&mut lock)?;

    // can do runtime checks against configured capacity
    if TSC_TRACE_CAPACITY > 0 {
      // write the array of traces to binary file
      let mut bin = std::fs::File::create("/tmp/traces")?;
      write_traces_binary(&mut bin)?;
    } else {
      println!("tracing is off, not writing binary file");
    }

    Ok(())
}

fn some_function() {
    trace_span!(Traces::SomeFunction);
    println!("doing some work in some_function");
}
