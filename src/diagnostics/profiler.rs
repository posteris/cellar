use crate::{
    diagnostics::{chaos::Chaos, watch::Watch},
    rendering::ansi::terminal::Terminal,
};
use std::{io::Result, time::Duration};

#[derive(Default)]
pub struct Profiler {
    events: Watch,
    paint: Watch,
    present: Watch,
    flush: Watch,
}

impl Profiler {
    #[inline]
    pub fn bench(&mut self, terminal: &mut Terminal, iterations: usize) -> Result<()> {
        let mut chaos = Chaos::new();

        for _ in 0..iterations {
            {
                let _sample = self.events.sample();
                terminal.events()?;
            }

            {
                let _sample = self.paint.sample();
                terminal.paint(&mut chaos);
            }

            {
                let _sample = self.present.sample();
                terminal.present();
            }

            {
                let _sample = self.flush.sample();
                terminal.flush()?;
            }
        }

        Ok(())
    }

    #[inline]
    pub fn averages(&self) -> [(&'static str, Duration); 4] {
        [
            ("events", self.events.average()),
            ("paint", self.paint.average()),
            ("present", self.present.average()),
            ("flush", self.flush.average()),
        ]
    }
}
