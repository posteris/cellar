use crate::{
    diagnostics::{chaos::Chaos, watch::Watch},
    rendering::{frame::Frame, painter::Painter},
};
use std::{io::Result, time::Duration};

pub trait Renderer {
    fn render(&mut self, paint: impl FnMut(&mut Frame)) -> Result<()>;

    fn run(&mut self, painter: &mut impl Painter) -> Result<()> {
        loop {
            self.render(|f| painter.paint(f))?;
        }
    }

    fn bench(&mut self, iterations: u32) -> Result<Duration> {
        let mut painter = Chaos::new();
        let mut watch = Watch::new();

        for _ in 0..iterations {
            watch.measure(|| self.render(|f| painter.paint(f)))?;
        }

        Ok(watch.average())
    }
}
