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
}
