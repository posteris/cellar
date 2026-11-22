use crate::common::position::Position;
use crate::common::size::Size;
use crate::rendering::ansi::encoder::Encoder;
use crate::rendering::ansi::platform::{raw_stdout, terminal_size};
use crate::rendering::ansi::state::State;
use crate::rendering::frame::Frame;
use crate::rendering::renderer::Renderer;
use std::fs::File;
use std::io::{Result, Write};

pub struct Terminal {
    state: State,
    encoder: Encoder,
    output: File,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let output = raw_stdout()?;
        let size = terminal_size(&output)?;
        let encoder = Encoder::new(size.area() as usize * 48);
        let state = State::new();

        let mut terminal = Self {
            state,
            encoder,
            output,
        };

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
    #[rustfmt::skip]
    fn present(&mut self, frame: &Frame) -> Result<()> {
        for (y, row) in frame.indexed_rows() {
            self.encoder.encode_move(Position::new(0, y));
            for cell in row {
                self.state.ensure_foreground(cell.foreground(), |c| self.encoder.encode_foreground(c));
                self.state.ensure_background(cell.background(), |c| self.encoder.encode_background(c));
                self.encoder.encode_glyph_u8(cell.glyph());
            }
        }

        self.output.write_all(self.encoder.written())?;
        self.encoder.clear();
        Ok(())
    }
}
