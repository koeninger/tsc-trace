use std::cell::{ RefCell, Cell };

#[cfg(all(not(feature = "off"), feature = "capacity_1_million"))]
const CAPACITY: usize = 1_000_000 * 3;

#[cfg(all(not(feature = "off"), feature = "capacity_8_million"))]
const CAPACITY: usize = 8_000_000 * 3;

#[cfg(all(not(feature = "off"), feature = "capacity_16_million"))]
const CAPACITY: usize = 16_000_000 * 3;

#[cfg(all(not(feature = "off"), feature = "capacity_32_million"))]
const CAPACITY: usize = 32_000_000 * 3;

#[cfg(all(not(feature = "off"), feature = "capacity_64_million"))]
const CAPACITY: usize = 64_000_000 * 3;

#[cfg(feature = "off")]
const CAPACITY: usize = 0;

#[cfg(all(not(feature = "off"), not(feature = "capacity_1_million"), not(feature = "capacity_8_million"), not(feature = "capacity_16_million"), not(feature = "capacity_32_million"), not(feature = "capacity_64_million")))]
compile_error!("tsc-trace requires enabling exactly one of the features 'capacity_1_million' ... 'capacity_64_million', or 'off'");
#[cfg(all(not(feature = "off"), not(feature = "capacity_1_million"), not(feature = "capacity_8_million"), not(feature = "capacity_16_million"), not(feature = "capacity_32_million"), not(feature = "capacity_64_million")))]
const CAPACITY: usize = 0;

thread_local! {
    static TSC_TRACE_SPANS: RefCell<[u64; CAPACITY]> = const { RefCell::new([0; CAPACITY]) };
    static TSC_TRACE_INDEX: Cell<usize> = const { Cell::new(0) };
}

pub struct Printer {}

// TODO replace with macro
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
    #[cfg(feature = "lfence")]
    use core::arch::x86::_mm_lfence;
    unsafe {
        #[cfg(feature = "lfence")]
        _mm_lfence();
        let r = _rdtsc();
        #[cfg(feature = "lfence")]
        _mm_lfence();
        r
    }
}

#[inline(always)]
#[cfg(target_arch = "x86_64")]
pub fn rdtsc() -> u64 {
    use core::arch::x86_64::_rdtsc;
    use core::arch::x86_64::_mm_lfence;
    unsafe {
        #[cfg(feature = "lfence")]
        _mm_lfence();
        let r = _rdtsc();
        #[cfg(feature = "lfence")]
        _mm_lfence();
        r
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
pub fn rdtsc() -> u64 {
    unimplemented!("x86 needed for rdtsc")
}

/// Use the trace! macro, do not use this directly.
pub struct Span {
    tag: u64,
    start: u64,
}

impl Span {
    /// Do not call this, use the trace! macro instead
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
