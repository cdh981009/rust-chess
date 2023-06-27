use std::{fmt, mem::swap};

use ggez::{
    glam::{vec2, Vec2},
    graphics::{self, Image},
    *,
};

use crate::game::*;
use crate::move_calculator;

pub const BOARD_WIDTH: usize = 8;
pub const BOARD_HEIGHT: usize = 8;
pub const BOARD_SIZE_1D: usize = BOARD_WIDTH * BOARD_HEIGHT;

#[derive(PartialEq)]
enum TurnState {
    ComputeMoves,
    Normal,
    Check,
}

pub struct Board {
    // fields for game logic
    board_state: [Option<Piece>; BOARD_SIZE_1D],
    selected_cell: Option<(usize, usize)>,
    legal_moves: [[bool; BOARD_SIZE_1D]; BOARD_SIZE_1D],
    current_turn_color: PieceColor,
    turn_state: TurnState,

    // fields for drawing
    position: Vec2,
    cell_size: f32,
}

impl Board {
    pub fn new(position: Vec2) -> Self {
        let cell_size = 80.0;

        Board {
            board_state: [None; BOARD_SIZE_1D],
            selected_cell: None,
            legal_moves: [[false; BOARD_SIZE_1D]; BOARD_SIZE_1D],
            current_turn_color: PieceColor::White,
            turn_state: TurnState::ComputeMoves,

            position,
            cell_size,
        }
    }

    pub fn to_index1d((x, y): (usize, usize)) -> usize {
        y * BOARD_WIDTH + x
    }

    pub fn to_index2d(ind: usize) -> (usize, usize) {
        (ind % BOARD_WIDTH, ind / BOARD_WIDTH)
    }

    fn print(&self) {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                print!(
                    "{}",
                    if let Some(piece) = &self.board_state[Board::to_index1d((x, y))] {
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
                    PieceColor::Black
                } else {
                    PieceColor::White
                };

                self.board_state[Board::to_index1d((x, y))] = Some(Piece {
                    piece_type,
                    color,
                    has_moved: false,
                });
            }
        }

        self.print();

