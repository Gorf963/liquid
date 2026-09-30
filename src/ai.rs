//! Deterministic blue-side planning using territory pressure.
//!
//! Search budgets are bounded by at most 1,024 candidate evaluations per
//! board position (the directed-neighbor maximum on a 16x16 board): Easy
//! evaluates up to 1,024 candidates at depth 1, Medium up to 3,072 at depth
//! 2 with beam width 2, and Hard up to 9,216 at depth 3 with beam width 4.
//! Lookahead advances ticks without simulating human moves; each later beam
//! node may also pass for that tick. Territory pressure is provisional because
//! the game has no explicit victory condition.

use crate::board::{Board, Coordinate, PumpSide};
use std::cmp::Reverse;

const MAX_CANDIDATES_PER_POSITION: usize = 16 * 16 * 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

/// A directed CPU pipe command. The source is always blue-owned when returned
/// by [`choose_action`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipeAction {
    Add {
        source: Coordinate,
        destination: Coordinate,
    },
    Remove {
        source: Coordinate,
        destination: Coordinate,
    },
}

#[derive(Debug, Clone, Copy)]
struct SearchBudget {
    depth: usize,
    beam_width: usize,
    max_candidate_evaluations: usize,
}

impl SearchBudget {
    const EASY: Self = Self::new(1, 1);
    const MEDIUM: Self = Self::new(2, 2);
    const HARD: Self = Self::new(3, 4);

    const fn new(depth: usize, beam_width: usize) -> Self {
        Self {
            depth,
            beam_width,
            max_candidate_evaluations: MAX_CANDIDATES_PER_POSITION * (1 + (depth - 1) * beam_width),
        }
    }
}

fn search_budget(difficulty: Difficulty) -> SearchBudget {
    match difficulty {
        Difficulty::Easy => SearchBudget::EASY,
        Difficulty::Medium => SearchBudget::MEDIUM,
        Difficulty::Hard => SearchBudget::HARD,
    }
}

#[derive(Clone)]
struct SearchNode {
    board: Board,
    pressure: i64,
    first_action: PipeAction,
}

/// Returns the best improving blue pipe action for this board, if one exists.
///
/// Actions are ordered deterministically by row-major source, then North,
/// East, South, West destination order. Ties preserve that order.
pub fn choose_action(board: &Board, difficulty: Difficulty) -> Option<PipeAction> {
    let budget = search_budget(difficulty);
    let actions = candidate_actions(board);
    if actions.is_empty() {
        return None;
    }

    let mut baseline = board.clone();
    for _ in 0..budget.depth {
        baseline.advance_tick().ok()?;
    }
    let baseline_pressure = territory_pressure(&baseline);

    let mut evaluated_candidates = 0;
    let mut beam = Vec::with_capacity(budget.beam_width);
    for action in actions {
        if evaluated_candidates >= budget.max_candidate_evaluations {
            break;
        }
        evaluated_candidates += 1;
        let Some(next_board) = transition(board, Some(action)) else {
            continue;
        };
        beam.push(SearchNode {
            pressure: territory_pressure(&next_board),
            board: next_board,
            first_action: action,
        });
    }
    if beam.is_empty() {
        return None;
    }
    retain_best(&mut beam, budget.beam_width);

    for _ in 1..budget.depth {
        let mut expanded = Vec::new();
        'beam: for node in &beam {
            for action in candidate_actions(&node.board) {
                if evaluated_candidates >= budget.max_candidate_evaluations {
                    break 'beam;
                }
                evaluated_candidates += 1;
                let Some(next_board) = transition(&node.board, Some(action)) else {
                    continue;
                };
                expanded.push(SearchNode {
                    pressure: territory_pressure(&next_board),
                    board: next_board,
                    first_action: node.first_action,
                });
            }

            if let Some(waited_board) = transition(&node.board, None) {
                expanded.push(SearchNode {
                    pressure: territory_pressure(&waited_board),
                    board: waited_board,
                    first_action: node.first_action,
                });
            }
        }
        if expanded.is_empty() {
            break;
        }
        retain_best(&mut expanded, budget.beam_width);
        beam = expanded;
    }

    beam.first()
        .filter(|node| node.pressure > baseline_pressure)
        .map(|node| node.first_action)
}

