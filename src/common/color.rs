#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color([u8; 3]);

impl Color {
    pub const BLACK: Self = Self::new(0, 0, 0);

    #[inline]
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b])
    }

    #[inline]
    pub const fn values(&self) -> [u8; 3] {
        self.0
    }

    #[inline]
    pub const fn r(&self) -> u8 {
        self.values()[0]
    }

    #[inline]
    pub const fn g(&self) -> u8 {
        self.values()[1]
    }

    #[inline]
    pub const fn b(&self) -> u8 {
        self.values()[2]
    }

    // RGB in the low 24 bits; `a` occupies the high byte.
    #[inline]
    pub(crate) const fn pack(self, a: u8) -> u32 {
        self.0[0] as u32 | (self.0[1] as u32) << 8 | (self.0[2] as u32) << 16 | (a as u32) << 24
    }

    #[inline]
    pub(crate) const fn from_packed(packed: u32) -> Self {
        Self([packed as u8, (packed >> 8) as u8, (packed >> 16) as u8])
    }
}
