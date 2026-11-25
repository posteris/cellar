use crate::common::{digits::Digits, position::Position};

#[derive(Clone, Copy)]
pub enum Motion {
    Absolute(Position),
    Column(u16),
    Row(u16),
    Up(u16),
    Down(u16),
    Right(u16),
    Left(u16),
}

impl Motion {
    #[inline]
    pub fn encode_shortest(buffer: &mut Vec<u8>, from: Option<Position>, to: Position) {
        let Some(from) = from else {
            Self::Absolute(to).encode(buffer);
            return;
        };

        if from == to {
            return;
        }

        let absolute = Self::Absolute(to);
        let vertical = Self::shortest_vertical(from.y(), to.y());
        let horizontal = Self::shortest_horizontal(from.x(), to.x());
        let relative_length = vertical.map_or(0, |motion| motion.length())
            + horizontal.map_or(0, |motion| motion.length());

        if relative_length < absolute.length() {
            if let Some(relative) = vertical {
                relative.encode(buffer);
            }
            if let Some(relative) = horizontal {
                relative.encode(buffer);
            }
        } else {
            absolute.encode(buffer);
        }
    }

    #[inline]
    pub fn shortest_vertical(from: u16, to: u16) -> Option<Self> {
        if from == to {
            return None;
        }

        let absolute = Self::Row(to);
        let distance = to.abs_diff(from);
        let relative = if to < from {
            Self::Up(distance)
        } else {
            Self::Down(distance)
        };

        if relative.length() <= absolute.length() {
            Some(relative)
        } else {
            Some(absolute)
        }
    }

    #[inline]
    pub fn shortest_horizontal(from: u16, to: u16) -> Option<Self> {
        if from == to {
            return None;
        }

        let absolute = Self::Column(to);
        let distance = to.abs_diff(from);
        let relative = if to < from {
            Self::Left(distance)
        } else {
            Self::Right(distance)
        };

        if relative.length() <= absolute.length() {
            Some(relative)
        } else {
            Some(absolute)
        }
    }

    #[inline]
    pub fn encode(self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(b"\x1b[");
        match self {
            Self::Absolute(Position::ZERO) => buffer.push(b'H'),
            Self::Absolute(position) => {
                let row = Digits::from_u16(position.y().saturating_add(1));
                let col = Digits::from_u16(position.x().saturating_add(1));

                buffer.extend_from_slice(row.ascii());
                buffer.push(b';');
                buffer.extend_from_slice(col.ascii());
                buffer.push(b'H');
            }
            Self::Column(0) => buffer.push(b'G'),
            Self::Column(col) => {
                let digits = Digits::from_u16(col.saturating_add(1));
                buffer.extend_from_slice(digits.ascii());
                buffer.push(b'G');
            }
            Self::Row(0) => buffer.push(b'd'),
            Self::Row(row) => {
                let digits = Digits::from_u16(row.saturating_add(1));
                buffer.extend_from_slice(digits.ascii());
                buffer.push(b'd');
            }
            Self::Up(1) => buffer.push(b'A'),
            Self::Up(count) => {
                let digits = Digits::from_u16(count);
                buffer.extend_from_slice(digits.ascii());
                buffer.push(b'A');
            }
            Self::Down(1) => buffer.push(b'B'),
            Self::Down(count) => {
                let digits = Digits::from_u16(count);
                buffer.extend_from_slice(digits.ascii());
                buffer.push(b'B');
            }
            Self::Right(1) => buffer.push(b'C'),
            Self::Right(count) => {
                let digits = Digits::from_u16(count);
                buffer.extend_from_slice(digits.ascii());
                buffer.push(b'C');
            }
            Self::Left(1) => buffer.push(b'D'),
            Self::Left(count) => {
                let digits = Digits::from_u16(count);
                buffer.extend_from_slice(digits.ascii());
                buffer.push(b'D');
            }
        }
    }

