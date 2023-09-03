use tsc_trace::*;

// anything that `as u64` works on can be used as a tag for traces
enum Traces {
    Main = 0,
    SomeFunction = 1,
}

fn main() -> std::io::Result<()> {
    {
        trace!(Traces::Main); // reads rdtsc to get cycle count, stores it in a stack variable
        for _ in 1..10 {
            some_function();
        }
    } // drop impl of that variable reads rdtsc again, stores the tag and both cycle counts in a thread local array

    // write the array of traces to binary file
    let mut bin = std::fs::File::create("/tmp/traces")?;
    write_binary(&mut bin)?;

    // write the array of traces to stdout in comma-separated format
    let stdout = std::io::stdout();
    let mut lock = stdout.lock();
    write_csv(&mut lock)?;

    Ok(())
}

#[inline]
fn some_function() {
    trace!(Traces::SomeFunction);
    println!("doing some work in some_function");
}
