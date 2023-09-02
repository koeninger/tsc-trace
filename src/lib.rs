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
    start: u64
}

impl Span {
    pub fn new(tag: u8) -> Self {
        Span { tag, start: rdtsc() }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        let stop = rdtsc();
        let diff = stop - self.start;
        // eprintln!("{} {} {} {}", self.tag, self.start, stop, diff);
    }
}

#[macro_export]
#[cfg(not(feature = "off"))]
macro_rules! trace {
    ($e:expr) => {
        let _tsc_trace_span = Span::new($e as u8);
    }
}

#[macro_export]
#[cfg(feature = "off")]
macro_rules! trace {
    ($e:expr) => {
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
