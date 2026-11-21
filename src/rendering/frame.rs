use crate::common::{cell::Cell, position::Position, size::Size};

pub struct Frame {
    size: Size,
    cells: Vec<Cell>,
}

impl Frame {
    #[inline]
    pub fn new(size: Size) -> Self {
        let cells = Vec::with_capacity(size.area() as usize);
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
        if new_size.area() < self.size.area() {
            return;
        }

        let additional = new_size.area() - self.size.area();
        self.cells.reserve(additional as usize);
        self.size = new_size;
    }

    #[inline]
    pub fn fill(&mut self, mut source: impl FnMut(Position) -> Cell) {
        self.clear();
        for y in 0..self.size.height() {
            for x in 0..self.size.width() {
                self.cells.push(source(Position::new(x, y)));
            }
        }
    }
}
