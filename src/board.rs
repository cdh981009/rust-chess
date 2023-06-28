use std::{fmt, mem::swap};

use ggez::{
    glam::{vec2, Vec2},
    graphics::{self, Image, TextLayout, TextAlign},
    *,
};

use crate::{game::*, WINDOW_WIDTH};
use crate::move_calculator;

pub const BOARD_WIDTH: usize = 8;
pub const BOARD_HEIGHT: usize = 8;
pub const BOARD_SIZE_1D: usize = BOARD_WIDTH * BOARD_HEIGHT;

#[derive(PartialEq)]
enum TurnState {
    Normal,
    Check,
}

pub struct Board {
    // fields for game logic
    board_state: [Option<Piece>; BOARD_SIZE_1D],
    selected_cell: Option<(usize, usize)>,
    legal_moves: [[bool; BOARD_SIZE_1D]; BOARD_SIZE_1D],
    is_movable: [bool; BOARD_SIZE_1D],
    current_turn_color: PieceColor,
    turn_state: TurnState,
    is_moves_computed: bool,

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
            is_movable: [false; BOARD_SIZE_1D],
            current_turn_color: PieceColor::White,
            turn_state: TurnState::Normal,
            is_moves_computed: false,

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
        let setup: &[u8] = "rnbqkbnr\
                            pppppppp\
                            --------\
                            --------\
                            --------\
                            --------\
                            PPPPPPPP\
                            RNBQKBNR"
            .as_bytes();

        for (pos, curr) in setup.iter().enumerate() {
            let curr = *curr as char;

            let piece_type = match curr {
                '-' => continue,
                'r' | 'R' => PieceType::Rook,
                'n' | 'N' => PieceType::Knight,
                'b' | 'B' => PieceType::Bishop,
                'q' | 'Q' => PieceType::Queen,
                'k' | 'K' => PieceType::King,
                'p' | 'P' => PieceType::Pawn,
                other => panic!("invalid board configuration: {other}"),
            };

            let color = if curr.is_lowercase() {
                PieceColor::Black
            } else {
                PieceColor::White
            };

            self.board_state[pos] = Some(Piece {
                piece_type,
                color,
                has_moved: false,
            });
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

    // move piece and return the piece previously on the destination
    fn move_piece(&mut self, from: (usize, usize), to: (usize, usize)) -> Option<Piece> {
        let from_1d = Board::to_index1d(from);
        let to_1d = Board::to_index1d(to);

        let mut src = self.board_state[from_1d];
        let dst = self.board_state[to_1d];

        let Some(src_piece) = &mut src else { panic!("{:?} should contain a piece", from) };
        src_piece.has_moved = true;

        self.board_state[from_1d] = None;
        self.board_state[to_1d] = src;

        dst
    }

    // compute and populate each piece's legal moves
    fn compute_each_legal_moves(&mut self) {
        // reset previous legal moves
        self.legal_moves = [[false; BOARD_SIZE_1D]; BOARD_SIZE_1D];

        // iterate each piece of current turn and compute its legal moves
        for pos_1d in 0..BOARD_SIZE_1D {
            let pos_2d = Board::to_index2d(pos_1d);

            if !Board::is_color_on(&self.board_state, pos_2d, self.current_turn_color) {
                continue;
            }

            move_calculator::get_moves(&self.board_state, pos_2d, &mut self.legal_moves[pos_1d]);

            self.eliminate_illegal_moves(pos_1d);
        }
    }

    fn eliminate_illegal_moves(&mut self, src_pos_1d: usize) {
        let mut moves = self.legal_moves[src_pos_1d];

        for dst_pos_1d in 0..BOARD_SIZE_1D {
            if !moves[dst_pos_1d] {
                continue;
            }

            let from = Board::to_index2d(src_pos_1d);
            let to = Board::to_index2d(dst_pos_1d);

            // temporarily move the piece to the destination
            let original_src = self.board_state[src_pos_1d];
            let original_dst = self.move_piece(from, to);

            if self.is_in_check(self.current_turn_color) {
                moves[dst_pos_1d] = false;
            }

            // recover the original state
            self.board_state[src_pos_1d] = original_src;
            self.board_state[dst_pos_1d] = original_dst;
        }

        self.legal_moves[src_pos_1d] = moves;
    }

    fn is_in_check(&self, color: PieceColor) -> bool {
        let mut kings_position: Option<usize> = None;

        // find king of the given color
        for pos_1d in 0..BOARD_SIZE_1D {
            if self.board_state[pos_1d]
                .is_some_and(|piece| piece.color == color && piece.piece_type == PieceType::King)
            {
                kings_position = Some(pos_1d);
                break;
            }
        }

        let kings_position = kings_position.expect("king not found in the board");
        let enemy_color = color.get_enemy_color();

        // get all psuedo-legal moves of the enemy (moves that doesn't consider check)
        let mut enemy_moves = [false; BOARD_SIZE_1D];

        for pos_1d in 0..BOARD_SIZE_1D {
            let Some(piece) = self.board_state[pos_1d] else { continue };
            if piece.color != enemy_color {
                continue;
            };

            move_calculator::get_moves(
                &self.board_state,
                Board::to_index2d(pos_1d),
                &mut enemy_moves,
            );
        }

        enemy_moves[kings_position]
    }

    fn change_turn(&mut self) {
        self.current_turn_color = self.current_turn_color.get_enemy_color();
        self.selected_cell = None;

        self.turn_state = if self.is_in_check(self.current_turn_color) {
            TurnState::Check
        } else {
            TurnState::Normal
        };

        self.is_moves_computed = false;
    }

    fn compute_is_movable(&mut self) {
        for (pos, is_movable) in self.is_movable.iter_mut().enumerate() {
            *is_movable = self.legal_moves[pos].contains(&true);
        }
    }

    pub fn update(&mut self, mouse: &Mouse) {
        if !self.is_moves_computed {
            self.compute_each_legal_moves();
            self.compute_is_movable();
            self.is_moves_computed = true;

            // if no legal moves for all pieces
            //      if inCheck
            //          then checkmate -> current color loses
            //      else
            //          then stalemate -> draw

            if !self.is_movable.contains(&true) {
                // no legal moves
                if self.turn_state == TurnState::Check {
                    // checkmate
                } else {
                    // stalemate
                }
            }
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
            } else {
                self.selected_cell = None;
            }
        }
    }

