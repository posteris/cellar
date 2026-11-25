#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digits<const C: usize> {
    ascii: [u8; C],
    count: u8,
}

impl<const C: usize> Digits<C> {
    #[inline]
    pub fn ascii(&self) -> &[u8] {
        &self.ascii[C - self.count as usize..]
    }

    #[inline]
    pub const fn count(&self) -> u8 {
        self.count
    }
}

impl Digits<3> {
    #[inline]
    pub const fn from_u8(value: u8) -> Self {
        let count = 1 + (value >= 10) as u8 + (value >= 100) as u8;
        let ascii = [
            b'0' + value / 100,
            b'0' + value / 10 % 10,
            b'0' + value % 10,
        ];

        Self { ascii, count }
    }

    #[inline]
    pub fn lookup_u8(value: u8) -> Self {
        static DIGITS3: [Digits<3>; 256] = Digits::<3>::create_lookup_table();
        DIGITS3[value as usize]
    }

    const fn create_lookup_table() -> [Self; 256] {
        let mut table = [Self::from_u8(0); 256];
        let mut i = 0;

        while i < 256 {
            table[i] = Self::from_u8(i as u8);
            i += 1;
        }

        table
    }
}

impl Digits<5> {
    #[inline]
    pub const fn from_u16(value: u16) -> Self {
        let count = 1
            + (value >= 10) as u8
            + (value >= 100) as u8
            + (value >= 1000) as u8
            + (value >= 10_000) as u8;
        let ascii = [
            b'0' + (value / 10000) as u8,
            b'0' + (value / 1000 % 10) as u8,
            b'0' + (value / 100 % 10) as u8,
            b'0' + (value / 10 % 10) as u8,
            b'0' + (value % 10) as u8,
        ];

        Self { ascii, count }
    }
}