fn candidate_actions(board: &Board) -> Vec<PipeAction> {
    let mut actions = Vec::with_capacity(MAX_CANDIDATES_PER_POSITION);
    'cells: for y in 0..board.height() {
        for x in 0..board.width() {
            let source = Coordinate::new(x, y);
            let Some(source_cell) = board.cell(source) else {
                continue;
            };
            if source_cell.is_blocked()
                || source_cell.liquid() == 0
                || source_cell.liquid_side() != Some(PumpSide::Second)
            {
                continue;
            }

            let neighbors = [
                y.checked_sub(1)
                    .map(|neighbor_y| Coordinate::new(x, neighbor_y)),
                x.checked_add(1)
                    .filter(|neighbor_x| *neighbor_x < board.width())
                    .map(|neighbor_x| Coordinate::new(neighbor_x, y)),
                y.checked_add(1)
                    .filter(|neighbor_y| *neighbor_y < board.height())
                    .map(|neighbor_y| Coordinate::new(x, neighbor_y)),
                x.checked_sub(1)
                    .map(|neighbor_x| Coordinate::new(neighbor_x, y)),
            ];

            for destination in neighbors.into_iter().flatten() {
                let Some(destination_cell) = board.cell(destination) else {
                    continue;
                };
                if destination_cell.is_blocked() {
                    continue;
                }

                let has_forward_link = board.has_pipe_connection(source, destination) == Ok(true);
                let has_reverse_link = board.has_pipe_connection(destination, source) == Ok(true);
                let action = if has_forward_link {
                    Some(PipeAction::Remove {
                        source,
                        destination,
                    })
                } else if !has_reverse_link {
                    Some(PipeAction::Add {
                        source,
                        destination,
                    })
                } else {
                    None
                };
                if let Some(action) = action {
                    actions.push(action);
                    if actions.len() == MAX_CANDIDATES_PER_POSITION {
                        break 'cells;
                    }
                }
            }
        }
    }
    actions
}

fn transition(board: &Board, action: Option<PipeAction>) -> Option<Board> {
    let mut next_board = board.clone();
    if let Some(action) = action
        && !apply_action(&mut next_board, action)
    {
        return None;
    }
    next_board.advance_tick().ok()?;
    Some(next_board)
}

pub(crate) fn apply_action(board: &mut Board, action: PipeAction) -> bool {
    let (source, destination) = match action {
        PipeAction::Add {
            source,
            destination,
        } => {
            if !board.cell(source).is_some_and(|cell| {
                cell.liquid() > 0 && cell.liquid_side() == Some(PumpSide::Second)
            }) || board.cell(destination).is_none_or(|cell| cell.is_blocked())
                || board.has_pipe_connection(source, destination) != Ok(false)
                || board.has_pipe_connection(destination, source) != Ok(false)
            {
                return false;
            }
            (source, destination)
        }
        PipeAction::Remove {
            source,
            destination,
        } => {
            if !board.cell(source).is_some_and(|cell| {
                cell.liquid() > 0 && cell.liquid_side() == Some(PumpSide::Second)
            }) || board.cell(destination).is_none_or(|cell| cell.is_blocked())
                || board.has_pipe_connection(source, destination) != Ok(true)
                || board.has_pipe_connection(destination, source) != Ok(false)
            {
                return false;
            }
            (source, destination)
        }
    };

    board
        .toggle_pipe_connection(source, destination)
        .is_ok_and(|added| matches!(action, PipeAction::Add { .. }) == added)
}

fn territory_pressure(board: &Board) -> i64 {
    let mut pressure = 0;
    for y in 0..board.height() {
        for x in 0..board.width() {
            let Some(cell) = board.cell(Coordinate::new(x, y)) else {
                continue;
            };
            if cell.liquid() == 0 {
                continue;
            }
            pressure += match cell.liquid_side() {
                Some(PumpSide::Second) => 2,
                Some(PumpSide::First) => -1,
                None => 0,
            };
        }
    }
    pressure
}

fn retain_best(nodes: &mut Vec<SearchNode>, beam_width: usize) {
    nodes.sort_by_key(|node| Reverse(node.pressure));
    nodes.truncate(beam_width);
}

#[cfg(test)]
mod tests {
    use super::{
        Difficulty, PipeAction, SearchBudget, candidate_actions, choose_action, territory_pressure,
        transition,
    };
    use crate::board::{Board, Coordinate, PumpSide};

