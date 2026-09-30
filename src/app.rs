use macroquad::input::{is_key_pressed, is_mouse_button_pressed, mouse_position};
use macroquad::prelude::{KeyCode, MouseButton, next_frame};

use crate::ai::{self, Difficulty as AiDifficulty, PipeAction};
use crate::board::{Board, BoardError, Coordinate, PumpSide};
use crate::render::{self, BoardLayout};

const CELL_CAPACITY: u32 = 160;
const TICK_SECONDS: f32 = 1.0;
const MAX_CATCH_UP_TICKS: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    MainMenu,
    NewGame,
    Playing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    const ALL: [Self; 3] = [Self::Easy, Self::Medium, Self::Hard];

    const fn label(self) -> &'static str {
        match self {
            Self::Easy => "Easy",
            Self::Medium => "Medium",
            Self::Hard => "Hard",
        }
    }

    const fn ai_difficulty(self) -> AiDifficulty {
        match self {
            Self::Easy => AiDifficulty::Easy,
            Self::Medium => AiDifficulty::Medium,
            Self::Hard => AiDifficulty::Hard,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoardSize {
    Small,
    Medium,
    Large,
}

impl BoardSize {
    const ALL: [Self; 3] = [Self::Small, Self::Medium, Self::Large];

    const fn dimensions(self) -> (usize, usize) {
        match self {
            Self::Small => (8, 8),
            Self::Medium => (12, 12),
            Self::Large => (16, 16),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Small => "Small  8 x 8",
            Self::Medium => "Medium  12 x 12",
            Self::Large => "Large  16 x 16",
        }
    }
}

struct App {
    screen: Screen,
    difficulty: Difficulty,
    board_size: BoardSize,
    pumps_per_side: usize,
    board: Option<Board>,
    tick_accumulator: f32,
    status: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            screen: Screen::MainMenu,
            difficulty: Difficulty::Medium,
            board_size: BoardSize::Small,
            pumps_per_side: 2,
            board: None,
            tick_accumulator: 0.0,
            status: String::from("Select a neighbor by clicking toward it."),
        }
    }
}

pub async fn run() {
    let mut app = App::default();
    loop {
        if app.frame() {
            break;
        }
        next_frame().await;
    }
}

impl App {
    fn frame(&mut self) -> bool {
        if is_key_pressed(KeyCode::Escape) {
            match self.screen {
                Screen::MainMenu => return true,
                Screen::NewGame | Screen::Playing => {
                    self.screen = Screen::MainMenu;
                    self.tick_accumulator = 0.0;
                }
            }
        }

        match self.screen {
            Screen::MainMenu => self.main_menu(),
            Screen::NewGame => self.new_game_menu(),
            Screen::Playing => self.playing(),
        }
    }

    fn main_menu(&mut self) -> bool {
        render::draw_menu_background("LIQUID", "A deterministic liquid network prototype");
        let x = render::screen_center_x() - 150.0;
        let y = render::screen_center_y() - 29.0;
        if render::button(render::rect(x, y, 300.0, 54.0), "New Game", false) {
            self.screen = Screen::NewGame;
        }
        render::button(render::rect(x, y + 72.0, 300.0, 54.0), "Exit Game", false)
    }

