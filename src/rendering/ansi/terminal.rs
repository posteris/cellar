use crate::rendering::ansi::encoder::Encoder;
use crate::rendering::ansi::platform::{raw_stdout, terminal_size};
use crate::rendering::ansi::state::State;
use crate::rendering::frame::Frame;
use crate::rendering::renderer::Renderer;
use std::fs::File;
use std::io::{Result, Write};

pub struct Terminal {
    current: Frame,
    previous: Frame,

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

        let current = Frame::new(size);
        let previous = Frame::new(size);

        let mut terminal = Self {
            current,
            previous,
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
    fn render(&mut self, mut paint: impl FnMut(&mut Frame)) -> Result<()> {
        let Self {
            current,
            previous,
            state,
            encoder,
            output,
        } = self;

        let current_size = terminal_size(&output)?;
        current.resize(current_size);

        paint(&mut self.current);

        for delta in self.current.diff(&previous) {
            state.ensure_cursor(delta.position(), |p| encoder.encode_move(p));
            state.ensure_foreground(delta.cell().foreground(), |c| encoder.encode_foreground(c));
            state.ensure_background(delta.cell().background(), |c| encoder.encode_background(c));
            encoder.encode_glyph_u8(delta.cell().glyph());
            state.advance_cursor(current_size);
        }

        output.write_all(encoder.written())?;
        encoder.clear();
        Ok(())
    }
}
