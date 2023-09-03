use tsc_trace::*;

enum Traces {
    Main = 0,
    SomeFunction = 1,
}

fn main() {
    let _p = Printer {};
    trace!(Traces::Main);
    for _ in 1..10 {
        some_function();
    }
}

#[inline]
fn some_function() {
    trace!(Traces::SomeFunction);
}
