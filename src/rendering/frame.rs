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
    pub fn size(&self) -> Size {
        self.size
    }

    #[inline]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    #[inline]
    pub fn clear(&mut self) {
        self.cells.clear();
    }

    #[inline]
    pub fn resize(&mut self, new_size: Size) {
        if new_size.area() > self.size.area() {
            let additional = new_size.area() - self.size.area();
            self.cells.reserve(additional as usize);
        }

        self.size = new_size;
    }

    #[inline]
    pub fn fill(&mut self, mut source: impl FnMut(Position) -> Cell) {
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
        let stride = self.size().width();
        let index = position.flatten(stride);
        &self.cells[index as usize]
    }
}

impl IndexMut<Position> for Frame {
    #[inline]
    fn index_mut(&mut self, position: Position) -> &mut Self::Output {
        let stride = self.size().width();
        let index = position.flatten(stride);
        &mut self.cells[index as usize]
    }
}