    fn new_game_menu(&mut self) -> bool {
        render::draw_menu_background("NEW GAME", "Choose difficulty and configure the board");
        let center = render::screen_center_x();
        let y = render::screen_center_y() - 205.0;
        render::draw_section_label("LEVEL", center - 278.0, y);
        let difficulty_width = 176.0;
        let difficulty_gap = 14.0;
        let difficulty_x = center - (difficulty_width * 3.0 + difficulty_gap * 2.0) / 2.0;
        for (index, difficulty) in Difficulty::ALL.into_iter().enumerate() {
            if render::button(
                render::rect(
                    difficulty_x + index as f32 * (difficulty_width + difficulty_gap),
                    y + 24.0,
                    difficulty_width,
                    54.0,
                ),
                difficulty.label(),
                self.difficulty == difficulty,
            ) {
                self.difficulty = difficulty;
            }
        }

        let board_y = y + 98.0;
        render::draw_section_label("BOARD SIZE", center - 278.0, board_y);
        let width = 176.0;
        let gap = 12.0;
        let start_x = center - (width * 3.0 + gap * 2.0) / 2.0;
        for (index, size) in BoardSize::ALL.into_iter().enumerate() {
            if render::button(
                render::rect(
                    start_x + index as f32 * (width + gap),
                    board_y + 24.0,
                    width,
                    54.0,
                ),
                size.label(),
                self.board_size == size,
            ) {
                self.board_size = size;
                self.clamp_pump_count();
            }
        }

        let pump_y = board_y + 96.0;
        render::draw_section_label("PUMPS PER SIDE", center - 278.0, pump_y);
        if render::button(
            render::rect(center - 112.0, pump_y + 28.0, 54.0, 48.0),
            "-",
            false,
        ) {
            self.pumps_per_side = self.pumps_per_side.saturating_sub(1).max(1);
        }
        render::draw_count(self.pumps_per_side, center, pump_y + 39.0);
        if render::button(
            render::rect(center + 58.0, pump_y + 28.0, 54.0, 48.0),
            "+",
            false,
        ) {
            self.pumps_per_side = (self.pumps_per_side + 1).min(self.max_pump_count());
        }
        render::draw_caption(
            &format!("Allowed: 1 to {}", self.max_pump_count()),
            center,
            pump_y + 98.0,
        );

        let action_y = pump_y + 142.0;
        if render::button(
            render::rect(center - 218.0, action_y, 204.0, 52.0),
            "Back",
            false,
        ) {
            self.screen = Screen::MainMenu;
        }
        if render::button(
            render::rect(center + 14.0, action_y, 204.0, 52.0),
            "Start Game",
            true,
        ) {
            match build_board(self.board_size, self.pumps_per_side) {
                Ok(board) => {
                    self.board = Some(board);
                    self.tick_accumulator = 0.0;
                    self.status = String::from("Select a neighbor by clicking toward it.");
                    self.screen = Screen::Playing;
                }
                Err(error) => self.status = format!("Could not create board: {error:?}"),
            }
        }
        false
    }

    fn playing(&mut self) -> bool {
        let Some(board) = self.board.as_mut() else {
            self.screen = Screen::NewGame;
            return false;
        };
        let (width, height) = self.board_size.dimensions();
        let layout = layout_for(board);
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mouse_x, mouse_y) = mouse_position();
            if let Some((position, local_x, local_y)) = layout.cell_at(mouse_x, mouse_y, board)
                && let Some(neighbor) =
                    nearest_valid_neighbor(board, position, local_x, local_y, layout.cell_size)
            {
                match apply_human_action(board, position, neighbor) {
                    Ok(Some(true)) => self.status = String::from("Pipe connected."),
                    Ok(Some(false)) => self.status = String::from("Pipe removed."),
                    Ok(None) => {
                        self.status = String::from("Player pipes must start from red liquid.")
                    }
                    Err(error) => self.status = format!("Pipe change rejected: {error:?}"),
                }
            }
        }

        Self::advance_simulation(
            board,
            &mut self.tick_accumulator,
            &mut self.status,
            macroquad::prelude::get_frame_time(),
            self.difficulty.ai_difficulty(),
        );
        render::draw_board(board, layout);
        let open_menu = render::button(render::rect(24.0, 22.0, 138.0, 42.0), "Menu", false);
        let new_game = render::button(render::rect(174.0, 22.0, 164.0, 42.0), "New Game", true);
        render::draw_hud(
            self.difficulty.label(),
            width,
            height,
            self.pumps_per_side,
            &self.status,
        );
        if open_menu {
            self.screen = Screen::MainMenu;
            self.tick_accumulator = 0.0;
        } else if new_game {
            self.screen = Screen::NewGame;
            self.tick_accumulator = 0.0;
        }
        false
    }

    fn advance_simulation(
        board: &mut Board,
        tick_accumulator: &mut f32,
        status: &mut String,
        frame_seconds: f32,
        difficulty: AiDifficulty,
    ) {
        *tick_accumulator += frame_seconds;
        let mut ticks = 0;
        while *tick_accumulator >= TICK_SECONDS && ticks < MAX_CATCH_UP_TICKS {
            if let Err(error) = advance_one_tick(board, difficulty) {
                *status = format!("Simulation stopped: {error:?}");
                *tick_accumulator = 0.0;
                return;
            }
            *tick_accumulator -= TICK_SECONDS;
            ticks += 1;
        }
        if *tick_accumulator >= TICK_SECONDS {
            *tick_accumulator = 0.0;
        }
    }

    fn max_pump_count(&self) -> usize {
        self.board_size.dimensions().1 - 1
    }

    fn clamp_pump_count(&mut self) {
        self.pumps_per_side = self.pumps_per_side.clamp(1, self.max_pump_count());
    }
}

