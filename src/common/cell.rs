use crate::common::color::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    foreground: Color,
    background: Color,
    glyph: u8,
}

impl Cell {
    pub const EMPTY: Self = Self::new(Color::BLACK, Color::BLACK, b' ');

    #[inline]
    pub const fn new(foreground: Color, background: Color, glyph: u8) -> Self {
        Self {
            foreground,
            background,
            glyph,
        }
    }

    #[inline]
    pub const fn solid(color: Color) -> Self {
        Self {
            foreground: Color::BLACK,
            background: color,
            glyph: b' ',
        }
    }

    #[inline]
    pub const fn foreground(&self) -> Color {
        self.foreground
    }

    #[inline]
    pub const fn background(&self) -> Color {
        self.background
    }

    #[inline]
    pub const fn glyph(&self) -> u8 {
        self.glyph
    }
}
