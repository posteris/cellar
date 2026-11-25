use crate::rendering::ansi::platform::{raw_stdout, terminal_size};
use crate::rendering::ansi::writer::Writer;
use crate::rendering::frame::Frame;
use crate::rendering::renderer::Renderer;
use std::fs::File;
use std::io::{Result, Write};

pub struct Terminal {
    frame: Frame,
    writer: Writer,
    output: File,
}

impl Terminal {
    #[inline]
    pub fn new() -> Result<Self> {
        let mut output = raw_stdout()?;
        let size = terminal_size(&output)?;

        let frame = Frame::new(size);
        let mut writer = Writer::new(size);
        writer.begin();

        output.write_all(writer.written())?;
        writer.clear();

        Ok(Self {
            frame,
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
    #[inline]
    fn render(&mut self, mut paint: impl FnMut(&mut Frame)) -> Result<()> {
        let size = terminal_size(&self.output)?;

        if self.frame.size() != size {
            self.frame.resize(size);
            self.writer.resize(size);
        }

        paint(&mut self.frame);

        for delta in self.frame.dirty() {
            self.writer.emit_move(delta.position());
            self.writer.emit_foreground(delta.cell().foreground());
            self.writer.emit_background(delta.cell().background());
            self.writer.emit_glyph_u8(delta.cell().glyph());
        }

        if let Err(error) = self.output.write_all(self.writer.written()) {
            self.writer.forget();
            self.writer.clear();
            return Err(error);
        }

        self.writer.clear();
        self.frame.clear_dirty();
        Ok(())
    }
}
