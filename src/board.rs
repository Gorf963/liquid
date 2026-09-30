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
    IdenticalEndpoints,
    NotAdjacent,
    Blocked,
    CapacityExceeded,
    CellContainsLiquid,
    CellHasPipes,
    CellContainsPump,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PumpSide {
    First,
    Second,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    position: Coordinate,
    capacity: u32,
    liquid: u32,
    blocked: bool,
    pump_side: Option<PumpSide>,
    liquid_side: Option<PumpSide>,
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

    pub const fn pump_side(&self) -> Option<PumpSide> {
        self.pump_side
    }

    pub const fn liquid_side(&self) -> Option<PumpSide> {
        self.liquid_side
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    pipe_connections: Vec<u8>,
}

const NORTH: u8 = 1 << 0;
const EAST: u8 = 1 << 1;
const SOUTH: u8 = 1 << 2;
const WEST: u8 = 1 << 3;
const DIRECTIONS: [u8; 4] = [NORTH, EAST, SOUTH, WEST];

#[derive(Debug, Clone, Copy)]
struct Transfer {
    source: usize,
    destination: usize,
    side: PumpSide,
    amount: u32,
}

const fn side_sign(side: PumpSide) -> i64 {
    match side {
        PumpSide::First => 1,
        PumpSide::Second => -1,
    }
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
            pump_side: None,
            liquid_side: None,
        }));

        let mut pipe_connections = Vec::new();
        pipe_connections
            .try_reserve_exact(cell_count)
            .map_err(|_| BoardError::AllocationFailed)?;
        pipe_connections.resize(cell_count, 0);

        Ok(Self {
            width,
            height,
            cells,
            pipe_connections,
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
        if self.cells[index].liquid != 0 {
            return Err(BoardError::CellContainsLiquid);
        }
        if self.cell_has_pipe(index) {
            return Err(BoardError::CellHasPipes);
        }
        if self.cells[index].pump_side.is_some() {
            return Err(BoardError::CellContainsPump);
        }
        self.cells[index].blocked = true;
        Ok(())
    }

    pub fn place_pump(&mut self, position: Coordinate, side: PumpSide) -> Result<(), BoardError> {
        let index = self.index_of(position).ok_or(BoardError::OutOfBounds)?;
        let cell = &mut self.cells[index];
        if cell.blocked {
            return Err(BoardError::Blocked);
        }
        if cell.pump_side.is_some() {
            return Err(BoardError::CellContainsPump);
        }
        cell.pump_side = Some(side);
        if cell.liquid == 0 {
            cell.liquid_side = Some(side);
        }
        Ok(())
    }

    pub fn pump_side(&self, position: Coordinate) -> Result<Option<PumpSide>, BoardError> {
        let index = self.index_of(position).ok_or(BoardError::OutOfBounds)?;
        Ok(self.cells[index].pump_side)
    }

    pub fn pump_count(&self, side: PumpSide) -> usize {
        self.cells
            .iter()
            .filter(|cell| cell.pump_side == Some(side))
            .count()
    }

    pub fn toggle_pipe_connection(
        &mut self,
        first: Coordinate,
        second: Coordinate,
    ) -> Result<bool, BoardError> {
        let (first_index, second_index, first_direction, second_direction) =
            self.connection_indices(first, second)?;
        if self.cells[first_index].blocked || self.cells[second_index].blocked {
            return Err(BoardError::Blocked);
        }

        let first_to_second = self.pipe_connections[first_index] & first_direction != 0;
        let second_to_first = self.pipe_connections[second_index] & second_direction != 0;
        if first_to_second || second_to_first {
            self.pipe_connections[first_index] &= !first_direction;
            self.pipe_connections[second_index] &= !second_direction;
            Ok(false)
        } else {
            self.pipe_connections[first_index] |= first_direction;
            Ok(true)
        }
    }

    pub fn has_pipe_connection(
        &self,
        first: Coordinate,
        second: Coordinate,
    ) -> Result<bool, BoardError> {
        let (first_index, _, first_direction, _) = self.connection_indices(first, second)?;
        Ok(self.pipe_connections[first_index] & first_direction != 0)
    }

    pub fn connected_pipe_count(&self, position: Coordinate) -> Result<usize, BoardError> {
        let index = self.index_of(position).ok_or(BoardError::OutOfBounds)?;
        Ok(self.pipe_connections[index].count_ones() as usize)
    }

    pub fn add_liquid(&mut self, position: Coordinate, amount: u32) -> Result<(), BoardError> {
        let index = self.index_of(position).ok_or(BoardError::OutOfBounds)?;
        let side = self.cells[index]
            .liquid_side
            .or(self.cells[index].pump_side)
            .unwrap_or(PumpSide::First);
        self.add_liquid_for_side(position, side, amount)
    }

    pub fn add_liquid_for_side(
        &mut self,
        position: Coordinate,
        side: PumpSide,
        amount: u32,
    ) -> Result<(), BoardError> {
        let index = self.index_of(position).ok_or(BoardError::OutOfBounds)?;
        let cell = &mut self.cells[index];
        if cell.blocked {
            return Err(BoardError::Blocked);
        }
        if amount == 0 {
            return Ok(());
        }

        let existing = cell
            .liquid_side
            .filter(|_| cell.liquid > 0)
            .map_or(0, |current_side| {
                side_sign(current_side) * i64::from(cell.liquid)
            });
        let resulting = existing + side_sign(side) * i64::from(amount);
        if resulting.unsigned_abs() > u64::from(cell.capacity) {
            return Err(BoardError::CapacityExceeded);
        }
        cell.liquid = resulting.unsigned_abs() as u32;
        cell.liquid_side = match resulting.cmp(&0) {
            std::cmp::Ordering::Greater => Some(PumpSide::First),
            std::cmp::Ordering::Less => Some(PumpSide::Second),
            std::cmp::Ordering::Equal => None,
        };
        Ok(())
    }

    /// Advances one simultaneous flow tick. Capacity contention is resolved in
    /// receiver row-major order, then sender row-major order; if returned flow
    /// would overfill a sender, all incoming flow to that cell is rejected.
    pub fn advance_tick(&mut self) -> Result<(), BoardError> {
        let cell_count = self.cells.len();
        let transfer_count = self
            .pipe_connections
            .iter()
            .try_fold(0usize, |count, mask| {
                count
                    .checked_add(mask.count_ones() as usize)
                    .ok_or(BoardError::SizeOverflow)
            })?;

        let mut transfers = Vec::new();
        transfers
            .try_reserve_exact(transfer_count)
            .map_err(|_| BoardError::AllocationFailed)?;
        let mut offered = Vec::new();
        offered
            .try_reserve_exact(cell_count)
            .map_err(|_| BoardError::AllocationFailed)?;
        offered.resize(cell_count, 0u32);

        let mut snapshot_amounts = Vec::new();
        snapshot_amounts
            .try_reserve_exact(cell_count)
            .map_err(|_| BoardError::AllocationFailed)?;
        let mut snapshot_sides = Vec::new();
        snapshot_sides
            .try_reserve_exact(cell_count)
            .map_err(|_| BoardError::AllocationFailed)?;

        let mut signed_amounts = Vec::new();
        signed_amounts
            .try_reserve_exact(cell_count)
            .map_err(|_| BoardError::AllocationFailed)?;
        let mut accepted = Vec::new();
        accepted
            .try_reserve_exact(transfer_count)
            .map_err(|_| BoardError::AllocationFailed)?;
        let mut incoming = Vec::new();
        incoming
            .try_reserve_exact(cell_count)
            .map_err(|_| BoardError::AllocationFailed)?;

        for cell in &self.cells {
            let mut amount = cell.liquid;
            if cell.pump_side.is_some() && cell.liquid_side.is_some() {
                amount += 8.min(cell.capacity - amount);
            }
            snapshot_amounts.push(amount);
            snapshot_sides.push(cell.liquid_side);
        }

        for (source, offered_amount) in offered.iter_mut().enumerate() {
            let Some(side) = snapshot_sides[source].filter(|_| snapshot_amounts[source] > 0) else {
                continue;
            };
            let pipe_count = self.pipe_connections[source].count_ones();
            if pipe_count == 0 {
                continue;
            }

            let total = snapshot_amounts[source].saturating_sub(1).min(8);
            if total == 0 {
                continue;
            }
            let divisor = pipe_count;
            let base = total / divisor;
            let remainder = total % divisor;
            let mut remainder_left = remainder;
            for direction in DIRECTIONS {
                if self.pipe_connections[source] & direction == 0 {
                    continue;
                }
                let amount = base + u32::from(remainder_left > 0);
                remainder_left = remainder_left.saturating_sub(1);
                if let Some(destination) = self.neighbor_index(source, direction) {
                    transfers.push(Transfer {
                        source,
                        destination,
                        side,
                        amount,
                    });
                    *offered_amount += amount;
                }
            }
        }

        transfers.sort_unstable_by_key(|transfer| (transfer.destination, transfer.source));

        signed_amounts.extend(
            snapshot_amounts
                .iter()
                .zip(&snapshot_sides)
                .zip(&offered)
                .map(|((&amount, &side), &offered_amount)| {
                    let signed = side.filter(|_| amount > 0).map_or(0, |current_side| {
                        side_sign(current_side) * i64::from(amount)
                    });
                    signed
                        - side.filter(|_| amount > 0).map_or(0, |current_side| {
                            side_sign(current_side) * i64::from(offered_amount)
                        })
                }),
        );

        accepted.resize(transfers.len(), 0u32);
        incoming.resize(cell_count, 0u32);

        for (transfer_index, transfer) in transfers.iter().enumerate() {
            let sign = side_sign(transfer.side);
            let destination = transfer.destination;
            for _ in 0..transfer.amount {
                let resulting = signed_amounts[destination] + sign;
                if resulting.unsigned_abs() <= u64::from(self.cells[destination].capacity) {
                    signed_amounts[destination] = resulting;
                    accepted[transfer_index] += 1;
                    incoming[destination] += 1;
                }
            }
        }

        for (transfer_index, transfer) in transfers.iter().enumerate() {
            let rejected = transfer.amount - accepted[transfer_index];
            signed_amounts[transfer.source] += side_sign(transfer.side) * i64::from(rejected);
        }

        while let Some(overfull) = signed_amounts
            .iter()
            .enumerate()
            .find_map(|(index, amount)| {
                (amount.unsigned_abs() > u64::from(self.cells[index].capacity)).then_some(index)
            })
        {
            let mut rejected_any = false;
            for (transfer_index, transfer) in transfers.iter().enumerate() {
                if transfer.destination == overfull && accepted[transfer_index] > 0 {
                    let returned = accepted[transfer_index];
                    signed_amounts[transfer.source] +=
                        side_sign(transfer.side) * i64::from(returned);
                    signed_amounts[overfull] -= side_sign(transfer.side) * i64::from(returned);
                    incoming[overfull] -= returned;
                    accepted[transfer_index] = 0;
                    rejected_any = true;
                }
            }
            if !rejected_any {
                return Err(BoardError::CapacityExceeded);
            }
        }

        for (index, cell) in self.cells.iter_mut().enumerate() {
            let signed = signed_amounts[index];
            let (amount, side) = match signed.cmp(&0) {
                std::cmp::Ordering::Greater => (signed as u32, Some(PumpSide::First)),
                std::cmp::Ordering::Less => ((-signed) as u32, Some(PumpSide::Second)),
                std::cmp::Ordering::Equal
                    if incoming[index] == 0
                        && snapshot_amounts[index] == 0
                        && cell.pump_side.is_some() =>
                {
                    (0, snapshot_sides[index])
                }
                std::cmp::Ordering::Equal => (0, None),
            };
            cell.liquid = amount;
            cell.liquid_side = side;
        }

        Ok(())
    }

    fn neighbor_index(&self, index: usize, direction: u8) -> Option<usize> {
        let position = self.cells[index].position;
        match direction {
            NORTH if position.y > 0 => self.index_of(Coordinate::new(position.x, position.y - 1)),
            EAST if position.x + 1 < self.width => {
                self.index_of(Coordinate::new(position.x + 1, position.y))
            }
            SOUTH if position.y + 1 < self.height => {
                self.index_of(Coordinate::new(position.x, position.y + 1))
            }
            WEST if position.x > 0 => self.index_of(Coordinate::new(position.x - 1, position.y)),
            _ => None,
        }
    }

    fn cell_has_pipe(&self, index: usize) -> bool {
        if self.pipe_connections[index] != 0 {
            return true;
        }
        DIRECTIONS.into_iter().any(|direction| {
            self.neighbor_index(index, direction)
                .is_some_and(|neighbor| {
                    self.pipe_connections[neighbor] & opposite_direction(direction) != 0
                })
        })
    }

    fn index_of(&self, position: Coordinate) -> Option<usize> {
        if position.x >= self.width || position.y >= self.height {
            return None;
        }
        Some(position.y * self.width + position.x)
    }

    fn connection_indices(
        &self,
        first: Coordinate,
        second: Coordinate,
    ) -> Result<(usize, usize, u8, u8), BoardError> {
        let first_index = self.index_of(first).ok_or(BoardError::OutOfBounds)?;
        let second_index = self.index_of(second).ok_or(BoardError::OutOfBounds)?;
        if first == second {
            return Err(BoardError::IdenticalEndpoints);
        }

        let (first_direction, second_direction) =
            if first.y == second.y && first.x.abs_diff(second.x) == 1 {
                if first.x < second.x {
                    (EAST, WEST)
                } else {
                    (WEST, EAST)
                }
            } else if first.x == second.x && first.y.abs_diff(second.y) == 1 {
                if first.y < second.y {
                    (SOUTH, NORTH)
                } else {
                    (NORTH, SOUTH)
                }
            } else {
                return Err(BoardError::NotAdjacent);
            };

        Ok((first_index, second_index, first_direction, second_direction))
    }
}