    pub fn draw(
        &self,
        ctx: &mut Context,
        canvas: &mut graphics::Canvas,
        assets: &mut Assets,
    ) -> GameResult {
        self.draw_turn_state(canvas);
        self.draw_board(canvas, self.position, self.cell_size);
        self.draw_pieces(ctx, canvas, assets, self.position, self.cell_size);

        Ok(())
    }

    fn draw_turn_state(&self, canvas: &mut graphics::Canvas) {
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

        let check_text = graphics::Text::new("Check")
            .set_scale(32.)
            .set_layout(TextLayout { // right align
                h_align: TextAlign::End,
                v_align: TextAlign::Begin,
            })
            .clone();

        canvas.draw(
            &turn_text,
            graphics::DrawParam::from(vec2(15., 15.)).color(graphics::Color::from((0, 0, 0, 255))),
        );

        if self.turn_state == TurnState::Check {
            canvas.draw(
                &check_text,
                graphics::DrawParam::from(vec2(WINDOW_WIDTH - 15., 15.)).color(graphics::Color::from((0, 0, 0, 255))),
            );
        }
    }

    fn draw_board(&self, canvas: &mut graphics::Canvas, pos: Vec2, cell_size: f32) {
        let light_color = graphics::Color::from_rgb_u32(0x9699A1);
        let dark_color = graphics::Color::from_rgb_u32(0x434347);
        let select_color = graphics::Color::from_rgba_u32(0xFF000066);
        let movable_color = graphics::Color::from_rgba_u32(0x00FF0023);

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

                let is_movable_piece = self.is_movable[Board::to_index1d((cell_x, cell_y))];

                if is_selected_cell || is_movable_cell {
                    canvas.draw(&graphics::Quad, param.color(select_color));
                } else if is_movable_piece {
                    canvas.draw(&graphics::Quad, param.color(movable_color));
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

#[derive(PartialEq, Copy, Clone)]
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

impl PieceColor {
    pub fn get_enemy_color(&self) -> PieceColor {
        match *self {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
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