        self
    }

    pub fn is_position_in_bound((x, y): (i32, i32)) -> bool {
        x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32
    }

    pub fn is_empty_on(board_state: &[Option<Piece>; BOARD_SIZE_1D], pos: (usize, usize)) -> bool {
        board_state[Board::to_index1d(pos)].is_none()
    }

    pub fn is_color_on(
        board_state: &[Option<Piece>; BOARD_SIZE_1D],
        pos: (usize, usize),
        color: PieceColor,
    ) -> bool {
        let Some(piece) = board_state[Board::to_index1d(pos)] else { return false; };
        piece.color == color
    }

    pub fn get_piece(&self, pos: (usize, usize)) -> &Option<Piece> {
        &self.board_state[Board::to_index1d(pos)]
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

    fn move_piece(&mut self, from: (usize, usize), to: (usize, usize)) {
        let mut src = self.board_state[Board::to_index1d(from)];

        let Some(src_piece) = &mut src else { panic!("{:?} should contain a piece", from) };
        src_piece.has_moved = true;

        self.board_state[Board::to_index1d(from)] = None;
        self.board_state[Board::to_index1d(to)] = src;
    }

    // compute and populate each piece's legal moves
    fn compute_each_moves(&mut self) {
        // reset previous legal moves
        self.legal_moves = [[false; BOARD_SIZE_1D]; BOARD_SIZE_1D];

        // iterate each piece of current turn and compute its legal moves
        for pos in 0..BOARD_SIZE_1D {
            let pos_2d = Board::to_index2d(pos);

            if !Board::is_color_on(&self.board_state, pos_2d, self.current_turn_color) {
                continue;
            }

            move_calculator::get_moves_of(&self.board_state, pos_2d, &mut self.legal_moves[pos]);

            // TODO: simulate every possible move and see if it causes check (put the king in danger)
        }
    }

    fn change_turn(&mut self) {
        self.current_turn_color = if self.current_turn_color == PieceColor::White {
            PieceColor::Black
        } else {
            PieceColor::White
        };

        self.turn_state = TurnState::ComputeMoves;
        self.selected_cell = None;
    }

    pub fn update(&mut self, mouse: &Mouse) {
        if self.turn_state == TurnState::ComputeMoves {
            self.compute_each_moves();

            // if no legal moves for all pieces
            //      if inCheck
            //          then checkmate -> current color loses
            //      else
            //          then stalemate -> draw
            // else if there exists some legal move
            //      self.turn_state = TurnState::Normal

            self.turn_state = TurnState::Normal;
        }

        if mouse.is_mouse_pressed(event::MouseButton::Left) {
            let cell = self.try_select_cell(mouse);

            if let Some(cell_position) = cell {
                let is_movable = self.selected_cell.is_some_and(|selected_piece| {
                    self.legal_moves[Board::to_index1d(selected_piece)]
                        [Board::to_index1d(cell_position)]
                });

                if is_movable {
                    // when there's a selected piece and newly-selected cell is one of it's possible moves
                    // move the piece and change the turn
                    self.move_piece(self.selected_cell.unwrap(), cell_position);
                    self.change_turn();
                } else {
                    // select new piece on this cell
                    self.selected_cell = cell;
                }
            }
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
        assets: &mut Assets,
    ) -> GameResult {
        self.draw_turn(canvas);
        self.draw_board(canvas, self.position, self.cell_size);
        self.draw_pieces(ctx, canvas, assets, self.position, self.cell_size);

        Ok(())
    }

    fn draw_turn(&self, canvas: &mut graphics::Canvas) {
        let turn_text = graphics::Text::new(format!(
            "{}'s turn",
            if self.current_turn_color == PieceColor::White {
                "White"
            } else {
                "Black"
            }
        ))
        //.set_font("LiberationMono")
        .set_scale(32.)
        .clone();

        canvas.draw(
            &turn_text,
            graphics::DrawParam::from(vec2(15., 15.)).color(graphics::Color::from((0, 0, 0, 255))),
        );
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
                    pos + vec2(cell_size * cell_x as f32, cell_size * cell_y as f32);
                let param = graphics::DrawParam::default().scale(scale).dest(cell_pos);

                // draw checker pattern
                canvas.draw(&graphics::Quad, param.color(color));

                // draw transparent highlight on the selected cell and its movable cells
                let is_selected_cell = self
                    .selected_cell
                    .is_some_and(|(sx, sy)| (sx, sy) == (cell_x, cell_y));

                let is_movable_cell = self.selected_cell.is_some_and(|selected| {
                    self.legal_moves[Board::to_index1d(selected)]
                        [Board::to_index1d((cell_x, cell_y))]
                });

                if is_selected_cell || is_movable_cell {
                    canvas.draw(&graphics::Quad, param.color(select_color));
                }
            }
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
                if let Some(piece) = &self.board_state[Board::to_index1d((cell_x, cell_y))] {
                    // set pos to the center of the cell
                    let cell_pos = pos + vec2(cell_size * cell_x as f32, cell_size * cell_y as f32);
                    let cell_pos_centered = cell_pos + vec2(cell_size / 2.0, cell_size / 2.0);

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
pub struct Piece {
    piece_type: PieceType,
    color: PieceColor,
    has_moved: bool,
}

impl Piece {
    fn get_image<'a>(&self, ctx: &mut Context, assets: &'a mut Assets) -> &'a Image {
        let sprite = self.color.to_string() + &self.piece_type.to_string();

        assets.try_get_image(ctx, &sprite).unwrap()
    }

    pub fn get_piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn get_color(&self) -> PieceColor {
        self.color
    }

    pub fn has_moved(&self) -> bool {
        self.has_moved
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c = self.piece_type.to_string();

        write!(
            f,
            "{}",
            if self.color == PieceColor::White {
                c.to_uppercase()
            } else {
                c
            }
        )
    }
}

#[derive(Copy, Clone)]
pub enum PieceType {
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
pub enum PieceColor {
    White,
    Black,
}

impl fmt::Display for PieceColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PieceColor::*;

        let c = match self {
            White => 'w',
            Black => 'b',
        };

        write!(f, "{c}")
    }
}
