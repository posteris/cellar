use std::time::{Duration, Instant};

pub struct Watch {
    laps: u32,
    elapsed: Duration,
}

impl Watch {
    #[inline]
    pub fn new() -> Self {
        let laps = 0;
        let elapsed = Duration::ZERO;
        Self { laps, elapsed }
    }

    #[inline]
    pub fn measure<T>(&mut self, function: impl FnOnce() -> T) -> T {
        let origin = Instant::now();
        let result = function();
        self.elapsed += origin.elapsed();
        self.laps += 1;
        result
    }

    #[inline]
    pub fn total(&self) -> Duration {
        self.elapsed
    }

    #[inline]
    pub fn average(&self) -> Duration {
        self.elapsed / self.laps
    }
}

impl Default for Watch {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
