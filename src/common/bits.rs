pub const BITS_PER_WORD: usize = u64::BITS as usize;

#[inline]
pub const fn word_of(bit: usize) -> usize {
    bit / BITS_PER_WORD
}

#[inline]
pub const fn offset_of(bit: usize) -> usize {
    bit % BITS_PER_WORD
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Word(u64);

impl Word {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(u64::MAX);

    #[inline]
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    #[inline]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[inline]
    pub const fn any(self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub const fn from_bit(bit: usize) -> Self {
        Self(1u64 << bit)
    }

    #[inline]
    pub const fn contains(self, bit: usize) -> bool {
        self.0 & (1u64 << bit) != 0
    }

    #[inline]
    pub const fn insert(&mut self, bit: usize) {
        self.0 = self.0 | (1u64 << bit);
    }

    #[inline]
    pub const fn remove(&mut self, bit: usize) {
        self.0 = self.0 & !(1u64 << bit);
    }

    #[inline]
    pub const fn count(self) -> usize {
        self.0.count_ones() as usize
    }

    #[inline]
    pub const fn lowest(self) -> Option<usize> {
        if self.0 == 0 {
            None
        } else {
            Some(self.0.trailing_zeros() as usize)
        }
    }

    #[inline]
    pub const fn without_lowest(self) -> Self {
        if self.0 == 0 {
            Self::EMPTY
        } else {
            Self(self.0 & (self.0 - 1))
        }
    }

    #[inline]
    pub fn iter_set_bits(self, base: usize) -> impl Iterator<Item = usize> {
        let mut remaining = self;

        std::iter::from_fn(move || {
            let bit = remaining.lowest()?;
            remaining = remaining.without_lowest();
            Some(base + bit)
        })
    }

    #[inline]
    pub const fn mask_below(bit: usize) -> Self {
        if bit >= BITS_PER_WORD {
            Self::FULL
        } else {
            Self((1u64 << bit) - 1)
        }
    }

    #[inline]
    pub const fn mask_through(bit: usize) -> Self {
        Self::mask_below(bit.saturating_add(1))
    }

    #[inline]
    pub const fn mask_from(bit: usize) -> Self {
        if bit >= BITS_PER_WORD {
            Self::EMPTY
        } else {
            Self(u64::MAX << bit)
        }
    }

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline]
    pub const fn difference(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    #[inline]
    pub const fn complement(self) -> Self {
        Self(!self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_bit_sets_a_single_bit() {
        assert_eq!(Word::from_bit(0).bits(), 1);
        assert_eq!(Word::from_bit(63).bits(), 1u64 << 63);
    }

    #[test]
    fn contains_matches_inserted_bit() {
        let mut word = Word::EMPTY;

        assert!(!word.contains(5));

        word.insert(5);

        assert!(word.contains(5));
        assert!(!word.contains(4));
    }

    #[test]
    fn remove_clears_the_bit() {
        let mut word = Word::from_bit(9);

        word.remove(9);

        assert_eq!(word, Word::EMPTY);
    }

    #[test]
    fn insert_and_remove_work_in_const_context() {
        const WORD: Word = {
            let mut word = Word::EMPTY;
            word.insert(3);
            word.remove(1);
            word
        };

        assert_eq!(WORD, Word::from_bit(3));
    }

    #[test]
    fn empty_and_full_report_correctly() {
        assert!(Word::EMPTY.is_empty());
        assert!(!Word::EMPTY.any());
        assert!(Word::FULL.any());
        assert!(!Word::FULL.is_empty());
    }

    #[test]
    fn count_ones_counts_set_bits() {
        assert_eq!(Word::EMPTY.count(), 0);
        assert_eq!(Word::FULL.count(), BITS_PER_WORD);
        assert_eq!(Word::from_bits(0b1011).count(), 3);
    }

    #[test]
    fn lowest_reports_the_first_set_bit() {
        assert_eq!(Word::EMPTY.lowest(), None);
        assert_eq!(Word::from_bit(0).lowest(), Some(0));
        assert_eq!(Word::from_bits(0b1010).lowest(), Some(1));
    }

    #[test]
    fn without_lowest_clears_the_first_set_bit() {
        assert_eq!(Word::EMPTY.without_lowest(), Word::EMPTY);
        assert_eq!(Word::from_bit(0).without_lowest(), Word::EMPTY);
        assert_eq!(
            Word::from_bits(0b1010).without_lowest(),
            Word::from_bits(0b1000)
        );
    }

    #[test]
    fn iter_set_bits_is_ascending_and_offset_by_base() {
        let word = Word::from_bits(0b1010_0100);

        let bits: Vec<_> = word.iter_set_bits(64).collect();

        assert_eq!(bits, [66, 69, 71]);
    }

    #[test]
    fn iter_set_bits_on_empty_is_empty() {
        assert_eq!(Word::EMPTY.iter_set_bits(0).count(), 0);
    }

    #[test]
    fn mask_below_keeps_low_bits() {
        assert_eq!(Word::mask_below(0), Word::EMPTY);
        assert_eq!(Word::mask_below(1), Word::from_bits(0b1));
        assert_eq!(Word::mask_below(4), Word::from_bits(0b1111));
        assert_eq!(Word::mask_below(BITS_PER_WORD), Word::FULL);
        assert_eq!(Word::mask_below(BITS_PER_WORD + 10), Word::FULL);
    }

    #[test]
    fn mask_through_includes_the_bit() {
        assert_eq!(Word::mask_through(0), Word::from_bits(0b1));
        assert_eq!(Word::mask_through(3), Word::from_bits(0b1111));
        assert_eq!(Word::mask_through(BITS_PER_WORD - 1), Word::FULL);
    }

    #[test]
    fn mask_from_keeps_high_bits() {
        assert_eq!(Word::mask_from(0), Word::FULL);
        assert_eq!(
            Word::mask_from(BITS_PER_WORD - 1),
            Word::from_bits(1u64 << 63)
        );
        assert_eq!(Word::mask_from(BITS_PER_WORD), Word::EMPTY);
    }

    #[test]
    fn set_algebra_combines_words() {
        let a = Word::from_bits(0b1100);
        let b = Word::from_bits(0b1010);

        assert_eq!(a.union(b), Word::from_bits(0b1110));
        assert_eq!(a.intersection(b), Word::from_bits(0b1000));
        assert_eq!(a.difference(b), Word::from_bits(0b0100));
        assert_eq!(a.complement().intersection(a), Word::EMPTY);
    }

    #[test]
    fn word_and_offset_split_a_bit_index() {
        assert_eq!((word_of(0), offset_of(0)), (0, 0));
        assert_eq!((word_of(63), offset_of(63)), (0, 63));
        assert_eq!((word_of(64), offset_of(64)), (1, 0));
        assert_eq!((word_of(130), offset_of(130)), (2, 2));
    }
}
