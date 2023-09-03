use std::cell::RefCell;

#[cfg(not(feature = "off"))]
const CAPACITY: usize = 16384;

#[cfg(feature = "off")]
const CAPACITY: usize = 0;

thread_local! {
    static TSC_TRACE_START: RefCell<Vec<u64>> = RefCell::new(Vec::with_capacity(CAPACITY));

    static TSC_TRACE_STOP: RefCell<Vec<u64>> = RefCell::new(Vec::with_capacity(CAPACITY));

    static TSC_TRACE_TAG: RefCell<Vec<u8>> = RefCell::new(Vec::with_capacity(CAPACITY));
}

/// how many traces have been recorded
pub fn len() -> usize {
    TSC_TRACE_TAG.with(|v| v.borrow().len())    
}

pub struct Printer {}

pub fn print_traces() {
    TSC_TRACE_TAG.with(|tags| {
        TSC_TRACE_START.with(|starts| {
            TSC_TRACE_STOP.with(|stops| {
                let tags = tags.borrow();
                let starts = starts.borrow();
                let stops = stops.borrow();
                for ((start, stop), tag) in starts.iter().zip(stops.iter()).zip(tags.iter()) {
                    println!("{tag} {start} {stop} {}", stop - start);
                }
            })
        })
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
    tag: u8,
    start: u64,
}

impl Span {
    pub fn new(tag: u8) -> Self {
        Span {
            tag,
            start: rdtsc(),
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        let stop = rdtsc();
        TSC_TRACE_START.with(|v| v.borrow_mut().push(self.start));
        TSC_TRACE_STOP.with(|v| v.borrow_mut().push(stop));
        TSC_TRACE_TAG.with(|v| v.borrow_mut().push(self.tag));
    }
}

#[macro_export]
#[cfg(not(feature = "off"))]
macro_rules! trace {
    ($e:expr) => {
        let _tsc_trace_span = Span::new($e as u8);
    };
}

#[macro_export]
#[cfg(feature = "off")]
macro_rules! trace {
    ($e:expr) => {};
}
