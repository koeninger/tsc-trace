use tsc_trace::*;

enum Traces {
    Main = 0,
    SomeFunction = 1,
}

fn main() {
    trace!(Traces::Main);
    some_function();
}

#[inline]
fn some_function() {
    trace!(Traces::SomeFunction);
}
