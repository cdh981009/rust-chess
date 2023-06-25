use std::fmt;

use ggez::{
    glam::Vec2,
    graphics::{self, Image},
    *,
};

use crate::game::*;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

pub struct Board {
    pieces: Vec<Vec<Option<Piece>>>,
    selected: Option<(usize, usize)>,
    highlight_positions: Vec<(usize, usize)>,
    x: f32,
    y: f32,
    cell_size: f32,
}

impl Board {
    pub fn new((x, y): (f32, f32)) -> Self {
        let config = "rnbqkbnr\
                            pppppppp\
                            --------\
                            --------\
                            --------\
                            --------\
                            PPPPPPPP\
                            RNBQKBNR";

        let mut pieces = vec![vec![None; BOARD_WIDTH]; BOARD_HEIGHT];

        for row in 0..BOARD_HEIGHT {
            for col in 0..BOARD_WIDTH {
                let current = config.as_bytes()[row * BOARD_WIDTH + col] as char;

                let piece_type = match current {
                    '-' => continue,
                    'r' | 'R' => PieceType::Rook,
                    'n' | 'N' => PieceType::Knight,
                    'b' | 'B' => PieceType::Bishop,
                    'q' | 'Q' => PieceType::Queen,
                    'k' | 'K' => PieceType::King,
                    'p' | 'P' => PieceType::Pawn,
                    other => panic!("invalid board configuration: {other}"),
                };

                let color = if current.is_lowercase() {
                    Color::Black
                } else {
                    Color::White
                };

                pieces[row][col] = Some(Piece {
                    piece_type,
                    color,
                    has_moved: false,
                });
            }
        }

        // this is for debug
        for row in &pieces {
            for elem in row {
                print!(
                    "{}",
                    if let Some(piece) = elem {
                        piece.to_string()
                    } else {
                        '_'.to_string()
                    }
                );
            }
            println!("");
        }
        // end

        let selected = None;
        let cell_size = 80.0;

        Board {
            pieces,
            selected,
            x,
            y,
            cell_size,
            highlight_positions: vec![],
        }
    }

    fn is_position_in_bound(&self, (x, y): (i32, i32)) -> bool {
        x >= 0 && x < self.pieces[0].len() as i32 && y >= 0 && y < self.pieces.len() as i32
    }

    fn is_empty(&self, (x, y): (usize, usize)) -> bool {
        self.pieces[y][x].is_none()
    }

    fn is_color(&self, (x, y): (usize, usize), color: Color) -> bool {
        let Some(piece) = &self.pieces[y][x] else { return false; };
        piece.color == color
    }

    fn select_cell(&self, mouse: &Mouse) -> Option<(usize, usize)> {
        let (mx, my) = mouse.get_mouse();
        let (mx_rel, my_rel) = ((mx - self.x).floor(), (my - self.y).floor());

        let x = (mx_rel / self.cell_size) as i32;
        let y = (my_rel / self.cell_size) as i32;

        if self.is_position_in_bound((x, y)) {
            return Some((x as usize, y as usize));
        }

        None
    }

    fn compute_pawn_moves(
        &self,
        piece: &Piece,
        (x, y): (usize, usize),
        moves: &mut Vec<(usize, usize)>,
    ) {
        // pawn move rule:
        // only can move forward. 2 if it's the first time, 1 otherwise.
        // can attack diagonally.

        let y_direction = if piece.color == Color::White { -1 } else { 1 };
        let reach = if piece.has_moved { 1 } else { 2 };

        // move
        for move_y in 1..=reach {
            let (nx, ny) = (x as i32, y as i32 + move_y * y_direction);

            if self.is_position_in_bound((nx, ny)) && self.is_empty((nx as usize, ny as usize)) {
                moves.push((nx as usize, ny as usize));
            } else {
                break;
            }
        }

        // attack
        let enemy_color = if piece.color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        for move_x in [-1, 1] {
            let (nx, ny) = (x as i32 + move_x, y as i32 + y_direction);

            if self.is_position_in_bound((nx, ny))
                && self.is_color((nx as usize, ny as usize), enemy_color)
            {
                moves.push((nx as usize, ny as usize));
            }
        }
    }

