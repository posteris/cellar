use crate::rendering::ansi::encoder::Encoder;
use crate::rendering::ansi::platform::{raw_stdout, terminal_size};
use crate::rendering::ansi::state::State;
use crate::rendering::frame::Frame;
use crate::rendering::renderer::Renderer;
use std::fs::File;
use std::io::{Result, Write};

pub struct Terminal {
    frame: Frame,
    state: State,
    encoder: Encoder,
    output: File,
}

impl Terminal {
    pub fn new() -> Result<Self> {
        let output = raw_stdout()?;
        let size = terminal_size(&output)?;
        let encoder = Encoder::new(size.area() as usize * 64);
        let state = State::new();

        let frame = Frame::new(size);

        let mut terminal = Self {
            frame,
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
    #[rustfmt::skip]
    fn render(&mut self, mut paint: impl FnMut(&mut Frame)) -> Result<()> {
        let size = terminal_size(&self.output)?;

        if self.frame.size() != size {
            self.frame.resize(size);
            self.state.forget_cursor();
            self.state.forget_foreground();
            self.state.forget_background();
            self.encoder.encode_reset();
            self.encoder.encode_clear();
        }

        paint(&mut self.frame);

        for delta in self.frame.dirty() {
            self.state.ensure_cursor(delta.position(), |p| self.encoder.encode_move(p));
            self.state.ensure_foreground(delta.cell().foreground(), |c| self.encoder.encode_foreground(c));
            self.state.ensure_background(delta.cell().background(), |c| self.encoder.encode_background(c));
            self.encoder.encode_glyph_u8(delta.cell().glyph());
            self.state.advance_cursor(size);
        }

        if let Err(error) = self.output.write_all(self.encoder.written()) {
            self.state.forget_cursor();
            self.state.forget_foreground();
            self.state.forget_background();
            self.encoder.clear();
            return Err(error);
        }

        self.encoder.clear();
        self.frame.clear_dirty();
        Ok(())
    }
}
