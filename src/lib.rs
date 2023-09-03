use std::cell::{ RefCell, Cell };

#[cfg(not(feature = "off"))]
const CAPACITY: usize = 1_000_000 * 3;

#[cfg(feature = "off")]
const CAPACITY: usize = 0;

thread_local! {
    static TSC_TRACE_SPANS: RefCell<[u64; CAPACITY]> = const { RefCell::new([0; CAPACITY]) };
    static TSC_TRACE_INDEX: Cell<usize> = const { Cell::new(0) };
}

pub struct Printer {}

pub fn print_traces() {
    TSC_TRACE_SPANS.with(|spans| {
        let spans = spans.borrow();
        for i in (0..CAPACITY).step_by(3) {
            let tag = spans[i];
            let start = spans[i+1];
            let stop = spans[i+2];
            println!("{tag} {start} {stop} {}", stop - start);
        }
    });
}

impl Drop for Printer {
    fn drop(&mut self) {
        print_traces();
    }
}

#[inline(always)]
#[cfg(target_arch = "x86")]
pub fn rdtsc() -> u64 {
    use core::arch::x86::_rdtsc;
    unsafe { _rdtsc() }
}

#[inline(always)]
#[cfg(target_arch = "x86_64")]
pub fn rdtsc() -> u64 {
    use core::arch::x86_64::_rdtsc;
    unsafe { _rdtsc() }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn rdtsc() -> u64 {
    unimplemented!("x86 needed for rdtsc")
}

pub struct Span {
    tag: u64,
    start: u64,
}

impl Span {
    pub fn new(tag: u64) -> Self {
        Span {
            tag,
            start: rdtsc(),
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        let stop = rdtsc();
        TSC_TRACE_INDEX.with(|index| {
            let mut i = index.get();
            if i >= CAPACITY {
                i = 0;
            }
            TSC_TRACE_SPANS.with(|spans| {
                let mut spans = spans.borrow_mut();
                spans[i] = self.tag;
                i += 1;
                spans[i] = self.start;
                i += 1;
                spans[i] = stop;
                i += 1;
            });
            index.set(i);
        })
    }
}

#[macro_export]
#[cfg(not(feature = "off"))]
macro_rules! trace {
    ($e:expr) => {
        let _tsc_trace_span = Span::new(($e) as u64);
    };
}

#[macro_export]
#[cfg(feature = "off")]
macro_rules! trace {
    ($e:expr) => {};
}
