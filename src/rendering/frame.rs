use crate::common::{cell::Cell, position::Position, size::Size};
use std::ops::{Index, IndexMut};

pub struct Frame {
    size: Size,
    cells: Vec<Cell>,
}

impl Frame {
    #[inline]
    pub fn new(size: Size) -> Self {
        let cells = vec![Cell::EMPTY; size.area() as usize];
        Self { size, cells }
    }

    #[inline]
    pub fn resize(&mut self, new_size: Size) {
        self.cells.resize(new_size.area() as usize, Cell::EMPTY);
        self.size = new_size;
    }

    #[inline]
    pub fn size(&self) -> Size {
        self.size
    }

    #[inline]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    #[inline]
    pub fn clear(&mut self) {
        self.cells.fill(Cell::EMPTY);
    }

    #[inline]
    pub fn fill(&mut self, fill: Cell) {
        self.cells.fill(fill);
    }

    #[inline]
    pub fn fill_with(&mut self, mut source: impl FnMut(Position) -> Cell) {
        for y in 0..self.size.height() {
            for x in 0..self.size.width() {
                let position = Position::new(x, y);
                self[position] = source(position);
            }
        }
    }
}

impl Index<Position> for Frame {
    type Output = Cell;

    #[inline]
    fn index(&self, position: Position) -> &Self::Output {
        let index = position.flatten(self.size());
        &self.cells[index as usize]
    }
}

impl IndexMut<Position> for Frame {
    #[inline]
    fn index_mut(&mut self, position: Position) -> &mut Self::Output {
        let index = position.flatten(self.size());
        &mut self.cells[index as usize]
    }
}
