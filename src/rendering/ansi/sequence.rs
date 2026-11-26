use crate::common::{color::Color, digits::Digits, position::Position};

trait Sink {
    fn bytes(&mut self, bytes: &[u8]) -> usize;
    fn digits<const C: usize>(&mut self, digits: Digits<C>) -> usize;
    fn byte(&mut self, byte: u8) -> usize;
}

impl Sink for Vec<u8> {
    #[inline]
    fn bytes(&mut self, bytes: &[u8]) -> usize {
        self.extend_from_slice(bytes);
        bytes.len()
    }

    #[inline]
    fn digits<const C: usize>(&mut self, digits: Digits<C>) -> usize {
        let bytes = digits.ascii();
        self.extend_from_slice(bytes);
        bytes.len()
    }

    #[inline]
    fn byte(&mut self, byte: u8) -> usize {
        self.push(byte);
        1
    }
}

struct Counter;
impl Sink for Counter {
    #[inline]
    fn bytes(&mut self, bytes: &[u8]) -> usize {
        bytes.len()
    }

    #[inline]
    fn digits<const C: usize>(&mut self, digits: Digits<C>) -> usize {
        digits.count() as usize
    }

    #[inline]
    fn byte(&mut self, _byte: u8) -> usize {
        1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sequence {
    Absolute(Position),
    Column(u16),
    Row(u16),
    Up(u16),
    Down(u16),
    Right(u16),
    Left(u16),

    Foreground(Color),
    Background(Color),

    Alternate(bool),
    Cursor(bool),
    Clear,
    Reset,
}

impl Sequence {
    #[inline]
    pub fn encode(self, buffer: &mut Vec<u8>) {
        self.encode_with(buffer);
    }

    #[inline]
    pub fn length(self) -> usize {
        self.encode_with(&mut Counter)
    }

    #[inline]
    fn encode_with(self, sink: &mut impl Sink) -> usize {
        let mut written = 0;
        match self {
            Self::Absolute(Position::ZERO) => written += sink.bytes(b"\x1b[H"),
            Self::Absolute(position) => {
                let row = Digits::from_u16(position.y().saturating_add(1));
                let col = Digits::from_u16(position.x().saturating_add(1));

                written += sink.bytes(b"\x1b[");
                written += sink.digits(row);
                written += sink.byte(b';');
                written += sink.digits(col);
                written += sink.byte(b'H');
            }

            Self::Column(0) => written += sink.bytes(b"\x1b[G"),
            Self::Column(col) => {
                let digits = Digits::from_u16(col.saturating_add(1));
                written += sink.bytes(b"\x1b[");
                written += sink.digits(digits);
                written += sink.byte(b'G');
            }

            Self::Row(0) => written += sink.bytes(b"\x1b[d"),
            Self::Row(row) => {
                let digits = Digits::from_u16(row.saturating_add(1));
                written += sink.bytes(b"\x1b[");
                written += sink.digits(digits);
                written += sink.byte(b'd');
            }

            Self::Up(0) | Self::Up(1) => written += sink.bytes(b"\x1b[A"),
            Self::Up(count) => {
                let digits = Digits::from_u16(count);
                written += sink.bytes(b"\x1b[");
                written += sink.digits(digits);
                written += sink.byte(b'A');
            }

            Self::Down(0) | Self::Down(1) => written += sink.bytes(b"\x1b[B"),
            Self::Down(count) => {
                let digits = Digits::from_u16(count);
                written += sink.bytes(b"\x1b[");
                written += sink.digits(digits);
                written += sink.byte(b'B');
            }

            Self::Right(0) | Self::Right(1) => written += sink.bytes(b"\x1b[C"),
            Self::Right(count) => {
                let digits = Digits::from_u16(count);
                written += sink.bytes(b"\x1b[");
                written += sink.digits(digits);
                written += sink.byte(b'C');
            }

            Self::Left(0) | Self::Left(1) => written += sink.bytes(b"\x1b[D"),
            Self::Left(count) => {
                let digits = Digits::from_u16(count);
                written += sink.bytes(b"\x1b[");
                written += sink.digits(digits);
                written += sink.byte(b'D');
            }

            Self::Foreground(color) => {
                let r = Digits::lookup_u8(color.r());
                let g = Digits::lookup_u8(color.g());
                let b = Digits::lookup_u8(color.b());

                written += sink.bytes(b"\x1b[38;2;");
                written += sink.digits(r);
                written += sink.byte(b';');
                written += sink.digits(g);
                written += sink.byte(b';');
                written += sink.digits(b);
                written += sink.byte(b'm');
            }

            Self::Background(color) => {
                let r = Digits::lookup_u8(color.r());
                let g = Digits::lookup_u8(color.g());
                let b = Digits::lookup_u8(color.b());

                written += sink.bytes(b"\x1b[48;2;");
                written += sink.digits(r);
                written += sink.byte(b';');
                written += sink.digits(g);
                written += sink.byte(b';');
                written += sink.digits(b);
                written += sink.byte(b'm');
            }

            Self::Alternate(true) => written += sink.bytes(b"\x1b[?1049h"),
            Self::Alternate(false) => written += sink.bytes(b"\x1b[?1049l"),
            Self::Cursor(true) => written += sink.bytes(b"\x1b[?25h"),
            Self::Cursor(false) => written += sink.bytes(b"\x1b[?25l"),
            Self::Clear => written += sink.bytes(b"\x1b[2J"),
            Self::Reset => written += sink.bytes(b"\x1b[0m"),
        }

        written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(sequence: Sequence) -> Vec<u8> {
        let mut buffer = Vec::new();
        sequence.encode(&mut buffer);
        buffer
    }

    fn samples() -> Vec<Sequence> {
        let mut sequences = Vec::new();

        for position in [
            Position::ZERO,
            Position::new(1, 1),
            Position::new(9, 9),
            Position::new(10, 10),
            Position::new(99, 99),
            Position::new(100, 100),
            Position::new(999, 999),
            Position::new(9999, 9999),
            Position::new(u16::MAX, u16::MAX),
            Position::new(0, 5),
            Position::new(5, 0),
        ] {
            sequences.push(Sequence::Absolute(position));
        }

        for value in [
            0,
            1,
            2,
            9,
            10,
            11,
            99,
            100,
            999,
            1000,
            9999,
            10_000,
            u16::MAX,
        ] {
            sequences.push(Sequence::Column(value));
            sequences.push(Sequence::Row(value));
            sequences.push(Sequence::Up(value));
            sequences.push(Sequence::Down(value));
            sequences.push(Sequence::Right(value));
            sequences.push(Sequence::Left(value));
        }

        for color in [
            Color::BLACK,
            Color::new(1, 20, 255),
            Color::new(0, 128, 7),
            Color::new(9, 10, 99),
            Color::new(100, 199, 200),
            Color::new(255, 255, 255),
        ] {
            sequences.push(Sequence::Foreground(color));
            sequences.push(Sequence::Background(color));
        }

        sequences.extend([
            Sequence::Alternate(true),
            Sequence::Alternate(false),
            Sequence::Cursor(true),
            Sequence::Cursor(false),
            Sequence::Clear,
            Sequence::Reset,
        ]);

        sequences
    }

    #[test]
    fn length_matches_encoded_length() {
        for sequence in samples() {
            let bytes = encoded(sequence);

            assert_eq!(sequence.length(), bytes.len(), "{sequence:?} -> {bytes:?}");
        }
    }

    #[test]
    fn static_sequences_have_expected_bytes() {
        assert_eq!(encoded(Sequence::Alternate(true)), b"\x1b[?1049h");
        assert_eq!(encoded(Sequence::Alternate(false)), b"\x1b[?1049l");
        assert_eq!(encoded(Sequence::Cursor(true)), b"\x1b[?25h");
        assert_eq!(encoded(Sequence::Cursor(false)), b"\x1b[?25l");
        assert_eq!(encoded(Sequence::Clear), b"\x1b[2J");
        assert_eq!(encoded(Sequence::Reset), b"\x1b[0m");
    }

    #[test]
    fn absolute_uses_one_based_minimal_coordinates() {
        assert_eq!(encoded(Sequence::Absolute(Position::ZERO)), b"\x1b[H");
        assert_eq!(encoded(Sequence::Absolute(Position::new(0, 0))), b"\x1b[H");
        assert_eq!(
            encoded(Sequence::Absolute(Position::new(4, 2))),
            b"\x1b[3;5H"
        );
        assert_eq!(
            encoded(Sequence::Absolute(Position::new(u16::MAX, u16::MAX))),
            b"\x1b[65535;65535H"
        );
    }

    #[test]
    fn column_and_row_use_one_based_minimal_coordinates() {
        assert_eq!(encoded(Sequence::Column(0)), b"\x1b[G");
        assert_eq!(encoded(Sequence::Column(4)), b"\x1b[5G");
        assert_eq!(encoded(Sequence::Row(0)), b"\x1b[d");
        assert_eq!(encoded(Sequence::Row(4)), b"\x1b[5d");
    }

    #[test]
    fn relative_counts_zero_and_one_are_elided() {
        for sequence in [
            Sequence::Up(0),
            Sequence::Up(1),
            Sequence::Down(0),
            Sequence::Down(1),
            Sequence::Right(0),
            Sequence::Right(1),
            Sequence::Left(0),
            Sequence::Left(1),
        ] {
            assert_eq!(encoded(sequence).len(), 3, "{sequence:?}");
            assert_eq!(sequence.length(), 3, "{sequence:?}");
        }

        assert_eq!(encoded(Sequence::Up(2)), b"\x1b[2A");
        assert_eq!(encoded(Sequence::Down(2)), b"\x1b[2B");
        assert_eq!(encoded(Sequence::Right(2)), b"\x1b[2C");
        assert_eq!(encoded(Sequence::Left(2)), b"\x1b[2D");
    }

    #[test]
    fn colors_use_minimal_truecolor_digits() {
        assert_eq!(
            encoded(Sequence::Foreground(Color::new(1, 20, 255))),
            b"\x1b[38;2;1;20;255m"
        );
        assert_eq!(
            encoded(Sequence::Background(Color::new(0, 128, 7))),
            b"\x1b[48;2;0;128;7m"
        );
    }
}