fn advance_one_tick(
    board: &mut Board,
    difficulty: AiDifficulty,
) -> Result<Option<PipeAction>, BoardError> {
    let action = ai::choose_action(board, difficulty);
    let applied_action = action.filter(|action| ai::apply_action(board, *action));
    board.advance_tick()?;
    Ok(applied_action)
}

fn apply_human_action(
    board: &mut Board,
    source: Coordinate,
    destination: Coordinate,
) -> Result<Option<bool>, BoardError> {
    let Some(source_cell) = board.cell(source) else {
        return Err(BoardError::OutOfBounds);
    };
    if source_cell.liquid() == 0 || source_cell.liquid_side() != Some(PumpSide::First) {
        return Ok(None);
    }
    if board.has_pipe_connection(destination, source)? {
        return Ok(None);
    }
    board.toggle_pipe_connection(source, destination).map(Some)
}

fn build_board(board_size: BoardSize, pumps_per_side: usize) -> Result<Board, BoardError> {
    let (width, height) = board_size.dimensions();
    let mut board = Board::new(width, height, CELL_CAPACITY)?;
    let center_x = width / 2 - 1;
    let center_y = height / 2 - 1;
    for y in center_y..center_y + 2 {
        for x in center_x..center_x + 2 {
            board.block_cell(Coordinate::new(x, y))?;
        }
    }
    for index in 0..pumps_per_side {
        let row = (index + 1) * height / (pumps_per_side + 1);
        board.place_pump(Coordinate::new(0, row), PumpSide::First)?;
        board.place_pump(Coordinate::new(width - 1, row), PumpSide::Second)?;
    }
    Ok(board)
}

fn nearest_valid_neighbor(
    board: &Board,
    origin: Coordinate,
    local_x: f32,
    local_y: f32,
    cell_size: f32,
) -> Option<Coordinate> {
    let candidates = [
        (
            origin
                .y
                .checked_sub(1)
                .map(|y| Coordinate::new(origin.x, y)),
            cell_size / 2.0,
            -cell_size / 2.0,
        ),
        (
            origin
                .x
                .checked_add(1)
                .map(|x| Coordinate::new(x, origin.y)),
            cell_size * 1.5,
            cell_size / 2.0,
        ),
        (
            origin
                .y
                .checked_add(1)
                .map(|y| Coordinate::new(origin.x, y)),
            cell_size / 2.0,
            cell_size * 1.5,
        ),
        (
            origin
                .x
                .checked_sub(1)
                .map(|x| Coordinate::new(x, origin.y)),
            -cell_size / 2.0,
            cell_size / 2.0,
        ),
    ];
    candidates
        .into_iter()
        .filter_map(|(position, target_x, target_y)| {
            position
                .filter(|position| board.cell(*position).is_some_and(|cell| !cell.is_blocked()))
                .map(|position| (position, target_x, target_y))
        })
        .min_by(|(_, first_x, first_y), (_, second_x, second_y)| {
            let first_distance = (local_x - first_x).powi(2) + (local_y - first_y).powi(2);
            let second_distance = (local_x - second_x).powi(2) + (local_y - second_y).powi(2);
            first_distance.total_cmp(&second_distance)
        })
        .map(|(position, _, _)| position)
}