const fn opposite_direction(direction: u8) -> u8 {
    match direction {
        NORTH => SOUTH,
        EAST => WEST,
        SOUTH => NORTH,
        WEST => EAST,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, BoardError, Coordinate, PumpSide};

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
        assert_eq!(cell.pump_side(), None);
        assert_eq!(cell.liquid_side(), None);
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
    fn side_aware_addition_captures_or_neutralizes_without_exceeding_capacity() {
        let mut board = Board::new(1, 1, 8).expect("valid board dimensions");
        let position = Coordinate::new(0, 0);
        board
            .add_liquid_for_side(position, PumpSide::First, 5)
            .expect("cell has capacity");
        board
            .add_liquid_for_side(position, PumpSide::Second, 7)
            .expect("opposing liquid captures the cell");

        let cell = board.cell(position).expect("coordinate is in bounds");
        assert_eq!(cell.liquid(), 2);
        assert_eq!(cell.liquid_side(), Some(PumpSide::Second));

        board
            .add_liquid_for_side(position, PumpSide::First, 2)
            .expect("equal opposing liquid neutralizes the cell");
        let cell = board.cell(position).expect("coordinate is in bounds");
        assert_eq!(cell.liquid(), 0);
        assert_eq!(cell.liquid_side(), None);
    }

    #[test]
    fn legacy_liquid_addition_uses_existing_color_or_first_side_for_empty_cells() {
        let mut board = Board::new(2, 1, 10).expect("valid board dimensions");
        let first = Coordinate::new(0, 0);
        let second = Coordinate::new(1, 0);
        board
            .add_liquid(first, 3)
            .expect("legacy addition colors an empty cell");
        board
            .add_liquid_for_side(second, PumpSide::Second, 2)
            .expect("cell receives second-side liquid");
        board
            .add_liquid(second, 3)
            .expect("legacy addition follows existing color");

        assert_eq!(
            board.cell(first).map(|cell| cell.liquid_side()),
            Some(Some(PumpSide::First))
        );
        assert_eq!(board.cell(second).map(|cell| cell.liquid()), Some(5));
        assert_eq!(
            board.cell(second).map(|cell| cell.liquid_side()),
            Some(Some(PumpSide::Second))
        );
    }

    #[test]
    fn pump_output_uses_its_current_side_before_one_way_flow() {
        let mut board = Board::new(2, 1, 20).expect("valid board dimensions");
        let source = Coordinate::new(0, 0);
        let pump = Coordinate::new(1, 0);
        board
            .toggle_pipe_connection(source, pump)
            .expect("adjacent cells accept a pipe");
        board
            .add_liquid_for_side(source, PumpSide::First, 10)
            .expect("source has capacity");
        board
            .place_pump(pump, PumpSide::Second)
            .expect("empty cell accepts a pump");

        board.advance_tick().expect("tick succeeds");

        let source_cell = board.cell(source).expect("coordinate is in bounds");
        assert_eq!(source_cell.liquid(), 2);
        assert_eq!(source_cell.liquid_side(), Some(PumpSide::First));
        let cell = board.cell(pump).expect("coordinate is in bounds");
        assert_eq!(cell.liquid(), 0);
        assert_eq!(cell.pump_side(), Some(PumpSide::Second));
        assert_eq!(cell.liquid_side(), None);
    }

    #[test]
    fn pump_output_can_be_neutralized_by_opposing_flow() {
        let mut board = Board::new(2, 1, 12).expect("valid board dimensions");
        let source = Coordinate::new(0, 0);
        let pump = Coordinate::new(1, 0);
        board
            .toggle_pipe_connection(source, pump)
            .expect("adjacent cells accept a pipe");
        board
            .add_liquid_for_side(source, PumpSide::First, 9)
            .expect("source has capacity");
        board
            .place_pump(pump, PumpSide::Second)
            .expect("empty cell accepts a pump");

        board.advance_tick().expect("tick succeeds");

        let cell = board.cell(pump).expect("coordinate is in bounds");
        assert_eq!(cell.liquid(), 0);
        assert_eq!(cell.liquid_side(), None);
    }

    #[test]
    fn pump_output_flows_through_an_adjacent_pipe_in_the_same_tick() {
        let mut board = Board::new(2, 1, 160).expect("valid board dimensions");
        let pump = Coordinate::new(0, 0);
        let neighbor = Coordinate::new(1, 0);
        board
            .place_pump(pump, PumpSide::Second)
            .expect("empty cell accepts a pump");
        board
            .toggle_pipe_connection(pump, neighbor)
            .expect("adjacent cells accept a pipe");

        board.advance_tick().expect("tick succeeds");

        let pump_cell = board.cell(pump).expect("coordinate is in bounds");
        assert_eq!(pump_cell.liquid(), 1);
        assert_eq!(pump_cell.liquid_side(), Some(PumpSide::Second));
        let neighbor_cell = board.cell(neighbor).expect("coordinate is in bounds");
        assert_eq!(neighbor_cell.liquid(), 7);
        assert_eq!(neighbor_cell.liquid_side(), Some(PumpSide::Second));
    }

    #[test]
    fn pump_seeds_empty_cell_emits_eight_and_stops_at_capacity() {
        let mut board = Board::new(1, 1, 10).expect("valid board dimensions");
        let pump = Coordinate::new(0, 0);
        board
            .place_pump(pump, PumpSide::Second)
            .expect("empty cell accepts a pump");

        board.advance_tick().expect("first tick succeeds");
        assert_eq!(board.cell(pump).map(|cell| cell.liquid()), Some(8));
        assert_eq!(
            board.cell(pump).map(|cell| cell.liquid_side()),
            Some(Some(PumpSide::Second))
        );

        board.advance_tick().expect("second tick succeeds");
        assert_eq!(board.cell(pump).map(|cell| cell.liquid()), Some(10));
    }

    #[test]
    fn flow_splits_evenly_with_remainders_in_north_east_south_west_order() {
        let mut board = Board::new(3, 3, 20).expect("valid board dimensions");
        let source = Coordinate::new(1, 1);
        let north = Coordinate::new(1, 0);
        let east = Coordinate::new(2, 1);
        let south = Coordinate::new(1, 2);
        for destination in [north, east, south] {
            board
                .toggle_pipe_connection(source, destination)
                .expect("adjacent cells accept a pipe");
        }
        board
            .add_liquid_for_side(source, PumpSide::First, 9)
            .expect("source has capacity");

        board.advance_tick().expect("tick succeeds");

        assert_eq!(board.cell(source).map(|cell| cell.liquid()), Some(1));
        assert_eq!(board.cell(north).map(|cell| cell.liquid()), Some(3));
        assert_eq!(board.cell(east).map(|cell| cell.liquid()), Some(3));
        assert_eq!(board.cell(south).map(|cell| cell.liquid()), Some(2));
    }

    #[test]
    fn source_keeps_rejected_flow_and_receiver_accounts_for_its_outflow() {
        let mut board = Board::new(3, 1, 5).expect("valid board dimensions");
        let source = Coordinate::new(0, 0);
        let receiver = Coordinate::new(1, 0);
        let sink = Coordinate::new(2, 0);
        board
            .toggle_pipe_connection(source, receiver)
            .expect("adjacent cells accept a pipe");
        board
            .toggle_pipe_connection(receiver, sink)
            .expect("adjacent cells accept a pipe");
        board
            .add_liquid_for_side(source, PumpSide::First, 5)
            .expect("source has capacity");
        board
            .add_liquid_for_side(receiver, PumpSide::First, 5)
            .expect("receiver has capacity");

        board.advance_tick().expect("tick succeeds");

        assert_eq!(board.cell(source).map(|cell| cell.liquid()), Some(1));
        assert_eq!(board.cell(receiver).map(|cell| cell.liquid()), Some(5));
        assert_eq!(board.cell(sink).map(|cell| cell.liquid()), Some(4));
    }

    #[test]
    fn opposing_flow_captures_a_cell_and_an_exact_tie_neutralizes_it() {
        let mut capture = Board::new(2, 1, 12).expect("valid board dimensions");
        let source = Coordinate::new(0, 0);
        let receiver = Coordinate::new(1, 0);
        capture
            .toggle_pipe_connection(source, receiver)
            .expect("adjacent cells accept a pipe");
        capture
            .add_liquid_for_side(source, PumpSide::First, 10)
            .expect("source has capacity");
        capture
            .add_liquid_for_side(receiver, PumpSide::Second, 2)
            .expect("receiver has capacity");
        capture.advance_tick().expect("capture tick succeeds");
        assert_eq!(capture.cell(receiver).map(|cell| cell.liquid()), Some(6));
        assert_eq!(
            capture.cell(receiver).map(|cell| cell.liquid_side()),
            Some(Some(PumpSide::First))
        );

        let mut tie = Board::new(2, 1, 12).expect("valid board dimensions");
        tie.toggle_pipe_connection(source, receiver)
            .expect("adjacent cells accept a pipe");
        tie.add_liquid_for_side(source, PumpSide::First, 2)
            .expect("source has capacity");
        tie.add_liquid_for_side(receiver, PumpSide::Second, 1)
            .expect("receiver has capacity");
        tie.advance_tick().expect("tie tick succeeds");
        assert_eq!(tie.cell(receiver).map(|cell| cell.liquid()), Some(0));
        assert_eq!(
            tie.cell(receiver).map(|cell| cell.liquid_side()),
            Some(None)
        );
    }

    #[test]
    fn neutral_cell_compares_incoming_sides_and_ties_to_empty() {
        let mut board = Board::new(3, 1, 10).expect("valid board dimensions");
        let first = Coordinate::new(0, 0);
        let neutral = Coordinate::new(1, 0);
        let second = Coordinate::new(2, 0);
        board
            .toggle_pipe_connection(first, neutral)
            .expect("adjacent cells accept a pipe");
        board
            .toggle_pipe_connection(second, neutral)
            .expect("adjacent cells accept a pipe");
        board
            .add_liquid_for_side(first, PumpSide::First, 2)
            .expect("first source has capacity");
        board
            .add_liquid_for_side(second, PumpSide::Second, 2)
            .expect("second source has capacity");

        board.advance_tick().expect("tick succeeds");

        let cell = board.cell(neutral).expect("coordinate is in bounds");
        assert_eq!(cell.liquid(), 0);
        assert_eq!(cell.liquid_side(), None);

        let mut winner = Board::new(3, 1, 10).expect("valid board dimensions");
        winner
            .toggle_pipe_connection(first, neutral)
            .expect("adjacent cells accept a pipe");
        winner
            .toggle_pipe_connection(second, neutral)
            .expect("adjacent cells accept a pipe");
        winner
            .add_liquid_for_side(first, PumpSide::First, 3)
            .expect("first source has capacity");
        winner
            .add_liquid_for_side(second, PumpSide::Second, 2)
            .expect("second source has capacity");

        winner.advance_tick().expect("winning contest succeeds");

        let cell = winner.cell(neutral).expect("coordinate is in bounds");
        assert_eq!(cell.liquid(), 1);
        assert_eq!(cell.liquid_side(), Some(PumpSide::First));
    }

    #[test]
    fn capacity_contention_prioritizes_lower_row_major_sender() {
        let mut board = Board::new(3, 3, 5).expect("valid board dimensions");
        let north = Coordinate::new(1, 0);
        let center = Coordinate::new(1, 1);
        let south = Coordinate::new(1, 2);
        board
            .toggle_pipe_connection(north, center)
            .expect("adjacent cells accept a pipe");
        board
            .toggle_pipe_connection(south, center)
            .expect("adjacent cells accept a pipe");
        board
            .add_liquid_for_side(north, PumpSide::First, 5)
            .expect("north has capacity");
        board
            .add_liquid_for_side(south, PumpSide::First, 5)
            .expect("south has capacity");

        board.advance_tick().expect("tick succeeds");

        assert_eq!(board.cell(north).map(|cell| cell.liquid()), Some(1));
        assert_eq!(board.cell(south).map(|cell| cell.liquid()), Some(4));
        assert_eq!(board.cell(center).map(|cell| cell.liquid()), Some(5));
        assert!(
            board
                .cells
                .iter()
                .all(|cell| cell.liquid() <= cell.capacity())
        );
    }

    #[test]
    fn cells_without_pipes_retain_liquid_and_blocked_cells_stay_empty() {
        let mut board = Board::new(2, 1, 10).expect("valid board dimensions");
        let filled = Coordinate::new(0, 0);
        let blocked = Coordinate::new(1, 0);
        board
            .add_liquid_for_side(filled, PumpSide::First, 6)
            .expect("cell has capacity");
        board.block_cell(blocked).expect("cell is empty");

        board.advance_tick().expect("tick succeeds");

        assert_eq!(board.cell(filled).map(|cell| cell.liquid()), Some(6));
        assert_eq!(board.cell(blocked).map(|cell| cell.liquid()), Some(0));
        assert_eq!(
            board.cell(blocked).map(|cell| cell.liquid_side()),
            Some(None)
        );
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

    #[test]
    fn pumps_can_be_placed_and_counted_by_side() {
        let mut board = Board::new(3, 1, 10).expect("valid board dimensions");
        let first = Coordinate::new(0, 0);
        let second = Coordinate::new(1, 0);
        let other_side = Coordinate::new(2, 0);

        board
            .place_pump(first, PumpSide::First)
            .expect("empty cell accepts a pump");
        board
            .place_pump(second, PumpSide::First)
            .expect("another empty cell accepts a pump");
        board
            .place_pump(other_side, PumpSide::Second)
            .expect("empty cell accepts the other side's pump");

        assert_eq!(board.pump_side(first), Ok(Some(PumpSide::First)));
        assert_eq!(board.pump_side(other_side), Ok(Some(PumpSide::Second)));
        assert_eq!(board.pump_count(PumpSide::First), 2);
        assert_eq!(board.pump_count(PumpSide::Second), 1);
    }

    #[test]
    fn blocked_and_pumped_cells_reject_placement_without_mutation() {
        let mut board = Board::new(2, 1, 10).expect("valid board dimensions");
        let blocked = Coordinate::new(0, 0);
        let occupied = Coordinate::new(1, 0);
        board.block_cell(blocked).expect("cell is empty");
        board
            .place_pump(occupied, PumpSide::First)
            .expect("empty cell accepts a pump");

        assert_eq!(
            board.place_pump(blocked, PumpSide::Second),
            Err(BoardError::Blocked)
        );
        assert_eq!(
            board.place_pump(occupied, PumpSide::Second),
            Err(BoardError::CellContainsPump)
        );
        assert_eq!(board.pump_side(blocked), Ok(None));
        assert_eq!(board.pump_side(occupied), Ok(Some(PumpSide::First)));
        assert_eq!(board.pump_count(PumpSide::First), 1);
        assert_eq!(board.pump_count(PumpSide::Second), 0);
    }

    #[test]
    fn pump_cells_can_have_pipe_connections_and_cannot_be_blocked() {
        let mut board = Board::new(3, 1, 10).expect("valid board dimensions");
        let pump_cell = Coordinate::new(0, 0);
        let neighbor = Coordinate::new(1, 0);
        let isolated_pump = Coordinate::new(2, 0);
        board
            .toggle_pipe_connection(pump_cell, neighbor)
            .expect("open cells accept a pipe");
        board
            .place_pump(pump_cell, PumpSide::Second)
            .expect("pipe connections do not prevent pump placement");
        board
            .place_pump(isolated_pump, PumpSide::First)
            .expect("empty cell accepts a pump");

        assert_eq!(board.connected_pipe_count(pump_cell), Ok(1));
        assert_eq!(board.pump_side(pump_cell), Ok(Some(PumpSide::Second)));
        assert_eq!(
            board.block_cell(isolated_pump),
            Err(BoardError::CellContainsPump)
        );
        assert_eq!(
            board.cell(pump_cell).map(|cell| cell.is_blocked()),
            Some(false)
        );
        assert_eq!(
            board.cell(isolated_pump).map(|cell| cell.is_blocked()),
            Some(false)
        );
        assert_eq!(board.pump_side(pump_cell), Ok(Some(PumpSide::Second)));
        assert_eq!(board.pump_side(isolated_pump), Ok(Some(PumpSide::First)));
    }

    #[test]
    fn pump_placement_and_coordinate_queries_reject_out_of_bounds() {
        let mut board = Board::new(1, 1, 10).expect("valid board dimensions");
        let out_of_bounds = Coordinate::new(1, 0);

        assert_eq!(
            board.place_pump(out_of_bounds, PumpSide::First),
            Err(BoardError::OutOfBounds)
        );
        assert_eq!(board.pump_side(out_of_bounds), Err(BoardError::OutOfBounds));
        assert_eq!(board.pump_count(PumpSide::First), 0);
    }

    #[test]
    fn pipe_queries_and_counts_are_directional() {
        let mut board = Board::new(3, 3, 10).expect("valid board dimensions");
        let left = Coordinate::new(0, 1);
        let right = Coordinate::new(1, 1);
        let above = Coordinate::new(1, 0);

        assert_eq!(board.toggle_pipe_connection(left, right), Ok(true));
        assert_eq!(board.toggle_pipe_connection(right, above), Ok(true));
        assert_eq!(board.has_pipe_connection(left, right), Ok(true));
        assert_eq!(board.has_pipe_connection(right, left), Ok(false));
        assert_eq!(board.has_pipe_connection(right, above), Ok(true));
        assert_eq!(board.has_pipe_connection(above, right), Ok(false));
        assert_eq!(board.connected_pipe_count(left), Ok(1));
        assert_eq!(board.connected_pipe_count(right), Ok(1));
        assert_eq!(board.connected_pipe_count(above), Ok(0));
    }

    #[test]
    fn toggling_an_existing_pipe_removes_it_and_updates_counts() {
        let mut board = Board::new(2, 2, 10).expect("valid board dimensions");
        let center = Coordinate::new(0, 0);
        let east = Coordinate::new(1, 0);
        let south = Coordinate::new(0, 1);

        assert_eq!(board.toggle_pipe_connection(center, east), Ok(true));
        assert_eq!(board.toggle_pipe_connection(center, south), Ok(true));
        assert_eq!(board.connected_pipe_count(center), Ok(2));
        assert_eq!(board.connected_pipe_count(east), Ok(0));

        assert_eq!(board.toggle_pipe_connection(east, center), Ok(false));
        assert_eq!(board.has_pipe_connection(center, east), Ok(false));
        assert_eq!(board.has_pipe_connection(east, center), Ok(false));
        assert_eq!(board.connected_pipe_count(center), Ok(1));
        assert_eq!(board.connected_pipe_count(east), Ok(0));

        assert_eq!(board.toggle_pipe_connection(east, center), Ok(true));
        assert_eq!(board.has_pipe_connection(east, center), Ok(true));
        assert_eq!(board.has_pipe_connection(center, east), Ok(false));
        assert_eq!(board.toggle_pipe_connection(center, east), Ok(false));
        assert_eq!(board.has_pipe_connection(east, center), Ok(false));
    }

    #[test]
    fn invalid_pipe_endpoints_return_errors_without_mutation() {
        let mut board = Board::new(2, 2, 10).expect("valid board dimensions");
        let origin = Coordinate::new(0, 0);
        let identical = origin;
        let diagonal = Coordinate::new(1, 1);
        let distant = Coordinate::new(0, 1);
        let out_of_bounds = Coordinate::new(2, 0);
        board.block_cell(distant).expect("cell is empty");

        assert_eq!(
            board.toggle_pipe_connection(origin, identical),
            Err(BoardError::IdenticalEndpoints)
        );
        assert_eq!(
            board.toggle_pipe_connection(origin, diagonal),
            Err(BoardError::NotAdjacent)
        );
        assert_eq!(
            board.toggle_pipe_connection(origin, out_of_bounds),
            Err(BoardError::OutOfBounds)
        );
        assert_eq!(
            board.toggle_pipe_connection(origin, distant),
            Err(BoardError::Blocked)
        );
        assert_eq!(board.connected_pipe_count(origin), Ok(0));
        assert_eq!(board.connected_pipe_count(diagonal), Ok(0));
        assert_eq!(
            board.has_pipe_connection(origin, diagonal),
            Err(BoardError::NotAdjacent)
        );
    }

    #[test]
    fn cells_with_pipe_connections_cannot_become_blocked() {
        let mut board = Board::new(2, 1, 10).expect("valid board dimensions");
        let first = Coordinate::new(0, 0);
        let second = Coordinate::new(1, 0);
        board
            .toggle_pipe_connection(first, second)
            .expect("cells are adjacent and open");

        assert_eq!(board.block_cell(first), Err(BoardError::CellHasPipes));
        assert_eq!(board.block_cell(second), Err(BoardError::CellHasPipes));
        assert!(
            board
                .has_pipe_connection(first, second)
                .expect("adjacent cells")
        );
        assert!(!board.cell(first).expect("cell exists").is_blocked());
    }

    #[test]
    fn liquid_does_not_flow_against_a_one_way_pipe() {
        let mut board = Board::new(2, 1, 20).expect("valid board dimensions");
        let source = Coordinate::new(0, 0);
        let destination = Coordinate::new(1, 0);
        board
            .toggle_pipe_connection(source, destination)
            .expect("adjacent cells accept a pipe");
        board
            .add_liquid_for_side(destination, PumpSide::Second, 10)
            .expect("destination has capacity");

        board.advance_tick().expect("tick succeeds");

        assert_eq!(board.cell(source).map(|cell| cell.liquid()), Some(0));
        assert_eq!(board.cell(destination).map(|cell| cell.liquid()), Some(10));
    }

    #[test]
    fn pump_fills_its_outgoing_neighbor_over_two_ticks_without_backflow() {
        let mut board = Board::new(2, 1, 20).expect("valid board dimensions");
        let pump = Coordinate::new(0, 0);
        let recipient = Coordinate::new(1, 0);
        board
            .place_pump(pump, PumpSide::Second)
            .expect("empty cell accepts a pump");
        board
            .toggle_pipe_connection(pump, recipient)
            .expect("adjacent cells accept a pipe");

        board.advance_tick().expect("first tick succeeds");
        assert_eq!(board.cell(pump).map(|cell| cell.liquid()), Some(1));
        assert_eq!(board.cell(recipient).map(|cell| cell.liquid()), Some(7));

        board.advance_tick().expect("second tick succeeds");
        assert_eq!(board.cell(pump).map(|cell| cell.liquid()), Some(1));
        assert_eq!(board.cell(recipient).map(|cell| cell.liquid()), Some(15));
        assert_eq!(board.has_pipe_connection(recipient, pump), Ok(false));
    }
}