    #[test]
    fn immediate_capture_ranks_above_neutral_expansion() {
        let mut board = Board::new(3, 1, 160).expect("board builds");
        let blue = Coordinate::new(1, 0);
        let neutral = Coordinate::new(0, 0);
        let red = Coordinate::new(2, 0);
        board
            .add_liquid_for_side(blue, PumpSide::Second, 10)
            .expect("blue source has capacity");
        board
            .add_liquid_for_side(red, PumpSide::First, 1)
            .expect("red cell has capacity");

        let neutral_pressure = territory_pressure(
            &transition(
                &board,
                Some(PipeAction::Add {
                    source: blue,
                    destination: neutral,
                }),
            )
            .expect("neutral expansion advances one tick"),
        );
        let red_pressure = territory_pressure(
            &transition(
                &board,
                Some(PipeAction::Add {
                    source: blue,
                    destination: red,
                }),
            )
            .expect("red capture advances one tick"),
        );
        assert!(red_pressure > neutral_pressure);
        assert_eq!(
            choose_action(&board, Difficulty::Easy),
            Some(PipeAction::Add {
                source: blue,
                destination: red,
            })
        );
    }

    #[test]
    fn identical_board_and_difficulty_produce_identical_actions() {
        let mut board = Board::new(3, 1, 160).expect("board builds");
        let blue = Coordinate::new(1, 0);
        board
            .add_liquid_for_side(blue, PumpSide::Second, 10)
            .expect("blue source has capacity");
        board
            .add_liquid_for_side(Coordinate::new(2, 0), PumpSide::First, 1)
            .expect("red cell has capacity");

        for difficulty in [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard] {
            assert_eq!(
                choose_action(&board, difficulty),
                choose_action(&board, difficulty)
            );
        }
    }

    #[test]
    fn planner_returns_none_when_there_are_no_blue_cells_or_targets() {
        let mut no_blue = Board::new(2, 1, 160).expect("board builds");
        no_blue
            .add_liquid_for_side(Coordinate::new(0, 0), PumpSide::First, 10)
            .expect("red cell has capacity");
        assert_eq!(choose_action(&no_blue, Difficulty::Hard), None);

        let mut no_targets = Board::new(1, 1, 160).expect("board builds");
        no_targets
            .add_liquid_for_side(Coordinate::new(0, 0), PumpSide::Second, 10)
            .expect("blue cell has capacity");
        assert_eq!(choose_action(&no_targets, Difficulty::Hard), None);
    }

    #[test]
    fn planner_can_remove_a_blue_link_to_concentrate_flow_into_red() {
        let mut board = Board::new(3, 1, 160).expect("board builds");
        let blue = Coordinate::new(1, 0);
        let neutral = Coordinate::new(0, 0);
        let red = Coordinate::new(2, 0);
        board
            .add_liquid_for_side(blue, PumpSide::Second, 10)
            .expect("blue source has capacity");
        board
            .add_liquid_for_side(red, PumpSide::First, 6)
            .expect("red cell has capacity");
        board
            .toggle_pipe_connection(blue, neutral)
            .expect("blue link is valid");
        board
            .toggle_pipe_connection(blue, red)
            .expect("blue link is valid");

        assert_eq!(
            choose_action(&board, Difficulty::Easy),
            Some(PipeAction::Remove {
                source: blue,
                destination: neutral,
            })
        );
    }

    #[test]
    fn reverse_red_link_is_neither_removed_nor_replaced_by_a_second_link() {
        let mut board = Board::new(3, 1, 160).expect("board builds");
        let red = Coordinate::new(0, 0);
        let blue = Coordinate::new(1, 0);
        board
            .add_liquid_for_side(blue, PumpSide::Second, 10)
            .expect("blue source has capacity");
        board
            .add_liquid_for_side(red, PumpSide::First, 1)
            .expect("red source has capacity");
        board
            .toggle_pipe_connection(red, blue)
            .expect("red-origin link is valid");

        let actions = candidate_actions(&board);
        assert!(!actions.contains(&PipeAction::Add {
            source: blue,
            destination: red,
        }));
        assert!(!actions.contains(&PipeAction::Remove {
            source: blue,
            destination: red,
        }));
    }

    #[test]
    fn search_budgets_strictly_increase_by_difficulty() {
        let easy = SearchBudget::EASY;
        let medium = SearchBudget::MEDIUM;
        let hard = SearchBudget::HARD;

        assert!(easy.depth < medium.depth && medium.depth < hard.depth);
        assert!(easy.beam_width < medium.beam_width && medium.beam_width < hard.beam_width);
        assert!(
            easy.max_candidate_evaluations < medium.max_candidate_evaluations
                && medium.max_candidate_evaluations < hard.max_candidate_evaluations
        );
        assert_eq!(easy.max_candidate_evaluations, 1_024);
        assert_eq!(medium.max_candidate_evaluations, 3_072);
        assert_eq!(hard.max_candidate_evaluations, 9_216);
    }
}
