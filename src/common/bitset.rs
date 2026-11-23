use crate::common::bits::{BITS_PER_WORD, Word, offset_of, word_of};

#[inline]
fn word_count(len: usize) -> usize {
    len.div_ceil(BITS_PER_WORD)
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Bitset {
    words: Vec<Word>,
    len: usize,
}

impl Bitset {
    #[inline]
    pub fn new(len: usize) -> Self {
        let words = vec![Word::EMPTY; word_count(len)];
        Self { words, len }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|word| word.is_empty())
    }

    #[inline]
    pub fn count_ones(&self) -> usize {
        self.words.iter().map(|word| word.count()).sum()
    }

    #[inline]
    pub fn contains(&self, index: usize) -> bool {
        index < self.len && self.words[word_of(index)].contains(offset_of(index))
    }

    #[inline]
    pub fn set_bit(&mut self, index: usize) -> bool {
        if index >= self.len {
            return false;
        }

        self.words[word_of(index)].insert(offset_of(index));
        true
    }

    #[inline]
    pub fn clear_bit(&mut self, index: usize) -> bool {
        if index >= self.len {
            return false;
        }

        self.words[word_of(index)].remove(offset_of(index));
        true
    }

    #[inline]
    pub fn set_range(&mut self, start: usize, end: usize) {
        let end = end.min(self.len);
        if start >= end {
            return;
        }

        let first = word_of(start);
        let last = word_of(end - 1);

        if first == last {
            let mask = Word::mask_from(offset_of(start))
                .intersection(Word::mask_through(offset_of(end - 1)));
            self.words[first] = self.words[first].union(mask);
            return;
        }

        self.words[first] = self.words[first].union(Word::mask_from(offset_of(start)));
        self.words[first + 1..last].fill(Word::FULL);
        self.words[last] = self.words[last].union(Word::mask_through(offset_of(end - 1)));
    }

    #[inline]
    pub fn clear_range(&mut self, start: usize, end: usize) {
        let end = end.min(self.len);
        if start >= end {
            return;
        }

        let first = word_of(start);
        let last = word_of(end - 1);

        if first == last {
            let mask = Word::mask_from(offset_of(start))
                .intersection(Word::mask_through(offset_of(end - 1)));
            self.words[first] = self.words[first].difference(mask);
            return;
        }

        self.words[first] = self.words[first].difference(Word::mask_from(offset_of(start)));
        self.words[first + 1..last].fill(Word::EMPTY);
        self.words[last] = self.words[last].difference(Word::mask_through(offset_of(end - 1)));
    }

    #[inline]
    pub fn set_all(&mut self) {
        self.set_range(0, self.len);
    }

    #[inline]
    pub fn clear_all(&mut self) {
        self.words.fill(Word::EMPTY);
    }

    #[inline]
    pub fn resize(&mut self, len: usize) {
        self.words.resize(word_count(len), Word::EMPTY);
        self.len = len;
        self.mask_last_word();
    }

    #[inline]
    pub fn iter_ones(&self) -> impl Iterator<Item = usize> + '_ {
        self.words
            .iter()
            .enumerate()
            .flat_map(|(index, &word)| word.iter_set_bits(index * BITS_PER_WORD))
    }

    #[inline]
    pub fn iter_ones_range(&self, start: usize, end: usize) -> impl Iterator<Item = usize> + '_ {
        let end = end.min(self.len);
        let start = start.min(end);

        let (first, last, words): (usize, usize, &[Word]) = if start >= end {
            (0, 0, &[])
        } else {
            let first = word_of(start);
            let last = word_of(end - 1);
            (first, last, &self.words[first..=last])
        };

        let last_bit = end.saturating_sub(1);

        words.iter().enumerate().flat_map(move |(index, &word)| {
            let word_index = first + index;
            let mut word = word;

            if word_index == first {
                word = word.difference(Word::mask_below(offset_of(start)));
            }

            if word_index == last {
                word = word.intersection(Word::mask_through(offset_of(last_bit)));
            }

            word.iter_set_bits(word_index * BITS_PER_WORD)
        })
    }

    #[inline]
    pub fn union_with(&mut self, other: &Self) {
        debug_assert_eq!(self.len, other.len);

        for (word, other) in self.words.iter_mut().zip(&other.words) {
            *word = word.union(*other);
        }
    }

    #[inline]
    pub fn intersection_with(&mut self, other: &Self) {
        debug_assert_eq!(self.len, other.len);

        for (word, other) in self.words.iter_mut().zip(&other.words) {
            *word = word.intersection(*other);
        }
    }

    #[inline]
    pub fn difference_with(&mut self, other: &Self) {
        debug_assert_eq!(self.len, other.len);

        for (word, other) in self.words.iter_mut().zip(&other.words) {
            *word = word.difference(*other);
        }
    }

    #[inline]
    pub fn is_subset_of(&self, other: &Self) -> bool {
        debug_assert_eq!(self.len, other.len);

        self.words
            .iter()
            .zip(&other.words)
            .all(|(&word, &other)| word.difference(other).is_empty())
    }

    #[inline]
    fn mask_last_word(&mut self) {
        let Some(word) = self.words.last_mut() else {
            return;
        };

        let valid = offset_of(self.len);
        if valid != 0 {
            *word = word.intersection(Word::mask_below(valid));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with(indices: &[usize], len: usize) -> Bitset {
        let mut bitset = Bitset::new(len);

        for &index in indices {
            bitset.set_bit(index);
        }

        bitset
    }

    fn ones(bitset: &Bitset) -> Vec<usize> {
        bitset.iter_ones().collect()
    }

    #[test]
    fn new_starts_empty() {
        let bitset = Bitset::new(10);

        assert_eq!(bitset.len(), 10);
        assert!(bitset.is_empty());
        assert_eq!(bitset.count_ones(), 0);
    }

    #[test]
    fn set_and_contains_a_bit() {
        let mut bitset = Bitset::new(10);

        assert!(!bitset.contains(3));
        assert!(bitset.set_bit(3));
        assert!(bitset.contains(3));
        assert!(bitset.set_bit(3));
        assert_eq!(bitset.count_ones(), 1);
    }

    #[test]
    fn out_of_range_operations_are_rejected() {
        let mut bitset = Bitset::new(4);

        assert!(!bitset.set_bit(4));
        assert!(!bitset.clear_bit(4));
        assert!(!bitset.contains(4));
        assert!(bitset.is_empty());
    }

    #[test]
    fn clear_bit_removes_a_bit() {
        let mut bitset = with(&[9], 10);

        assert!(bitset.clear_bit(9));
        assert!(!bitset.contains(9));
        assert!(bitset.is_empty());
    }

    #[test]
    fn set_range_sets_a_span() {
        let mut bitset = Bitset::new(10);

        bitset.set_range(2, 6);

        assert_eq!(ones(&bitset), [2, 3, 4, 5]);
    }

    #[test]
    fn set_range_clamps_to_len() {
        let mut bitset = Bitset::new(10);

        bitset.set_range(8, 100);

        assert_eq!(ones(&bitset), [8, 9]);
    }

    #[test]
    fn set_range_across_word_boundaries() {
        let mut bitset = Bitset::new(130);

        bitset.set_range(60, 70);

        assert_eq!(ones(&bitset), (60..70).collect::<Vec<_>>());
    }

    #[test]
    fn set_range_covering_whole_words() {
        let mut bitset = Bitset::new(128);

        bitset.set_range(0, 64);

        assert_eq!(ones(&bitset), (0..64).collect::<Vec<_>>());
    }

    #[test]
    fn empty_range_does_nothing() {
        let mut bitset = Bitset::new(10);

        bitset.set_range(6, 6);
        bitset.set_range(7, 3);

        assert!(bitset.is_empty());
    }

    #[test]
    fn clear_range_clears_a_span() {
        let mut bitset = Bitset::new(10);
        bitset.set_all();

        bitset.clear_range(2, 6);

        assert_eq!(ones(&bitset), [0, 1, 6, 7, 8, 9]);
    }

    #[test]
    fn clear_range_across_word_boundaries() {
        let mut bitset = Bitset::new(130);
        bitset.set_all();

        bitset.clear_range(60, 70);

        assert_eq!(bitset.count_ones(), 120);
        assert!(!bitset.contains(60));
        assert!(!bitset.contains(69));
        assert!(bitset.contains(59));
        assert!(bitset.contains(70));
    }

    #[test]
    fn set_all_sets_every_index() {
        let mut bitset = Bitset::new(130);

        bitset.set_all();

        assert_eq!(bitset.count_ones(), 130);
        assert_eq!(ones(&bitset), (0..130).collect::<Vec<_>>());
    }

    #[test]
    fn clear_removes_every_index() {
        let mut bitset = Bitset::new(130);
        bitset.set_all();

        bitset.clear_all();

        assert!(bitset.is_empty());
        assert_eq!(bitset.count_ones(), 0);
    }

    #[test]
    fn iter_ones_is_ascending() {
        let bitset = with(&[64, 5, 1, 63], 130);

        assert_eq!(ones(&bitset), [1, 5, 63, 64]);
    }

    #[test]
    fn iter_ones_range_respects_bounds() {
        let mut bitset = Bitset::new(130);
        bitset.set_all();

        assert_eq!(
            bitset.iter_ones_range(60, 70).collect::<Vec<_>>(),
            (60..70).collect::<Vec<_>>()
        );
        assert_eq!(
            bitset.iter_ones_range(1, 5).collect::<Vec<_>>(),
            (1..5).collect::<Vec<_>>()
        );
        assert_eq!(
            bitset.iter_ones_range(125, 500).collect::<Vec<_>>(),
            (125..130).collect::<Vec<_>>()
        );
    }

    #[test]
    fn iter_ones_range_is_empty_for_empty_ranges() {
        let mut bitset = Bitset::new(10);
        bitset.set_all();

        assert_eq!(bitset.iter_ones_range(4, 4).count(), 0);
        assert_eq!(bitset.iter_ones_range(7, 3).count(), 0);
        assert_eq!(bitset.iter_ones_range(10, 20).count(), 0);
    }

    #[test]
    fn set_algebra_combines_bitsets() {
        let mut a = with(&[1, 3], 10);
        let b = with(&[3, 4], 10);

        let mut union = a.clone();
        union.union_with(&b);
        assert_eq!(ones(&union), [1, 3, 4]);

        let mut intersection = a.clone();
        intersection.intersection_with(&b);
        assert_eq!(ones(&intersection), [3]);

        a.difference_with(&b);
        assert_eq!(ones(&a), [1]);
    }

    #[test]
    fn subset_is_detected() {
        let subset = with(&[1, 3], 10);
        let superset = with(&[1, 2, 3], 10);

        assert!(subset.is_subset_of(&superset));
        assert!(!superset.is_subset_of(&subset));
    }

    #[test]
    fn resize_grows_and_keeps_bits() {
        let mut bitset = with(&[4], 5);

        bitset.resize(10);

        assert_eq!(bitset.len(), 10);
        assert!(bitset.contains(4));
        assert_eq!(bitset.count_ones(), 1);
    }

    #[test]
    fn resize_shrinks_and_drops_out_of_range_bits() {
        let mut bitset = Bitset::new(130);
        bitset.set_all();

        bitset.resize(100);

        assert_eq!(bitset.len(), 100);
        assert_eq!(bitset.count_ones(), 100);
        assert_eq!(ones(&bitset), (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn resize_to_zero_empties_the_bitset() {
        let mut bitset = Bitset::new(10);
        bitset.set_all();

        bitset.resize(0);

        assert_eq!(bitset.len(), 0);
        assert!(bitset.is_empty());
        assert_eq!(ones(&bitset), []);
    }
}
