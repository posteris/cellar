use crate::common::{cell::Cell, position::Position, size::Size};

pub trait Canvas {
    fn size(&self) -> Size;
    fn plot(&mut self, position: Position, cell: Cell);
}
