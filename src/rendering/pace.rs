use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug)]
pub enum Pace {
    Next,
    At(Instant),
    Wait,
}

impl Pace {
    #[inline]
    pub const fn interval(fps: u64) -> Duration {
        const ONE_SECOND: u64 = 1_000_000_000;
        Duration::from_nanos(ONE_SECOND / fps)
    }

    #[inline]
    pub fn from(interval: Duration) -> Pace {
        let origin = Instant::now();
        Self::At(origin + interval)
    }
}
