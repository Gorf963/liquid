#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Coordinate {
    pub x: usize,
    pub y: usize,
}

impl Coordinate {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoardError {
    ZeroDimension,
    SizeOverflow,
    AllocationFailed,
    OutOfBounds,
    Blocked,
    CapacityExceeded,
    CellContainsLiquid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    position: Coordinate,
    capacity: u32,
    liquid: u32,
    blocked: bool,
}

impl Cell {
    pub const fn position(&self) -> Coordinate {
        self.position
    }

    pub const fn capacity(&self) -> u32 {
        self.capacity
    }

    pub const fn liquid(&self) -> u32 {
        self.liquid
    }

    pub const fn is_blocked(&self) -> bool {
        self.blocked
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Board {
    pub fn new(width: usize, height: usize, cell_capacity: u32) -> Result<Self, BoardError> {
        if width == 0 || height == 0 {
            return Err(BoardError::ZeroDimension);
        }

        let cell_count = width.checked_mul(height).ok_or(BoardError::SizeOverflow)?;
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(cell_count)
            .map_err(|_| BoardError::AllocationFailed)?;
        cells.extend((0..cell_count).map(|index| Cell {
            position: Coordinate::new(index % width, index / width),
            capacity: cell_capacity,
            liquid: 0,
            blocked: false,
        }));

        Ok(Self {
            width,
            height,
            cells,
        })
    }

    pub const fn width(&self) -> usize {
        self.width
    }

    pub const fn height(&self) -> usize {
        self.height
    }

    pub fn cell(&self, position: Coordinate) -> Option<&Cell> {
        self.index_of(position).map(|index| &self.cells[index])
    }

    pub fn block_cell(&mut self, position: Coordinate) -> Result<(), BoardError> {
        let index = self.index_of(position).ok_or(BoardError::OutOfBounds)?;
        let cell = &mut self.cells[index];
        if cell.liquid != 0 {
            return Err(BoardError::CellContainsLiquid);
        }
        cell.blocked = true;
        Ok(())
    }

    pub fn add_liquid(&mut self, position: Coordinate, amount: u32) -> Result<(), BoardError> {
        let index = self.index_of(position).ok_or(BoardError::OutOfBounds)?;
        let cell = &mut self.cells[index];
        if cell.blocked {
            return Err(BoardError::Blocked);
        }
        if amount > cell.capacity - cell.liquid {
            return Err(BoardError::CapacityExceeded);
        }
        cell.liquid += amount;
        Ok(())
    }

    fn index_of(&self, position: Coordinate) -> Option<usize> {
        if position.x >= self.width || position.y >= self.height {
            return None;
        }
        Some(position.y * self.width + position.x)
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, BoardError, Coordinate};

    #[test]
    fn construction_rejects_zero_dimensions_and_size_overflow() {
        assert_eq!(Board::new(0, 2, 10), Err(BoardError::ZeroDimension));
        assert_eq!(Board::new(2, 0, 10), Err(BoardError::ZeroDimension));
        assert_eq!(Board::new(usize::MAX, 2, 10), Err(BoardError::SizeOverflow));
    }

    #[test]
    fn coordinate_access_returns_the_expected_empty_cell() {
        let board = Board::new(3, 2, 10).expect("valid board dimensions");
        let cell = board
            .cell(Coordinate::new(2, 1))
            .expect("coordinate is in bounds");

        assert_eq!(cell.position(), Coordinate::new(2, 1));
        assert_eq!(cell.capacity(), 10);
        assert_eq!(cell.liquid(), 0);
        assert!(!cell.is_blocked());
    }

    #[test]
    fn out_of_bounds_access_returns_none() {
        let board = Board::new(2, 2, 10).expect("valid board dimensions");

        assert!(board.cell(Coordinate::new(2, 0)).is_none());
        assert!(board.cell(Coordinate::new(0, 2)).is_none());
    }

    #[test]
    fn liquid_can_be_added_to_an_open_cell_within_capacity() {
        let mut board = Board::new(2, 2, 10).expect("valid board dimensions");
        let position = Coordinate::new(1, 0);

        board.add_liquid(position, 4).expect("cell has capacity");
        assert_eq!(board.cell(position).map(|cell| cell.liquid()), Some(4));
    }

    #[test]
    fn blocked_cells_start_empty_and_reject_liquid_without_changing_state() {
        let mut board = Board::new(2, 2, 10).expect("valid board dimensions");
        let position = Coordinate::new(1, 1);
        board.block_cell(position).expect("cell is empty");

        assert_eq!(board.add_liquid(position, 3), Err(BoardError::Blocked));
        let cell = board.cell(position).expect("coordinate is in bounds");
        assert!(cell.is_blocked());
        assert_eq!(cell.liquid(), 0);
    }

    #[test]
    fn liquid_addition_cannot_exceed_cell_capacity() {
        let mut board = Board::new(1, 1, 5).expect("valid board dimensions");
        let position = Coordinate::new(0, 0);
        board.add_liquid(position, 4).expect("cell has capacity");

        assert_eq!(
            board.add_liquid(position, 2),
            Err(BoardError::CapacityExceeded)
        );
        assert_eq!(board.cell(position).map(|cell| cell.liquid()), Some(4));
    }

    #[test]
    fn nonempty_cells_cannot_become_blocked() {
        let mut board = Board::new(1, 1, 5).expect("valid board dimensions");
        let position = Coordinate::new(0, 0);
        board.add_liquid(position, 1).expect("cell has capacity");

        assert_eq!(
            board.block_cell(position),
            Err(BoardError::CellContainsLiquid)
        );
        let cell = board.cell(position).expect("coordinate is in bounds");
        assert!(!cell.is_blocked());
        assert_eq!(cell.liquid(), 1);
    }
}