    fn compute_knight_moves(
        &self,
        piece: &Piece,
        (x, y): (usize, usize),
        moves: &mut Vec<(usize, usize)>,
    ) {
        static DIRS: [(i32, i32); 8] = [
            (-2, -1),
            (-2, 1),
            (2, -1),
            (2, 1),
            (-1, -2),
            (-1, 2),
            (1, -2),
            (1, 2),
        ];

        let enemy_color = if piece.color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        for (move_x, move_y) in DIRS {
            let (nx, ny) = (x as i32 + move_x, y as i32 + move_y);

            if self.is_position_in_bound((nx, ny))
                && (self.is_empty((nx as usize, ny as usize))
                    || self.is_color((nx as usize, ny as usize), enemy_color))
            {
                moves.push((nx as usize, ny as usize));
            }
        }
    }

    fn compute_bishop_moves(
        &self,
        piece: &Piece,
        (x, y): (usize, usize),
        moves: &mut Vec<(usize, usize)>,
    ) {
        self.compute_diagonal_moves(piece, (x, y), moves)
    }

    fn compute_rook_moves(
        &self,
        piece: &Piece,
        (x, y): (usize, usize),
        moves: &mut Vec<(usize, usize)>,
    ) {
        self.compute_orthogonal_moves(piece, (x, y), moves)
    }

    fn compute_queen_moves(
        &self,
        piece: &Piece,
        (x, y): (usize, usize),
        moves: &mut Vec<(usize, usize)>,
    ) {
        self.compute_diagonal_moves(piece, (x, y), moves);
        self.compute_orthogonal_moves(piece, (x, y), moves);
    }

    fn compute_king_moves(
        &self,
        piece: &Piece,
        (x, y): (usize, usize),
        moves: &mut Vec<(usize, usize)>,
    ) {
        // todo: castling

        let enemy_color = if piece.color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        for move_x in -1..=1 {
            for move_y in -1..=1 {
                if (move_x, move_y) == (0, 0) {
                    continue;
                }

                let (nx, ny) = (x as i32 + move_x, y as i32 + move_y);

                if self.is_position_in_bound((nx, ny))
                    && (self.is_empty((nx as usize, ny as usize))
                        || self.is_color((nx as usize, ny as usize), enemy_color))
                {
                    moves.push((nx as usize, ny as usize));
                }
            }
        }
    }

    fn compute_moves_in_direction(
        &self,
        enemy_color: Color,
        (x, y): (usize, usize),
        (x_dir, y_dir): (i32, i32),
        moves: &mut Vec<(usize, usize)>,
    ) {
        let (mut nx, mut ny) = (x as i32, y as i32);

        loop {
            nx += x_dir;
            ny += y_dir;

            if !self.is_position_in_bound((nx, ny)) {
                break;
            }

            let is_empty = self.is_empty((nx as usize, ny as usize));
            let is_enemy = self.is_color((nx as usize, ny as usize), enemy_color);

            if is_empty || is_enemy {
                moves.push((nx as usize, ny as usize));
            }

            if !is_empty {
                break;
            }
        }
    }

    fn compute_orthogonal_moves(
        &self,
        piece: &Piece,
        (x, y): (usize, usize),
        moves: &mut Vec<(usize, usize)>,
    ) {
        static DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

        let enemy_color = if piece.color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        for (x_dir, y_dir) in DIRS {
            self.compute_moves_in_direction(enemy_color, (x, y), (x_dir, y_dir), moves);
        }
    }

    fn compute_diagonal_moves(
        &self,
        piece: &Piece,
        (x, y): (usize, usize),
        moves: &mut Vec<(usize, usize)>,
    ) {
        let enemy_color = if piece.color == Color::White {
            Color::Black
        } else {
            Color::White
        };

        for x_dir in [-1, 1] {
            for y_dir in [-1, 1] {
                self.compute_moves_in_direction(enemy_color, (x, y), (x_dir, y_dir), moves);
            }
        }
    }

    fn compute_moves_from(&self, (x, y): (usize, usize)) -> Vec<(usize, usize)> {
        let Some(piece) = &self.pieces[y][x] else { return vec![]; };

        use PieceType::*;

        let mut moves = Vec::new();

        match piece.piece_type {
            Pawn => self.compute_pawn_moves(piece, (x, y), &mut moves),
            Knight => self.compute_knight_moves(piece, (x, y), &mut moves),
            Bishop => self.compute_bishop_moves(piece, (x, y), &mut moves),
            Rook => self.compute_rook_moves(piece, (x, y), &mut moves),
            Queen => self.compute_queen_moves(piece, (x, y), &mut moves),
            King => self.compute_king_moves(piece, (x, y), &mut moves),
        }

        moves
    }

