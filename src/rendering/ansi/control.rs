#[derive(Clone, Copy)]
pub enum Control {
    Alternate(bool),
    Cursor(bool),
    Clear,
    Reset,
}

impl Control {
    #[inline]
    pub fn encode(self, buffer: &mut Vec<u8>) {
        match self {
            Self::Alternate(true) => buffer.extend_from_slice(b"\x1b[?1049h"),
            Self::Alternate(false) => buffer.extend_from_slice(b"\x1b[?1049l"),
            Self::Cursor(true) => buffer.extend_from_slice(b"\x1b[?25h"),
            Self::Cursor(false) => buffer.extend_from_slice(b"\x1b[?25l"),
            Self::Clear => buffer.extend_from_slice(b"\x1b[2J"),
            Self::Reset => buffer.extend_from_slice(b"\x1b[0m"),
        }
    }
}
