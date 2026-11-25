use crate::common::{color::Color, digits::Digits};

#[derive(Clone, Copy)]
pub enum Style {
    Foreground(Color),
    Background(Color),
}

impl Style {
    #[inline]
    pub fn encode(self, buffer: &mut Vec<u8>) {
        match self {
            Self::Foreground(color) => {
                let r = Digits::lookup_u8(color.r());
                let g = Digits::lookup_u8(color.g());
                let b = Digits::lookup_u8(color.b());

                buffer.extend_from_slice(b"\x1b[38;2;");
                buffer.extend_from_slice(r.ascii());
                buffer.push(b';');
                buffer.extend_from_slice(g.ascii());
                buffer.push(b';');
                buffer.extend_from_slice(b.ascii());
                buffer.push(b'm');
            }
            Self::Background(color) => {
                let r = Digits::lookup_u8(color.r());
                let g = Digits::lookup_u8(color.g());
                let b = Digits::lookup_u8(color.b());

                buffer.extend_from_slice(b"\x1b[48;2;");
                buffer.extend_from_slice(r.ascii());
                buffer.push(b';');
                buffer.extend_from_slice(g.ascii());
                buffer.push(b';');
                buffer.extend_from_slice(b.ascii());
                buffer.push(b'm');
            }
        }
    }
}
