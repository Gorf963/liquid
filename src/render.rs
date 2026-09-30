use macroquad::prelude::{
    Color, Rect, WHITE, clear_background, draw_circle_lines, draw_line, draw_rectangle,
    draw_rectangle_lines, draw_text, measure_text, mouse_position, screen_height, screen_width,
};

use crate::board::{Board, Coordinate, PumpSide};

const BACKGROUND: Color = Color::new(0.055, 0.075, 0.082, 1.0);
const PANEL: Color = Color::new(0.09, 0.12, 0.13, 1.0);
const CELL: Color = Color::new(0.17, 0.22, 0.22, 1.0);
const GRID: Color = Color::new(0.28, 0.35, 0.34, 1.0);
const FIRST: Color = Color::new(0.96, 0.31, 0.23, 1.0);
const SECOND: Color = Color::new(0.18, 0.72, 0.86, 1.0);
const ACCENT: Color = Color::new(0.88, 0.81, 0.49, 1.0);
const TEXT: Color = Color::new(0.91, 0.94, 0.91, 1.0);
const MUTED: Color = Color::new(0.58, 0.67, 0.65, 1.0);
const LIQUID_DISPLAY_CAPACITY: u32 = 180;

#[derive(Debug, Clone, Copy)]
pub struct BoardLayout {
    pub x: f32,
    pub y: f32,
    pub cell_size: f32,
}

impl BoardLayout {
    pub fn for_screen(board: &Board, screen_width: f32, screen_height: f32) -> Self {
        let available_width = (screen_width - 64.0).max(1.0);
        let available_height = (screen_height - 174.0).max(1.0);
        let cell_size = (available_width / board.width() as f32)
            .min(available_height / board.height() as f32)
            .max(1.0);
        let board_width = cell_size * board.width() as f32;
        let board_height = cell_size * board.height() as f32;
        Self {
            x: (screen_width - board_width) / 2.0,
            y: 104.0 + (available_height - board_height).max(0.0) / 2.0,
            cell_size,
        }
    }

    pub fn cell_at(
        self,
        screen_x: f32,
        screen_y: f32,
        board: &Board,
    ) -> Option<(Coordinate, f32, f32)> {
        let local_x = screen_x - self.x;
        let local_y = screen_y - self.y;
        if local_x < 0.0 || local_y < 0.0 {
            return None;
        }
        let x = (local_x / self.cell_size) as usize;
        let y = (local_y / self.cell_size) as usize;
        let position = Coordinate::new(x, y);
        board.cell(position)?;
        Some((
            position,
            local_x - x as f32 * self.cell_size,
            local_y - y as f32 * self.cell_size,
        ))
    }
}

pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect::new(x, y, width, height)
}

pub fn screen_center_x() -> f32 {
    screen_width() / 2.0
}

pub fn screen_center_y() -> f32 {
    screen_height() / 2.0
}

pub fn draw_menu_background(title: &str, subtitle: &str) {
    clear_background(BACKGROUND);
    draw_rectangle(0.0, 0.0, screen_width(), 7.0, ACCENT);
    draw_text(title, 40.0, 64.0, 34.0, TEXT);
    draw_text(subtitle, 42.0, 91.0, 16.0, MUTED);
}

pub fn draw_section_label(label: &str, x: f32, y: f32) {
    draw_text(label, x, y, 15.0, MUTED);
}

pub fn draw_caption(label: &str, center_x: f32, y: f32) {
    let dimensions = measure_text(label, None, 15, 1.0);
    draw_text(label, center_x - dimensions.width / 2.0, y, 15.0, MUTED);
}

pub fn draw_count(value: usize, x: f32, y: f32) {
    let label = value.to_string();
    let dimensions = measure_text(&label, None, 25, 1.0);
    draw_text(&label, x - dimensions.width / 2.0, y, 25.0, TEXT);
}

