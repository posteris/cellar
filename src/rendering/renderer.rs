use crate::rendering::painter::Painter;
use std::io::Result;

pub trait Renderer {
    type Metrics: Default;
    fn metrics(&self) -> &Self::Metrics;

    fn render(&mut self, painter: &mut impl Painter) -> Result<()>;
    fn run(&mut self, painter: &mut impl Painter) -> Result<()> {
        loop {
            self.render(painter)?;
        }
    }
    fn iterate(&mut self, iterations: usize, painter: &mut impl Painter) -> Result<()> {
        for _ in 0..iterations {
            self.render(painter)?;
        }

        Ok(())
    }
}
