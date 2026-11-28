use bytemuck::{Pod, Zeroable};

use crate::common::color::Color;

// GPU-friendly layout: two u32s per cell.
// - `foreground`: RGB in the low 24 bits, glyph in the high byte.
// - `background`: RGB in the low 24 bits.
//
// This means `&[Cell]` can be uploaded to a storage buffer verbatim; the shader
// unpacks the bytes it needs with shifts.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    foreground: u32,
    background: u32,
}

unsafe impl Zeroable for Cell {}
unsafe impl Pod for Cell {}

impl Cell {
    pub const EMPTY: Self = Self::new(Color::BLACK, Color::BLACK, b' ');

    #[inline]
    pub const fn new(foreground: Color, background: Color, glyph: u8) -> Self {
        Self {
            foreground: foreground.pack(glyph),
            background: background.pack(0),
        }
    }

    #[inline]
    pub const fn solid(color: Color) -> Self {
        Self::new(Color::BLACK, color, b' ')
    }

    #[inline]
    pub const fn foreground(&self) -> Color {
        Color::from_packed(self.foreground)
    }

    #[inline]
    pub const fn background(&self) -> Color {
        Color::from_packed(self.background)
    }

    #[inline]
    pub const fn glyph(&self) -> u8 {
        (self.foreground >> 24) as u8
    }
}
