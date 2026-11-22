use crate::common::{cell::Cell, position::Position, size::Size};

pub struct Frame {
    size: Size,
    cells: Vec<Cell>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Delta {
    position: Position,
    cell: Cell,
}

impl Delta {
    #[inline]
    pub const fn new(position: Position, cell: Cell) -> Self {
        Self { position, cell }
    }

    #[inline]
    pub const fn position(&self) -> Position {
        self.position
    }

    #[inline]
    pub const fn cell(&self) -> Cell {
        self.cell
    }
}

impl Frame {
    #[inline]
    pub fn new(size: Size) -> Self {
        let cells = vec![Cell::EMPTY; size.area() as usize];
        Self { size, cells }
    }

    #[inline]
    pub fn resize(&mut self, new_size: Size) {
        if new_size == self.size {
            return;
        }

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
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }

    #[inline]
    pub fn rows(&self) -> impl ExactSizeIterator<Item = &[Cell]> {
        self.cells.chunks_exact(self.size.width() as usize)
    }

    #[inline]
    pub fn rows_mut(&mut self) -> impl ExactSizeIterator<Item = &mut [Cell]> {
        self.cells.chunks_exact_mut(self.size.width() as usize)
    }

    #[inline]
    pub fn indexed_rows(&self) -> impl Iterator<Item = (u16, &[Cell])> {
        let width = self.size.width() as usize;
        (0..self.size.height()).zip(self.cells.chunks_exact(width))
    }

    #[inline]
    pub fn indexed_rows_mut(&mut self) -> impl Iterator<Item = (u16, &mut [Cell])> {
        let width = self.size.width() as usize;
        (0..self.size.height()).zip(self.cells.chunks_exact_mut(width))
    }

    #[inline]
    pub fn row(&self, y: u16) -> Option<&[Cell]> {
        let width = self.size.width() as usize;
        let start = y as usize * width;
        self.cells.get(start..start + width)
    }

    #[inline]
    pub fn row_mut(&mut self, y: u16) -> Option<&mut [Cell]> {
        let width = self.size.width() as usize;
        let start = y as usize * width;
        self.cells.get_mut(start..start + width)
    }

    #[inline]
    pub fn get(&self, position: Position) -> Option<&Cell> {
        self.row(position.y())?.get(position.x() as usize)
    }

    #[inline]
    pub fn get_mut(&mut self, position: Position) -> Option<&mut Cell> {
        self.row_mut(position.y())?.get_mut(position.x() as usize)
    }

    #[inline]
    pub fn fill(&mut self, fill: Cell) {
        self.cells.fill(fill);
    }

    #[inline]
    pub fn clear(&mut self) {
        self.fill(Cell::EMPTY);
    }

    #[inline]
    pub fn fill_with(&mut self, mut source: impl FnMut(Position) -> Cell) {
        let width = self.size.width();
        for (y, row) in self.indexed_rows_mut() {
            for (x, cell) in (0..width).zip(row) {
                *cell = source(Position::new(x, y));
            }
        }
    }

    pub fn diff<'a>(&'a self, previous: &'a Frame) -> impl Iterator<Item = Delta> + 'a {
        self.indexed_rows()
            .zip(previous.indexed_rows())
            .filter(|((_, current), (_, previous))| current != previous)
            .flat_map(|((y, current), (_, previous))| {
                current
                    .iter()
                    .zip(previous)
                    .zip(0..self.size().width())
                    .filter(|&((new, old), _)| new != old)
                    .map(move |((new, _), x)| Delta::new(Position::new(x, y), *new))
            })
    }
}
