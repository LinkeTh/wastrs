use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

const ROW_COUNT: i32 = 24;
const COLUMN_COUNT: i32 = 10;

fn create_wall_kicks(table: &[[(i8, i8); 6]]) -> HashMap<(i8, i8), Vec<(i8, i8)>> {
    table
        .iter()
        .map(|row| (row[0], row[1..].to_vec()))
        .collect()
}

lazy_static! {
    static ref WALL_KICKS_I: HashMap<(i8, i8), Vec<(i8, i8)>> = create_wall_kicks(&[
        [(0, 1), (0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
        [(1, 0), (0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        [(1, 2), (0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
        [(2, 1), (0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
        [(2, 3), (0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        [(3, 2), (0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
        [(3, 0), (0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
        [(0, 3), (0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
    ]);
    static ref WALL_KICKS_JLTSZ: HashMap<(i8, i8), Vec<(i8, i8)>> = create_wall_kicks(&[
        [(0, 1), (0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        [(1, 0), (0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        [(1, 2), (0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        [(2, 1), (0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        [(2, 3), (0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        [(3, 2), (0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        [(3, 0), (0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        [(0, 3), (0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
    ]);
}

#[wasm_bindgen]
pub struct Tetris {
    context: CanvasRenderingContext2d,
    board: [[u8; COLUMN_COUNT as usize]; ROW_COUNT as usize],
    current_tetromino: Option<Tetromino>,
    tetromino_x: i32,
    tetromino_y: i32,
    last_update_time: f64,
    speed: f64,
    game_over: bool,
    score: u32,
}

#[wasm_bindgen]
impl Tetris {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Tetris, JsValue> {
        let document = web_sys::window()
            .ok_or_else(|| JsValue::from("No window object"))?
            .document()
            .ok_or_else(|| JsValue::from("No document object"))?;

        let canvas: HtmlCanvasElement = document
            .get_element_by_id(canvas_id)
            .ok_or_else(|| JsValue::from("Canvas element not found"))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| JsValue::from("Failed to cast to HtmlCanvasElement"))?;

        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from("Failed to get 2D context"))?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from("Failed to cast to CanvasRenderingContext2d"))?;

        Ok(Tetris {
            context,
            board: [[0; COLUMN_COUNT as usize]; ROW_COUNT as usize],
            current_tetromino: None,
            tetromino_x: 0,
            tetromino_y: 0,
            last_update_time: 0.0,
            speed: 500.0,
            game_over: false,
            score: 0,
        })
    }

    pub fn render(&mut self) {
        let window = web_sys::window().expect("No access to window object");

        self.context.clear_rect(0.0, 0.0, 300.0, 600.0);

        // skip first 4 rows (those are offscreen for spawning new tetrominos)
        for y in 4..ROW_COUNT {
            for x in 0..COLUMN_COUNT {
                let color = if self.board[y as usize][x as usize] == 0 {
                    "#000000"
                } else {
                    "#F97316"
                };

                self.context.set_fill_style_str(color);
                self.context
                    .fill_rect(x as f64 * 30.0, (y - 4) as f64 * 30.0, 30.0, 30.0);
            }
        }

        if !self.game_over {
            let now = window.performance().expect("No performance API").now();

            if now - self.last_update_time >= self.speed {
                self.move_down();
                self.last_update_time = now;
            }

            self.display_score();

            if let Some(tetromino) = &self.current_tetromino {
                for (tetromino_y, row) in tetromino.shape.iter().enumerate() {
                    for (tetromino_x, &cell) in row.iter().enumerate() {
                        if cell != 0 {
                            let x = self.tetromino_x + tetromino_x as i32;
                            let y = self.tetromino_y + tetromino_y as i32;

                            if (0..COLUMN_COUNT).contains(&x) && (0..ROW_COUNT).contains(&y) {
                                self.context.set_fill_style_str(&tetromino.color);
                                self.context.fill_rect(
                                    x as f64 * 30.0,
                                    (y - 4) as f64 * 30.0,
                                    30.0,
                                    30.0,
                                );
                            }
                        }
                    }
                }
            }
        } else {
            self.display_game_over()
        }
    }

    pub fn move_down(&mut self) {
        if let Some(tetromino) = &self.current_tetromino {
            let new_y = self.tetromino_y + 1;

            if !Self::check_collision(tetromino, &self.board, &new_y, &self.tetromino_x) {
                self.tetromino_y = new_y;
            } else {
                self.lock_tetromino();
                self.spawn_tetromino();
            }
        }
    }

    pub fn move_left(&mut self) {
        if let Some(tetromino) = &self.current_tetromino {
            let new_x = self.tetromino_x - 1;
            if !Self::check_collision(tetromino, &self.board, &self.tetromino_y, &new_x) {
                self.tetromino_x = new_x;
            }
        }
    }

    pub fn move_right(&mut self) {
        if let Some(tetromino) = &self.current_tetromino {
            let new_x = self.tetromino_x + 1;
            if !Self::check_collision(tetromino, &self.board, &self.tetromino_y, &new_x) {
                self.tetromino_x = new_x;
            }
        }
    }

    pub fn start(&mut self) {
        web_sys::console::log_1(&"Start game!".to_string().into());

        self.context.clear_rect(0.0, 0.0, 300.0, 600.0);
        self.board = [[0; COLUMN_COUNT as usize]; ROW_COUNT as usize];
        self.current_tetromino = self.get_random_type();
        self.tetromino_x = 3;
        self.tetromino_y = 1;
        self.game_over = false;
        self.score = 0;
    }

    pub fn is_game_over(&self) -> bool {
        self.game_over
    }

    pub fn rotate_clockwise(&mut self) {
        if let Some(tetromino) = &mut self.current_tetromino {
            let current_rotation = tetromino.state.value();
            let target_rotation = (current_rotation + 1) % 4;
            let transition = (current_rotation as i8, target_rotation as i8);

            if let Some((new_x, new_y)) = Self::try_wall_kick(
                tetromino,
                &self.board,
                &self.tetromino_y,
                &self.tetromino_x,
                transition,
                true,
            ) {
                tetromino.rotate_clockwise();
                self.tetromino_y = new_y;
                self.tetromino_x = new_x;
            }
        }
    }

    pub fn rotate_counterclockwise(&mut self) {
        if let Some(tetromino) = &mut self.current_tetromino {
            let current_rotation = tetromino.state.value();
            let target_rotation = (current_rotation + 3) % 4;
            let transition = (current_rotation as i8, target_rotation as i8);

            if let Some((new_x, new_y)) = Self::try_wall_kick(
                tetromino,
                &self.board,
                &self.tetromino_y,
                &self.tetromino_x,
                transition,
                false,
            ) {
                tetromino.rotate_counterclockwise();
                self.tetromino_y = new_y;
                self.tetromino_x = new_x;
            }
        }
    }

    fn clear_full_lines(&mut self) {
        let mut lines_cleared = 0;
        for y in 4..ROW_COUNT as usize {
            if self.board[y].iter().all(|&cell| cell != 0) {
                lines_cleared += 1;
                for row in (1..=y).rev() {
                    self.board[row] = self.board[row - 1];
                }
            }
        }
        if lines_cleared > 0 {
            let points = lines_cleared * 100;
            self.score += points;
        }
    }

    fn draw_text(&self, text: &str, x: f64, y: f64, font_size: f64, color: &str) {
        self.context.set_font(&format!("{}px Arial", font_size));
        self.context.set_fill_style_str(color);
        self.context
            .fill_text(text, x, y)
            .unwrap_or_else(|e| web_sys::console::warn_1(&e));
    }

    fn display_game_over(&self) {
        self.draw_text("Game Over", 20.0, 300.0, 48.0, "#FF0000");
        self.draw_text("Press a key to restart", 20.0, 350.0, 24.0, "#FFFFFF");
    }

    fn display_score(&self) {
        self.draw_text(
            &format!("Score: {}", self.score),
            10.0,
            30.0,
            20.0,
            "#FFFFFF",
        );
    }

    fn lock_tetromino(&mut self) {
        if let Some(tetromino) = &self.current_tetromino {
            for (tetromino_y, row) in tetromino.shape.iter().enumerate() {
                for (tetromino_x, &cell) in row.iter().enumerate() {
                    if cell != 0 {
                        let x = self.tetromino_x + tetromino_x as i32;
                        let y = self.tetromino_y + tetromino_y as i32;

                        if (4..ROW_COUNT).contains(&y) && (0..COLUMN_COUNT).contains(&x) {
                            self.board[y as usize][x as usize] = cell;
                        }
                    }
                }
            }
            self.clear_full_lines();
            self.current_tetromino = None;
        }
    }

    fn get_random_type(&mut self) -> Option<Tetromino> {
        let mut tetromino_bag: Vec<Tetromino> = vec![
            Tetromino::new(TetrominoType::I),
            Tetromino::new(TetrominoType::J),
            Tetromino::new(TetrominoType::L),
            Tetromino::new(TetrominoType::O),
            Tetromino::new(TetrominoType::S),
            Tetromino::new(TetrominoType::T),
            Tetromino::new(TetrominoType::Z),
        ];
        // shuffle real good
        tetromino_bag.shuffle(&mut thread_rng());
        tetromino_bag.shuffle(&mut thread_rng());
        tetromino_bag.shuffle(&mut thread_rng());
        tetromino_bag.shuffle(&mut thread_rng());

        tetromino_bag.first().cloned()
    }

    fn spawn_tetromino(&mut self) {
        self.current_tetromino = self.get_random_type();
        self.tetromino_x = 3;
        self.tetromino_y = 1;

        if Self::check_collision(
            self.current_tetromino.as_ref().unwrap(),
            &self.board,
            &(&self.tetromino_y + 2),
            &self.tetromino_x,
        ) {
            self.game_over = true;
            self.current_tetromino = None;
            self.tetromino_x = 0;
            self.tetromino_y = 0;
        }
    }

    fn try_wall_kick(
        tetromino: &Tetromino,
        board: &[[u8; COLUMN_COUNT as usize]; ROW_COUNT as usize],
        y_to_check: &i32,
        x_to_check: &i32,
        transition: (i8, i8),
        clockwise: bool,
    ) -> Option<(i32, i32)> {
        let test_xy: &[(i8, i8)] = if tetromino.r#type == TetrominoType::O {
            &[(0, 0)]
        } else if tetromino.r#type == TetrominoType::I {
            WALL_KICKS_I.get(&transition)?
        } else {
            WALL_KICKS_JLTSZ.get(&transition)?
        };

        for &(x, y) in test_xy.iter() {
            let mut copy = tetromino.clone();

            let new_y = y_to_check + y as i32;
            let new_x = x_to_check + x as i32;

            if clockwise {
                copy.rotate_clockwise();
            } else {
                copy.rotate_counterclockwise();
            }

            if !Self::check_collision(&copy, board, &new_y, &new_x) {
                return Some((new_x, new_y));
            }
        }

        None
    }

    fn check_collision(
        tetromino: &Tetromino,
        board: &[[u8; COLUMN_COUNT as usize]; ROW_COUNT as usize],
        y_to_check: &i32,
        x_to_check: &i32,
    ) -> bool {
        let mut board_y: i32;
        let mut board_x: i32;

        for tetromino_y in 0..tetromino.shape.len() {
            for tetromino_x in 0..tetromino.shape[0].len() {
                if tetromino.shape[tetromino_y][tetromino_x] == 0 {
                    continue;
                }
                board_y = tetromino_y as i32 + y_to_check;
                board_x = tetromino_x as i32 + x_to_check;
                if board_x < 0 {
                    return true;
                }
                if board_x > COLUMN_COUNT - 1 {
                    return true;
                }
                if board_y > ROW_COUNT - 1 {
                    return true;
                }

                if board[board_y as usize][board_x as usize] != 0 {
                    return true;
                }
            }
        }

        false
    }
}

#[derive(PartialEq, Copy, Clone)]
enum OrientationState {
    Spawn,
    RotateClockwise,
    RotateCounterClockwise,
    Rotate180,
}

impl OrientationState {
    fn value(&self) -> u8 {
        match self {
            OrientationState::Spawn => 0,
            OrientationState::RotateClockwise => 1,
            OrientationState::Rotate180 => 2,
            OrientationState::RotateCounterClockwise => 3,
        }
    }

    fn next_clockwise(&self) -> OrientationState {
        match self {
            OrientationState::Spawn => OrientationState::RotateClockwise,
            OrientationState::RotateClockwise => OrientationState::Rotate180,
            OrientationState::Rotate180 => OrientationState::RotateCounterClockwise,
            OrientationState::RotateCounterClockwise => OrientationState::Spawn,
        }
    }

    fn next_counterclockwise(&self) -> OrientationState {
        match self {
            OrientationState::Spawn => OrientationState::RotateCounterClockwise,
            OrientationState::RotateCounterClockwise => OrientationState::Rotate180,
            OrientationState::Rotate180 => OrientationState::RotateClockwise,
            OrientationState::RotateClockwise => OrientationState::Spawn,
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
enum TetrominoType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
}

#[derive(PartialEq, Clone)]
struct Tetromino {
    r#type: TetrominoType,
    color: String,
    shape: Vec<Vec<u8>>,
    state: OrientationState,
}

impl Tetromino {
    fn new(r#type: TetrominoType) -> Self {
        match r#type {
            TetrominoType::I => Tetromino {
                r#type: TetrominoType::I,
                color: "#FF0000".to_string(),
                shape: vec![
                    vec![0, 0, 0, 0],
                    vec![1, 1, 1, 1],
                    vec![0, 0, 0, 0],
                    vec![0, 0, 0, 0],
                ],
                state: OrientationState::Spawn,
            },
            TetrominoType::J => Tetromino {
                r#type: TetrominoType::J,
                color: "#FF0000".to_string(),
                shape: vec![vec![1, 0, 0], vec![1, 1, 1], vec![0, 0, 0]],
                state: OrientationState::Spawn,
            },

            TetrominoType::L => Tetromino {
                r#type: TetrominoType::L,
                color: "#00FF00".to_string(),
                shape: vec![vec![0, 0, 1], vec![1, 1, 1], vec![0, 0, 0]],
                state: OrientationState::Spawn,
            },

            TetrominoType::S => Tetromino {
                r#type: TetrominoType::S,
                color: "#0000FF".to_string(),
                shape: vec![vec![0, 1, 1], vec![1, 1, 0], vec![0, 0, 0]],
                state: OrientationState::Spawn,
            },

            TetrominoType::Z => Tetromino {
                r#type: TetrominoType::Z,
                color: "#FFFF00".to_string(),
                shape: vec![vec![1, 1, 0], vec![0, 1, 1], vec![0, 0, 0]],
                state: OrientationState::Spawn,
            },

            TetrominoType::O => Tetromino {
                r#type: TetrominoType::O,
                color: "#FF00FF".to_string(),
                shape: vec![vec![1, 1], vec![1, 1]],
                state: OrientationState::Spawn,
            },

            TetrominoType::T => Tetromino {
                r#type: TetrominoType::T,
                color: "#00FFFF".to_string(),
                shape: vec![vec![0, 1, 0], vec![1, 1, 1], vec![0, 0, 0]],
                state: OrientationState::Spawn,
            },
        }
    }

    fn rotate_clockwise(&mut self) {
        let mut rotated = vec![vec![0; self.shape.len()]; self.shape[0].len()];

        for y in 0..self.shape.len() {
            for x in 0..self.shape[0].len() {
                let new_y = x;
                let new_x = self.shape.len() - 1 - y;
                rotated[new_y][new_x] = self.shape[y][x];
            }
        }

        self.state = self.state.next_clockwise();
        self.shape = rotated;
    }

    fn rotate_counterclockwise(&mut self) {
        let mut rotated = vec![vec![0; self.shape.len()]; self.shape[0].len()];

        for y in 0..self.shape.len() {
            for x in 0..self.shape[0].len() {
                let new_y = self.shape[0].len() - 1 - x;
                let new_x = y;
                rotated[new_y][new_x] = self.shape[y][x];
            }
        }

        self.state = self.state.next_counterclockwise();
        self.shape = rotated;
    }
}
