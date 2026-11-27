use crate::{diagnostics::monitor::Monitor, rendering::painter::Painter};
use std::io::Result;

pub trait Renderer {
    fn render(&mut self, painter: &mut impl Painter, monitor: &mut Monitor<3>) -> Result<()>;
    fn run(&mut self, painter: &mut impl Painter, monitor: &mut Monitor<3>) -> Result<()> {
        loop {
            self.render(painter, monitor)?;
        }
    }
    fn iterate(
        &mut self,
        iterations: usize,
        painter: &mut impl Painter,
        monitor: &mut Monitor<3>,
    ) -> Result<()> {
        for _ in 0..iterations {
            self.render(painter, monitor)?;
        }

        Ok(())
    }
}
