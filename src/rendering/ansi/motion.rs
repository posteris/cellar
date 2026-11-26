use crate::{common::position::Position, rendering::ansi::sequence::Sequence};

impl Sequence {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shortest(from: Option<Position>, to: Position) -> Vec<u8> {
        let mut buffer = Vec::new();
        Sequence::encode_shortest(&mut buffer, from, to);
        buffer
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

    #[test]
    fn shortest_axis_helpers_skip_unchanged_axes() {
        assert_eq!(Sequence::shortest_vertical(5, 5), None);
        assert_eq!(Sequence::shortest_horizontal(5, 5), None);
    }
}
