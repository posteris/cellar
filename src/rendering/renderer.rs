use crate::{
    common::size::Size,
    diagnostics::{chaos::Chaos, watch::Watch},
    rendering::{frame::Frame, painter::Painter},
};
use std::{io::Result, time::Duration};

pub trait Renderer {
    fn size(&self) -> Result<Size>;
    fn present(&mut self, frame: &mut Frame) -> Result<()>;

    fn run(&mut self, painter: &mut impl Painter) -> Result<()> {
        let mut frame = Frame::new(self.size()?);

        loop {
            frame.resize(self.size()?);
            painter.paint(&mut frame);
            self.present(&mut frame)?;
        }
    }

    fn bench(&mut self, iterations: u32) -> Result<[Duration; 4]> {
        let mut frame = Frame::new(self.size()?);
        let mut painter = Chaos::new();

        let mut size_watch = Watch::new();
        let mut resize_watch = Watch::new();
        let mut paint_watch = Watch::new();
        let mut present_watch = Watch::new();

        for _ in 0..iterations {
            let size = size_watch.measure(|| self.size())?;
            resize_watch.measure(|| frame.resize(size));
            paint_watch.measure(|| painter.paint(&mut frame));
            present_watch.measure(|| self.present(&mut frame))?;
        }

        Ok([
            size_watch.average(),
            resize_watch.average(),
            paint_watch.average(),
            present_watch.average(),
        ])
    }
}
