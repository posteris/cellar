use crate::diagnostics::watch::Watch;
use crate::rendering::ansi::platform::{raw_stdout, terminal_size};
use crate::rendering::ansi::writer::Writer;
use crate::rendering::canvas::Canvas;
use crate::rendering::painter::Painter;
use crate::rendering::renderer::Renderer;
use std::fs::File;
use std::io::{Result, Write};

#[derive(Default)]
pub struct Metrics {
    pub paint: Watch,
    pub present: Watch,
    pub flush: Watch,
}

pub struct Terminal {
    metrics: Metrics,
    canvas: Canvas,
    writer: Writer,
    output: File,
}

impl Terminal {
    #[inline]
    pub fn new() -> Result<Self> {
        let metrics = Metrics::default();
        let mut output = raw_stdout()?;
        let size = terminal_size(&output)?;

        let canvas = Canvas::new(size);
        let mut writer = Writer::new(size);
        writer.begin();

        output.write_all(writer.written())?;
        writer.clear();

        Ok(Self {
            metrics,
            canvas,
            writer,
            output,
        })
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.writer.end();
        let _ = self.output.write_all(self.writer.written());
    }
}

impl Renderer for Terminal {
    type Metrics = Metrics;

    #[inline]
    fn metrics(&self) -> &Self::Metrics {
        &self.metrics
    }

    #[inline]
    fn render(&mut self, painter: &mut impl Painter) -> Result<()> {
        let size = terminal_size(&self.output)?;
        if self.canvas.size() != size {
            self.canvas.resize(size);
            self.writer.resize(size);
        }

        {
            let _lap = self.metrics.paint.lap();
            painter.paint(&mut self.canvas);
        }

        {
            let _lap = self.metrics.present.lap();
            for delta in self.canvas.delta() {
                self.writer.emit_move(delta.position());
                self.writer.emit_foreground(delta.cell().foreground());
                self.writer.emit_background(delta.cell().background());
                self.writer.emit_glyph_u8(delta.cell().glyph());
            }
        }

        {
            let _lap = self.metrics.flush.lap();
            if let Err(error) = self.output.write_all(self.writer.written()) {
                self.writer.forget();
                self.writer.clear();
                return Err(error);
            }
        }

        self.writer.clear();
        self.canvas.clear_delta();
        Ok(())
    }
}