    #[inline]
    pub fn length(self) -> usize {
        match self {
            Self::Absolute(Position::ZERO) => 3,
            Self::Absolute(position) => {
                4 + Digits::from_u16(position.y().saturating_add(1)).count() as usize
                    + Digits::from_u16(position.x().saturating_add(1)).count() as usize
            }
            Self::Column(0) | Self::Row(0) => 3,
            Self::Column(col) => 3 + Digits::from_u16(col.saturating_add(1)).count() as usize,
            Self::Row(row) => 3 + Digits::from_u16(row.saturating_add(1)).count() as usize,
            Self::Up(1) | Self::Down(1) | Self::Right(1) | Self::Left(1) => 3,
            Self::Up(count) | Self::Down(count) | Self::Right(count) | Self::Left(count) => {
                3 + Digits::from_u16(count).count() as usize
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(motion: Motion) -> Vec<u8> {
        let mut buffer = Vec::new();
        motion.encode(&mut buffer);
        buffer
    }

    fn shortest(from: Option<Position>, to: Position) -> Vec<u8> {
        let mut buffer = Vec::new();
        Motion::encode_shortest(&mut buffer, from, to);
        buffer
    }

    fn samples() -> Vec<Motion> {
        let mut motions = Vec::new();

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
            motions.push(Motion::Absolute(position));
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
            motions.push(Motion::Column(value));
            motions.push(Motion::Row(value));
            motions.push(Motion::Up(value));
            motions.push(Motion::Down(value));
            motions.push(Motion::Right(value));
            motions.push(Motion::Left(value));
        }

        motions
    }

    #[test]
    fn length_matches_encoded_length() {
        for motion in samples() {
            let bytes = encoded(motion);

            assert_eq!(motion.length(), bytes.len(), "{bytes:?}");
        }
    }

    #[test]
    fn shortest_uses_absolute_when_there_is_no_cursor() {
        assert_eq!(shortest(None, Position::new(5, 3)), b"\x1b[4;6H");
        assert_eq!(shortest(None, Position::ZERO), b"\x1b[H");
        assert_eq!(
            shortest(None, Position::new(u16::MAX, u16::MAX)),
            b"\x1b[65535;65535H"
        );
    }

    #[test]
    fn shortest_emits_nothing_when_already_there() {
        assert!(shortest(Some(Position::new(10, 10)), Position::new(10, 10)).is_empty());
    }

    #[test]
    fn shortest_prefers_a_column_move() {
        assert_eq!(
            shortest(Some(Position::new(10, 5)), Position::new(0, 5)),
            b"\x1b[G"
        );
    }

    #[test]
    fn shortest_prefers_short_relative_moves() {
        assert_eq!(
            shortest(Some(Position::ZERO), Position::new(3, 0)),
            b"\x1b[3C"
        );
        assert_eq!(
            shortest(Some(Position::ZERO), Position::new(1, 0)),
            b"\x1b[C"
        );
        assert_eq!(
            shortest(Some(Position::new(5, 0)), Position::new(4, 0)),
            b"\x1b[D"
        );
        assert_eq!(
            shortest(Some(Position::new(4, 0)), Position::new(1, 0)),
            b"\x1b[3D"
        );
        assert_eq!(
            shortest(Some(Position::ZERO), Position::new(0, 2)),
            b"\x1b[2B"
        );
        assert_eq!(
            shortest(Some(Position::ZERO), Position::new(0, 1)),
            b"\x1b[B"
        );
        assert_eq!(
            shortest(Some(Position::new(0, 5)), Position::new(0, 4)),
            b"\x1b[A"
        );
    }

    #[test]
    fn shortest_prefers_absolute_moves_at_origin() {
        assert_eq!(
            shortest(Some(Position::new(20, 0)), Position::ZERO),
            b"\x1b[H"
        );
    }

    #[test]
    fn shortest_prefers_a_row_move() {
        assert_eq!(
            shortest(Some(Position::new(5, 5)), Position::new(5, 0)),
            b"\x1b[d"
        );
    }

    #[test]
    fn shortest_combines_short_relative_moves() {
        assert_eq!(
            shortest(Some(Position::new(40, 10)), Position::new(41, 11)),
            b"\x1b[B\x1b[C"
        );
    }

    #[test]
    fn shortest_prefers_absolute_for_distant_targets() {
        assert_eq!(
            shortest(Some(Position::ZERO), Position::new(70, 20)),
            b"\x1b[21;71H"
        );
        assert_eq!(
            shortest(Some(Position::new(40, 10)), Position::new(0, 20)),
            b"\x1b[21;1H"
        );
    }
}
