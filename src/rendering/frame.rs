use crate::common::{cell::Cell, position::Position, size::Size};

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
    fn index(&mut self, position: Position) -> Option<&mut Cell> {
        let index = position.flatten(self.size())?;
        Some(&mut self.cells[index as usize])
    }

    #[inline]
    pub fn fill_with(&mut self, mut source: impl FnMut(Position) -> Cell) {
        for y in 0..self.size.height() {
            for x in 0..self.size.width() {
                let position = Position::new(x, y);
                let cell = self.index(position).unwrap();
                *cell = source(position);
            }
        }
    }
}
