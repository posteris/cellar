use crate::common::position::Position;
use crate::common::size::Size;
use crate::rendering::ansi::encoder::Encoder;
use crate::rendering::ansi::platform::{raw_stdout, terminal_size};
use crate::rendering::frame::Frame;
use crate::rendering::renderer::Renderer;
use std::fs::File;
use std::io::{Result, Write};

pub struct Terminal {
    encoder: Encoder,
    output: File,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let output = raw_stdout()?;
        let size = terminal_size(&output)?;
        let encoder = Encoder::new(size.area() as usize * 48);

        let mut terminal = Self { encoder, output };

        terminal.encoder.encode_cursor_hide();
        terminal.encoder.encode_alternate_enter();
        terminal.encoder.encode_clear();
        terminal.output.write_all(terminal.encoder.written())?;

        Ok(terminal)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.encoder.clear(); // Discard half-built frame
        self.encoder.encode_reset();
        self.encoder.encode_cursor_show();
        self.encoder.encode_alternate_leave();
        let _ = self.output.write_all(self.encoder.written());
    }
}

impl Renderer for Terminal {
    #[inline]
    fn size(&self) -> Result<Size> {
        terminal_size(&self.output)
    }

    #[inline]
    fn present(&mut self, frame: &mut Frame) -> Result<()> {
        self.encoder.encode_move(Position::zero());
        for cell in frame.cells() {
            self.encoder.encode_foreground(cell.foreground);
            self.encoder.encode_background(cell.background);
            self.encoder.encode_glyph_u8(cell.glyph);
        }

        self.output.write_all(self.encoder.written())?;
        self.encoder.clear();
        Ok(())
    }
}
