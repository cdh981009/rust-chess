use std::{fmt, mem::swap};

use ggez::{
    glam::Vec2,
    graphics::{self, Image},
    *,
};

use crate::game::*;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

pub struct Board {
    pieces: [Option<Piece>; BOARD_WIDTH * BOARD_HEIGHT],
    selected: Option<(usize, usize)>,
    highlight_positions: Vec<(usize, usize)>,
    position: Vec2,
    cell_size: f32,
}

impl Board {
    pub fn new(position: Vec2) -> Self {
        let cell_size = 80.0;

        Board {
            pieces: [None; BOARD_WIDTH * BOARD_HEIGHT],
            selected: None,
            position,
            cell_size,
            highlight_positions: vec![],
        }
    }

    fn to_index1d((x, y): (usize, usize)) -> usize {
        y * BOARD_WIDTH + x
    }

    fn to_index2d(ind: usize) -> (usize, usize) {
        (ind % BOARD_WIDTH, ind / BOARD_WIDTH)
    }

    fn print(&self) {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                print!(
                    "{}",
                    if let Some(piece) = &self.pieces[Board::to_index1d((x, y))] {
                        piece.to_string()
                    } else {
                        '_'.to_string()
                    }
                );
            }
            println!();
        }
    }

    pub fn init(mut self) -> Self {
        let config: &[u8] = "rnbqkbnr\
                             pppppppp\
                             --------\
                             --------\
                             --------\
                             --------\
                             PPPPPPPP\
                             RNBQKBNR"
            .as_bytes();

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let current = config[Board::to_index1d((x, y))] as char;

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

                self.pieces[Board::to_index1d((x, y))] = Some(Piece {
                    piece_type,
                    color,
                    has_moved: false,
                });
            }
        }

        self.print();

        self
    }

    fn is_position_in_bound((x, y): (i32, i32)) -> bool {
        x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32
    }

    fn is_empty_on(&self, pos: (usize, usize)) -> bool {
        self.pieces[Board::to_index1d(pos)].is_none()
    }

    fn is_color_on(&self, pos: (usize, usize), color: Color) -> bool {
        let Some(piece) = &self.pieces[Board::to_index1d(pos)] else { return false; };
        piece.color == color
    }

    fn try_select_cell(&self, mouse: &Mouse) -> Option<(usize, usize)> {
        let m_pos = mouse.get_mouse();
        let m_cell = ((m_pos - self.position) / self.cell_size).floor();

        let (x, y) = (m_cell.x as i32, m_cell.y as i32);

        if Board::is_position_in_bound((x, y)) {
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

            if Board::is_position_in_bound((nx, ny)) && self.is_empty_on((nx as usize, ny as usize)) {
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

            if Board::is_position_in_bound((nx, ny))
                && self.is_color_on((nx as usize, ny as usize), enemy_color)
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

            if Board::is_position_in_bound((nx, ny))
                && (self.is_empty_on((nx as usize, ny as usize))
                    || self.is_color_on((nx as usize, ny as usize), enemy_color))
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

                if Board::is_position_in_bound((nx, ny))
                    && (self.is_empty_on((nx as usize, ny as usize))
                        || self.is_color_on((nx as usize, ny as usize), enemy_color))
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

            if !Board::is_position_in_bound((nx, ny)) {
                break;
            }

            let is_empty = self.is_empty_on((nx as usize, ny as usize));
            let is_enemy = self.is_color_on((nx as usize, ny as usize), enemy_color);

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

    fn compute_moves_from(&self, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let Some(piece) = &self.pieces[Board::to_index1d(pos)] else { return vec![]; };

        use PieceType::*;

        let mut moves = Vec::new();

        match piece.piece_type {
            Pawn => self.compute_pawn_moves(piece, pos, &mut moves),
            Knight => self.compute_knight_moves(piece, pos, &mut moves),
            Bishop => self.compute_bishop_moves(piece, pos, &mut moves),
            Rook => self.compute_rook_moves(piece, pos, &mut moves),
            Queen => self.compute_queen_moves(piece, pos, &mut moves),
            King => self.compute_king_moves(piece, pos, &mut moves),
        }

        moves
    }

    fn move_piece(&mut self, from: (usize, usize), to: (usize, usize)) {
        let mut src = self.pieces[Board::to_index1d(from)];

        let Some(src_piece) = &mut src else { panic!("{:?} should contain a piece", from) };
        src_piece.has_moved = true;

        self.pieces[Board::to_index1d(from)] = None;
        self.pieces[Board::to_index1d(to)] = src;
    }

    pub fn update(&mut self, mouse: &Mouse) {
        if mouse.is_mouse_pressed(event::MouseButton::Left) {
            let selected_cell = self.try_select_cell(mouse);

            if let Some((x, y)) = selected_cell {
                // valid cell is selected
                if self.selected.is_some() && self.highlight_positions.contains(&(x, y)) {
                    self.move_piece(self.selected.unwrap(), (x, y));
                    self.highlight_positions = Vec::new();
                    self.selected = None;
                } else {
                    self.highlight_positions = self.compute_moves_from((x, y));
                    self.selected = selected_cell;
                }
            } else {
                self.highlight_positions = Vec::new();
            };
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
        assets: &mut Assets,
    ) -> GameResult {
        self.draw_board(canvas, self.position, self.cell_size);
        self.draw_pieces(ctx, canvas, assets, self.position, self.cell_size);

        Ok(())
    }

    fn draw_board(&self, canvas: &mut graphics::Canvas, pos: Vec2, cell_size: f32) {
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

                let cell_pos: Vec2 =
                    pos + Vec2::new(cell_size * cell_x as f32, cell_size * cell_y as f32);
                let param = graphics::DrawParam::default().scale(scale).dest(cell_pos);

                // draw checker pattern
                canvas.draw(&graphics::Quad, param.color(color));

                // draw transparent highlight on selected cell
                if self
                    .selected
                    .is_some_and(|(sx, sy)| (sx, sy) == (cell_x, cell_y))
                {
                    canvas.draw(&graphics::Quad, param.color(select_color));
                }
            }
        }

        // draw highlight on current movable cells
        for (cell_x, cell_y) in self.highlight_positions.iter() {
            let cell_pos: Vec2 =
                pos + Vec2::new(cell_size * *cell_x as f32, cell_size * *cell_y as f32);
            let param = graphics::DrawParam::default().scale(scale).dest(cell_pos);

            canvas.draw(&graphics::Quad, param.color(select_color));
        }
    }

    fn draw_pieces(
        &self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
        assets: &mut Assets,
        pos: Vec2,
        cell_size: f32,
    ) {
        let sprite_original_size = 460.0;

        for cell_x in 0..BOARD_WIDTH {
            for cell_y in 0..BOARD_HEIGHT {
                if let Some(piece) = &self.pieces[Board::to_index1d((cell_x, cell_y))] {
                    // set pos to the center of the cell
                    let cell_pos: Vec2 =
                        pos + Vec2::new(cell_size * cell_x as f32, cell_size * cell_y as f32);
                    let cell_pos_centered = cell_pos + Vec2::new(cell_size / 2.0, cell_size / 2.0);

                    let image = piece.get_image(ctx, assets);
                    let drawparams = graphics::DrawParam::new()
                        .dest(cell_pos_centered)
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

#[derive(Copy, Clone)]
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
                c.to_uppercase()
            } else {
                c
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