pub fn button(bounds: Rect, label: &str, selected: bool) -> bool {
    let (mouse_x, mouse_y) = mouse_position();
    let hovered = bounds.contains(macroquad::prelude::vec2(mouse_x, mouse_y));
    let fill = if selected {
        Color::new(0.25, 0.33, 0.29, 1.0)
    } else if hovered {
        Color::new(0.19, 0.27, 0.26, 1.0)
    } else {
        PANEL
    };
    draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, fill);
    draw_rectangle_lines(
        bounds.x,
        bounds.y,
        bounds.w,
        bounds.h,
        1.0,
        if selected { ACCENT } else { GRID },
    );
    let font_size = 18;
    let dimensions = measure_text(label, None, font_size, 1.0);
    draw_text(
        label,
        bounds.x + (bounds.w - dimensions.width) / 2.0,
        bounds.y + (bounds.h + dimensions.height) / 2.0 - 3.0,
        font_size as f32,
        TEXT,
    );
    hovered && macroquad::prelude::is_mouse_button_pressed(macroquad::prelude::MouseButton::Left)
}

pub fn draw_hud(difficulty: &str, width: usize, height: usize, pumps: usize, status: &str) {
    draw_text(
        format!("{difficulty}  |  {width} x {height}  |  {pumps} pumps per side  |  1 tick / sec"),
        356.0,
        48.0,
        18.0,
        TEXT,
    );
    draw_text(status, 28.0, screen_height() - 20.0, 15.0, MUTED);
    let player = "PLAYER (RED)";
    let cpu = "CPU (BLUE)";
    let font_size = 14;
    let cpu_width = measure_text(cpu, None, font_size, 1.0).width;
    let player_width = measure_text(player, None, font_size, 1.0).width;
    let cpu_x = screen_width() - 28.0 - cpu_width;
    let player_x = cpu_x - 18.0 - player_width;
    draw_text(
        player,
        player_x,
        screen_height() - 25.0,
        font_size as f32,
        FIRST,
    );
    draw_text(cpu, cpu_x, screen_height() - 25.0, font_size as f32, SECOND);
}

pub fn draw_board(board: &Board, layout: BoardLayout) {
    clear_background(BACKGROUND);
    for y in 0..board.height() {
        for x in 0..board.width() {
            let position = Coordinate::new(x, y);
            let Some(cell) = board.cell(position) else {
                continue;
            };
            let left = layout.x + x as f32 * layout.cell_size;
            let top = layout.y + y as f32 * layout.cell_size;
            let padding = (layout.cell_size * 0.035).max(0.5);
            let bounds = rect(
                left + padding,
                top + padding,
                (layout.cell_size - padding * 2.0).max(0.0),
                (layout.cell_size - padding * 2.0).max(0.0),
            );
            let fill = if cell.is_blocked() {
                Color::new(0.015, 0.02, 0.02, 1.0)
            } else {
                CELL
            };
            draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, fill);
            if !cell.is_blocked() {
                if let Some(side) = cell.liquid_side().filter(|_| cell.liquid() > 0) {
                    let liquid_size = liquid_square_size(layout.cell_size, cell.liquid());
                    draw_rectangle(
                        left + (layout.cell_size - liquid_size) / 2.0,
                        top + (layout.cell_size - liquid_size) / 2.0,
                        liquid_size,
                        liquid_size,
                        side_color(side),
                    );
                }
                draw_rectangle_lines(bounds.x, bounds.y, bounds.w, bounds.h, 1.0, GRID);
            }
        }
    }

    for y in 0..board.height() {
        for x in 0..board.width() {
            let position = Coordinate::new(x, y);
            let Some(cell) = board.cell(position) else {
                continue;
            };
            if cell.is_blocked() {
                continue;
            }
            for (neighbor, horizontal, toward_positive) in [
                (
                    y.checked_sub(1)
                        .map(|neighbor_y| Coordinate::new(x, neighbor_y)),
                    false,
                    false,
                ),
                (
                    (x + 1 < board.width()).then_some(Coordinate::new(x + 1, y)),
                    true,
                    true,
                ),
                (
                    (y + 1 < board.height()).then_some(Coordinate::new(x, y + 1)),
                    false,
                    true,
                ),
                (
                    x.checked_sub(1)
                        .map(|neighbor_x| Coordinate::new(neighbor_x, y)),
                    true,
                    false,
                ),
            ]
            .into_iter()
            .filter_map(|(neighbor, horizontal, toward_positive)| {
                neighbor.map(|neighbor| (neighbor, horizontal, toward_positive))
            }) {
                if board.has_pipe_connection(position, neighbor) == Ok(true) {
                    let (start_x, start_y, end_x, end_y) = pipe_connector_points(
                        layout.x + x as f32 * layout.cell_size,
                        layout.y + y as f32 * layout.cell_size,
                        layout.cell_size,
                        horizontal,
                        toward_positive,
                    );
                    let pipe_width = (layout.cell_size * 0.13).clamp(2.0, 8.0);
                    draw_line(start_x, start_y, end_x, end_y, pipe_width + 2.0, BACKGROUND);
                    draw_line(start_x, start_y, end_x, end_y, pipe_width, ACCENT);
                }
            }
        }
    }

    for y in 0..board.height() {
        for x in 0..board.width() {
            let position = Coordinate::new(x, y);
            let Some(cell) = board.cell(position) else {
                continue;
            };
            if cell.is_blocked() {
                continue;
            }
            let center_x = layout.x + (x as f32 + 0.5) * layout.cell_size;
            let center_y = layout.y + (y as f32 + 0.5) * layout.cell_size;
            if let Some(side) = cell.pump_side() {
                let radius = (layout.cell_size * 0.26).clamp(4.0, 15.0);
                draw_circle_lines(center_x, center_y, radius, 2.5, side_color(side));
                draw_circle_lines(center_x, center_y, (radius - 3.0).max(1.0), 1.0, WHITE);
            } else if cell.liquid() > 0 && layout.cell_size >= 27.0 {
                let label = cell.liquid().to_string();
                let size = (layout.cell_size * 0.34).clamp(9.0, 15.0) as u16;
                let text_size = measure_text(&label, None, size, 1.0);
                if liquid_square_size(layout.cell_size, cell.liquid()) >= text_size.width + 4.0 {
                    draw_text(
                        &label,
                        center_x - text_size.width / 2.0,
                        center_y + text_size.height / 2.0,
                        f32::from(size),
                        TEXT,
                    );
                }
            }
        }
    }
}