fn layout_for(board: &Board) -> BoardLayout {
    BoardLayout::for_screen(
        board,
        macroquad::prelude::screen_width(),
        macroquad::prelude::screen_height(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BoardSize, advance_one_tick, apply_human_action, build_board, nearest_valid_neighbor,
    };
    use crate::ai::{Difficulty, PipeAction};
    use crate::board::{Board, Coordinate, PumpSide};

    #[test]
    fn initial_board_uses_capacity_obstacles_and_evenly_spaced_pumps() {
        let board = build_board(BoardSize::Small, 2).expect("preset board builds");
        assert_eq!((board.width(), board.height()), (8, 8));
        for y in 0..8 {
            for x in 0..8 {
                let cell = board.cell(Coordinate::new(x, y)).expect("in bounds");
                assert_eq!(cell.capacity(), 160);
                assert_eq!(
                    cell.is_blocked(),
                    (3..=4).contains(&x) && (3..=4).contains(&y)
                );
            }
        }
        assert_eq!(board.pump_count(PumpSide::First), 2);
        assert_eq!(board.pump_count(PumpSide::Second), 2);
        for row in [2, 5] {
            assert_eq!(
                board.pump_side(Coordinate::new(0, row)),
                Ok(Some(PumpSide::First))
            );
            assert_eq!(
                board.pump_side(Coordinate::new(7, row)),
                Ok(Some(PumpSide::Second))
            );
        }
    }

    #[test]
    fn neighbor_selection_uses_north_east_south_west_tie_order_and_skips_blocked() {
        let mut board = Board::new(3, 3, 160).expect("board builds");
        let origin = Coordinate::new(1, 1);
        let north = Coordinate::new(1, 0);
        assert_eq!(
            nearest_valid_neighbor(&board, origin, 16.0, 16.0, 32.0),
            Some(north)
        );

        board.block_cell(north).expect("empty neighbor blocks");
        assert_eq!(
            nearest_valid_neighbor(&board, origin, 16.0, 16.0, 32.0),
            Some(Coordinate::new(2, 1))
        );
    }

    #[test]
    fn neighbor_selection_returns_none_when_all_adjacent_cells_are_invalid() {
        let mut board = Board::new(1, 1, 160).expect("board builds");
        let origin = Coordinate::new(0, 0);
        assert_eq!(
            nearest_valid_neighbor(&board, origin, 16.0, 16.0, 32.0),
            None
        );
        board.block_cell(origin).expect("empty cell blocks");
        assert_eq!(
            nearest_valid_neighbor(&board, origin, 16.0, 16.0, 32.0),
            None
        );
    }

    #[test]
    fn cpu_decision_is_applied_before_flow_on_each_simulation_tick() {
        let mut board = Board::new(3, 1, 160).expect("board builds");
        let blue = Coordinate::new(1, 0);
        let red = Coordinate::new(2, 0);
        board
            .add_liquid_for_side(blue, PumpSide::Second, 10)
            .expect("blue source has capacity");
        board
            .add_liquid_for_side(red, PumpSide::First, 1)
            .expect("red target has capacity");

        assert_eq!(
            advance_one_tick(&mut board, Difficulty::Easy),
            Ok(Some(PipeAction::Add {
                source: blue,
                destination: red,
            }))
        );
        assert_eq!(board.has_pipe_connection(blue, red), Ok(true));
        assert_eq!(
            board.cell(red).map(|cell| cell.liquid_side()),
            Some(Some(PumpSide::Second))
        );
    }

    #[test]
    fn cpu_does_not_act_without_blue_liquid_or_a_valid_target() {
        let mut no_blue = Board::new(2, 1, 160).expect("board builds");
        no_blue
            .add_liquid_for_side(Coordinate::new(0, 0), PumpSide::First, 10)
            .expect("red source has capacity");
        assert_eq!(advance_one_tick(&mut no_blue, Difficulty::Hard), Ok(None));

        let mut no_targets = Board::new(1, 1, 160).expect("board builds");
        no_targets
            .add_liquid_for_side(Coordinate::new(0, 0), PumpSide::Second, 10)
            .expect("blue source has capacity");
        assert_eq!(
            advance_one_tick(&mut no_targets, Difficulty::Hard),
            Ok(None)
        );
        assert_eq!(
            no_targets.connected_pipe_count(Coordinate::new(0, 0)),
            Ok(0)
        );
    }

    #[test]
    fn human_pipe_actions_require_red_liquid_and_cannot_toggle_blue_links() {
        let red = Coordinate::new(0, 0);
        let blue = Coordinate::new(1, 0);
        let mut board = Board::new(2, 1, 160).expect("board builds");
        board
            .add_liquid_for_side(blue, PumpSide::Second, 10)
            .expect("blue source has capacity");
        assert_eq!(apply_human_action(&mut board, blue, red), Ok(None));
        assert_eq!(apply_human_action(&mut board, red, blue), Ok(None));
        assert_eq!(board.has_pipe_connection(blue, red), Ok(false));

        board
            .add_liquid_for_side(red, PumpSide::First, 10)
            .expect("red source has capacity");
        board
            .toggle_pipe_connection(blue, red)
            .expect("blue-origin pipe is valid");
        assert_eq!(apply_human_action(&mut board, red, blue), Ok(None));
        assert_eq!(board.has_pipe_connection(blue, red), Ok(true));

        let mut red_only = Board::new(2, 1, 160).expect("board builds");
        red_only
            .add_liquid_for_side(red, PumpSide::First, 10)
            .expect("red source has capacity");
        assert_eq!(apply_human_action(&mut red_only, red, blue), Ok(Some(true)));
        assert_eq!(
            apply_human_action(&mut red_only, red, blue),
            Ok(Some(false))
        );
    }
}
