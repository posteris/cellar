use std::{
    fs::File,
    io::{Result, Write},
    thread,
    time::{Duration, Instant},
};

use crate::rendering::{
    ansi::{
        platform::{raw_stdout, terminal_size},
        writer::Writer,
    },
    canvas::Canvas,
    pace::Pace,
    painter::Painter,
};

const WAIT_POLL_INTERVAL: Duration = Duration::from_millis(50);

pub struct Terminal {
    canvas: Canvas,
    writer: Writer,
    output: File,
}

impl Terminal {
    #[inline]
    pub fn new() -> Result<Self> {
        let mut output = raw_stdout()?;
        let size = terminal_size(&output)?;

        let canvas = Canvas::new(size);
        let mut writer = Writer::new(size);
        writer.begin();

        output.write_all(writer.written())?;
        writer.clear();

        Ok(Self {
            canvas,
            writer,
            output,
        })
    }

    pub fn run(&mut self, painter: &mut impl Painter) -> Result<()> {
        loop {
            self.events()?;

            let pace = self.paint(painter);

            self.present();
            self.flush()?;

            match pace {
                Pace::Next => {}
                Pace::At(deadline) => {
                    thread::sleep(deadline.saturating_duration_since(Instant::now()));
                }
                Pace::Wait => thread::sleep(WAIT_POLL_INTERVAL),
            }
        }
    }

    #[inline]
    pub(crate) fn events(&mut self) -> Result<()> {
        let size = terminal_size(&self.output)?;
        if self.canvas.size() != size {
            self.canvas.resize(size);
            self.writer.resize(size);
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn paint(&mut self, painter: &mut impl Painter) -> Pace {
        painter.paint(&mut self.canvas)
    }

    #[inline]
    pub(crate) fn present(&mut self) {
        for delta in self.canvas.delta() {
            self.writer.emit_move(delta.position());
            self.writer.emit_foreground(delta.cell().foreground());
            self.writer.emit_background(delta.cell().background());
            self.writer.emit_glyph_u8(delta.cell().glyph());
        }
    }

    #[inline]
    pub(crate) fn flush(&mut self) -> Result<()> {
        if let Err(error) = self.output.write_all(self.writer.written()) {
            self.writer.forget();
            self.writer.clear();
            return Err(error);
        }

        self.writer.clear();
        self.canvas.clear_delta();
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.writer.end();
        let _ = self.output.write_all(self.writer.written());
    }
}