fn liquid_square_size(cell_size: f32, amount: u32) -> f32 {
    let area_fraction = amount.min(LIQUID_DISPLAY_CAPACITY) as f32 / LIQUID_DISPLAY_CAPACITY as f32;
    cell_size * area_fraction.sqrt()
}

fn pipe_connector_points(
    x: f32,
    y: f32,
    cell_size: f32,
    horizontal: bool,
    toward_positive: bool,
) -> (f32, f32, f32, f32) {
    let center_x = x + cell_size / 2.0;
    let center_y = y + cell_size / 2.0;
    if horizontal {
        (
            center_x,
            center_y,
            if toward_positive { x + cell_size } else { x },
            center_y,
        )
    } else {
        (
            center_x,
            center_y,
            center_x,
            if toward_positive { y + cell_size } else { y },
        )
    }
}

const fn side_color(side: PumpSide) -> Color {
    match side {
        PumpSide::First => FIRST,
        PumpSide::Second => SECOND,
    }
}

#[cfg(test)]
mod tests {
    use super::{liquid_square_size, pipe_connector_points};

    #[test]
    fn pipe_connector_runs_from_source_center_to_shared_edge_in_all_directions() {
        assert_eq!(
            pipe_connector_points(10.0, 20.0, 100.0, true, true),
            (60.0, 70.0, 110.0, 70.0)
        );
        assert_eq!(
            pipe_connector_points(10.0, 20.0, 100.0, true, false),
            (60.0, 70.0, 10.0, 70.0)
        );
        assert_eq!(
            pipe_connector_points(10.0, 20.0, 100.0, false, false),
            (60.0, 70.0, 60.0, 20.0)
        );
        assert_eq!(
            pipe_connector_points(10.0, 20.0, 100.0, false, true),
            (60.0, 70.0, 60.0, 120.0)
        );
    }

    #[test]
    fn liquid_square_area_matches_amount_out_of_180() {
        let cell_size = 100.0;
        let square_size = liquid_square_size(cell_size, 16);
        let area_fraction = (square_size * square_size) / (cell_size * cell_size);

        assert!((area_fraction - 16.0 / 180.0).abs() < 0.0001);
    }

    #[test]
    fn liquid_square_is_empty_at_zero_and_capped_at_display_capacity() {
        assert_eq!(liquid_square_size(100.0, 0), 0.0);
        assert_eq!(liquid_square_size(100.0, 180), 100.0);
        assert_eq!(liquid_square_size(100.0, 200), 100.0);
    }
}