    pub fn update(&mut self, mouse: &Mouse) {
        if mouse.is_mouse_pressed() {
            self.selected = self.select_cell(mouse);

            self.highlight_positions = if let Some((x, y)) = self.selected {
                self.compute_moves_from((x, y))
            } else {
                Vec::new()
            };
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
        assets: &mut Assets,
    ) -> GameResult {
        let (x, y) = (self.x, self.y);

        self.draw_board(canvas, (x, y), self.cell_size);
        self.draw_pieces(ctx, canvas, assets, (x, y), self.cell_size);

        Ok(())
    }

    fn draw_board(&self, canvas: &mut graphics::Canvas, (x, y): (f32, f32), cell_size: f32) {
        let light_color = graphics::Color::from_rgb_u32(0x9699A1);
        let dark_color = graphics::Color::from_rgb_u32(0x434347);
        let select_color = graphics::Color::from_rgba_u32(0xFF000066);

        let scale = [cell_size, cell_size];

        for cell_x in 0..BOARD_WIDTH {
            for cell_y in 0..BOARD_HEIGHT {
                let color = if (cell_x + cell_y) % 2 == 0 {
                    light_color
                } else {
                    dark_color
                };

                let pos = [x + cell_size * cell_x as f32, y + cell_size * cell_y as f32];
                let param = graphics::DrawParam::default().scale(scale).dest(pos);

                canvas.draw(&graphics::Quad, param.color(color));

                if self
                    .selected
                    .is_some_and(|(sx, sy)| (sx, sy) == (cell_x, cell_y))
                {
                    canvas.draw(&graphics::Quad, param.color(select_color));
                }
            }
        }

        for (cell_x, cell_y) in self.highlight_positions.iter() {
            let pos = [
                x + cell_size * *cell_x as f32,
                y + cell_size * *cell_y as f32,
            ];
            let param = graphics::DrawParam::default().scale(scale).dest(pos);

            canvas.draw(&graphics::Quad, param.color(select_color));
        }
    }

    fn draw_pieces(
        &self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
        assets: &mut Assets,
        (x, y): (f32, f32),
        cell_size: f32,
    ) {
        let sprite_original_size = 460.0;

        for cell_x in 0..BOARD_WIDTH {
            for cell_y in 0..BOARD_HEIGHT {
                if let Some(piece) = &self.pieces[cell_y][cell_x] {
                    // set pos to the center of the cell
                    let pos: Vec2 =
                        <[f32; 2] as Into<Vec2>>::into([
                            x + cell_size * cell_x as f32,
                            y + cell_size * cell_y as f32,
                        ]) + <[f32; 2] as Into<Vec2>>::into([cell_size / 2.0, cell_size / 2.0]);

                    let image = piece.get_image(ctx, assets);
                    let drawparams = graphics::DrawParam::new()
                        .dest(pos)
                        .offset([0.5, 0.5]) // offset so that the sprite center and the drawing position align
                        .scale([
                            cell_size / sprite_original_size,
                            cell_size / sprite_original_size,
                        ]);
                    canvas.draw(image, drawparams);
                }
            }
        }
    }
}

#[derive(Clone)]
struct Piece {
    piece_type: PieceType,
    color: Color,
    has_moved: bool,
}

impl Piece {
    fn get_image<'a>(&self, ctx: &mut Context, assets: &'a mut Assets) -> &'a Image {
        let sprite = self.color.to_string() + &self.piece_type.to_string();

        assets.try_get_image(ctx, &sprite).unwrap()
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = self.piece_type.to_string();

        write!(
            f,
            "{}",
            if self.color == Color::White {
                c.to_uppercase().to_string()
            } else {
                c.to_string()
            }
        )
    }
}

#[derive(Copy, Clone)]
enum PieceType {
    Pawn,
    Rook,
    Bishop,
    Knight,
    King,
    Queen,
}

impl fmt::Display for PieceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PieceType::*;

        let c = match self {
            Pawn => 'p',
            Rook => 'r',
            Bishop => 'b',
            Knight => 'n',
            King => 'k',
            Queen => 'q',
        };

        write!(f, "{c}")
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Color {
    White,
    Black,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Color::*;

        let c = match self {
            White => 'w',
            Black => 'b',
        };

        write!(f, "{c}")
    }
}
