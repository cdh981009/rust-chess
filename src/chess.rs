use std::iter;

use ggez::{
    glam::{vec2, Vec2},
    graphics::{self, TextAlign, TextLayout},
    *,
};

use crate::move_calculator;
use crate::piece::*;
use crate::{game::*, WINDOW_WIDTH};

pub const BOARD_WIDTH: usize = 8;
pub const BOARD_HEIGHT: usize = 8;
pub const BOARD_SIZE_1D: usize = BOARD_WIDTH * BOARD_HEIGHT;

pub type Board<T> = [[T; BOARD_HEIGHT]; BOARD_WIDTH];

#[derive(PartialEq)]
enum TurnState {
    Normal,
    Check,
    Checkmate,
    Stalemate,
}

pub struct Chess {
    // fields for game logic
    board: Board<Option<Piece>>,
    selected_cell: Option<(usize, usize)>,
    legal_moves: Board<Board<bool>>,
    is_movable: Board<bool>,
    current_turn_color: PieceColor,
    turn_state: TurnState,
    is_moves_computed: bool,

    // fields for drawing
    position: Vec2,
    cell_size: f32,
}

impl Chess {
    pub fn new(position: Vec2) -> Self {
        let cell_size = 80.0;

        Chess {
            board: [[None; BOARD_HEIGHT]; BOARD_WIDTH],
            selected_cell: None,
            legal_moves: [[[[false; BOARD_HEIGHT]; BOARD_WIDTH]; BOARD_HEIGHT]; BOARD_WIDTH],
            is_movable: [[false; BOARD_HEIGHT]; BOARD_WIDTH],
            current_turn_color: PieceColor::White,
            turn_state: TurnState::Normal,
            is_moves_computed: false,

            position,
            cell_size,
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

        for (ind, curr) in setup.iter().enumerate() {
            let curr = *curr as char;

            let piece_type = match curr {
                '-' => continue,
                'r' | 'R' => PieceType::Rook,
                'n' | 'N' => PieceType::Knight,
                'b' | 'B' => PieceType::Bishop,
                'q' | 'Q' => PieceType::Queen,
                'k' | 'K' => PieceType::King,
                'p' | 'P' => PieceType::Pawn { en_passant: false },
                other => panic!("invalid board configuration: {other}"),
            };

            let color = if curr.is_lowercase() {
                PieceColor::Black
            } else {
                PieceColor::White
            };

            let (x, y) = Chess::to_index_2d(ind);
            self.board[x][y] = Some(Piece::new(piece_type, color));
        }

        self.print();

        self
    }

    fn print(&self) {
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                print!(
                    "{}",
                    if let Some(piece) = &self.board[x][y] {
                        piece.to_string()
                    } else {
                        '_'.to_string()
                    }
                );
            }
            println!();
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

            if !self.is_movable.iter().any(|row| row.contains(&true)) {
                self.turn_state = if self.turn_state == TurnState::Check {
                    TurnState::Checkmate
                } else {
                    TurnState::Stalemate
                };
            }

            return;
        }

        if mouse.is_mouse_pressed(event::MouseButton::Left) {
            let cell = self.try_select_cell(mouse);

            if let Some((cell_x, cell_y)) = cell {
                let is_movable = self.selected_cell.is_some_and(|(piece_x, piece_y)| {
                    self.legal_moves[piece_x][piece_y][cell_x][cell_y]
                });

                if is_movable {
                    // when there's a selected piece and newly-selected cell is one of it's possible moves
                    // move the piece and change the turn
                    self.move_piece(self.selected_cell.unwrap(), (cell_x, cell_y));
                    self.post_move_update();
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

    pub fn to_index_1d((x, y): (usize, usize)) -> usize {
        y * BOARD_WIDTH + x
    }

    pub fn to_index_2d(ind: usize) -> (usize, usize) {
        (ind % BOARD_WIDTH, ind / BOARD_WIDTH)
    }

    pub fn is_position_in_bound((x, y): (i32, i32)) -> bool {
        x >= 0 && x < BOARD_WIDTH as i32 && y >= 0 && y < BOARD_HEIGHT as i32
    }

    pub fn is_empty_on(board_state: &Board<Option<Piece>>, (x, y): (usize, usize)) -> bool {
        board_state[x][y].is_none()
    }

    pub fn is_color_on(
        board_state: &Board<Option<Piece>>,
        (x, y): (usize, usize),
        color: PieceColor,
    ) -> bool {
        let Some(piece) = board_state[x][y] else { return false; };
        piece.get_color() == color
    }

    fn try_select_cell(&self, mouse: &Mouse) -> Option<(usize, usize)> {
        let m_pos = mouse.get_mouse();
        let cell = ((m_pos - self.position) / self.cell_size).floor();

        let (x, y) = (cell.x as i32, cell.y as i32);

        if Chess::is_position_in_bound((x, y)) {
            return Some((x as usize, y as usize));
        }

        None
    }

    fn move_piece(&mut self, from: (usize, usize), to: (usize, usize)) {
        let mut src = self.board[from.0][from.1];

        let Some(src_piece) = &mut src else { panic!("{:?} should contain a piece", from) };

        src_piece.set_has_moved(true);

        let mut attacking_position = to;

        // handle special moves
        self.move_en_passant(from, to, src_piece, &mut attacking_position);
        self.move_castling(from, to, src_piece);

        self.board[from.0][from.1] = None;
        self.board[attacking_position.0][attacking_position.1] = None;
        self.board[to.0][to.1] = src;
    }

    fn move_castling(&mut self, from: (usize, usize), to: (usize, usize), src_piece: &mut Piece) {
        let is_castling =
            (src_piece.get_piece_type() == PieceType::King) && (from.0.abs_diff(to.0) == 2);

        if !is_castling {
            return;
        };

        let rook_x = if to.0 > from.0 { BOARD_WIDTH - 1 } else { 0 };
        let rook_new_x = if to.0 > from.0 { to.0 - 1 } else { to.0 + 1 };

        let mut rook = self.board[rook_x][from.1].expect("cell should not be empty");
        self.board[rook_x][from.1] = None;

        rook.set_has_moved(true);
        self.board[rook_new_x][from.1] = Some(rook);
    }

    fn move_en_passant(
        &mut self,
        from: (usize, usize),
        to: (usize, usize),
        src_piece: &mut Piece,
        attacking_position: &mut (usize, usize),
    ) {
        let PieceType::Pawn { en_passant } = src_piece.get_piece_type_mut() else { return };

        if from.0 != to.0 {
            // if the pawn moves diagonally, check for en passant attack
            let en_passant_target = self.board[to.0][from.1];

            if en_passant_target.is_some_and(|enemy| {
                enemy.get_color() != src_piece.get_color()
                    && matches!(enemy.get_piece_type(), PieceType::Pawn { en_passant: true })
            }) {
                *attacking_position = (to.0, from.1);
            }
        } else if from.1.abs_diff(to.1) == 2 {
            // if the pawn moves 2 cells vertically
            //      then it's its first move
            //           enable en passant for the next turn
            *en_passant = true;
        }
    }

    // compute and populate each piece's legal moves
    fn compute_each_legal_moves(&mut self) {
        // reset previous legal moves
        self.legal_moves = [[[[false; BOARD_HEIGHT]; BOARD_WIDTH]; BOARD_HEIGHT]; BOARD_WIDTH];

        // iterate each piece of current turn and compute its legal moves
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if !Chess::is_color_on(&self.board, (x, y), self.current_turn_color) {
                    continue;
                }

                move_calculator::get_pseudo_legal_moves(
                    &self.board,
                    (x, y),
                    &mut self.legal_moves[x][y],
                );

                self.eliminate_illegal_moves((x, y));
            }
        }
    }

    fn eliminate_illegal_moves(&mut self, from: (usize, usize)) {
        let mut moves = self.legal_moves[from.0][from.1];
        let board_saved = self.board;

        for to_x in 0..BOARD_WIDTH {
            for to_y in 0..BOARD_HEIGHT {
                let to = (to_x, to_y);

                if !moves[to.0][to.1] {
                    continue;
                }

                // temporarily move the piece to the destination
                self.move_piece(from, to);

                if self.is_in_check(self.current_turn_color) {
                    moves[to.0][to.1] = false;
                }

                // recover the original state
                self.board = board_saved;
            }
        }

        // special case: check illegal moves for castling
        self.eliminate_castling_illegal_moves(from, &mut moves, &board_saved);

        self.legal_moves[from.0][from.1] = moves;
    }

    fn eliminate_castling_illegal_moves(
        &mut self,
        from: (usize, usize),
        moves: &mut Board<bool>,
        board_saved: &Board<Option<Piece>>,
    ) {
        // legal castling condition:
        // A player may not castle out of, through, or into check.

        if self.board[from.0][from.1].unwrap().get_piece_type() != PieceType::King {
            return;
        }

        for x_dir in [-1, 1] {
            let castling_dst = (from.0 as i32 + x_dir * 2, from.1 as i32);

            if !Chess::is_position_in_bound(castling_dst) {
                continue;
            };

            let castling = &mut moves[castling_dst.0 as usize][castling_dst.1 as usize];

            if !*castling {
                continue;
            };

            // move the king one cell at a time towards the castling destination
            // and see if it's in check
            for x in 0..2 {
                let to = ((from.0 as i32 + x_dir * x) as usize, from.1);

                self.move_piece(from, to);

                if self.is_in_check(self.current_turn_color) {
                    *castling = false;
                }

                self.board = *board_saved;
            }
        }
    }

    fn is_in_check(&self, color: PieceColor) -> bool {
        let mut kings_position: Option<(usize, usize)> = None;

        // find king of the given color
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                if self.board[x][y].is_some_and(|piece| {
                    piece.get_color() == color && piece.get_piece_type() == PieceType::King
                }) {
                    kings_position = Some((x, y));
                    break;
                }
            }
        }

        let kings_position = kings_position.expect("king not found in the board");
        let enemy_color = color.get_enemy_color();

        let enemy_attacks = move_calculator::get_all_attacks(&self.board, enemy_color);

        enemy_attacks[kings_position.0][kings_position.1]
    }

    fn post_move_update(&mut self) {
        for x in 0..BOARD_WIDTH {
            for y in 0..BOARD_HEIGHT {
                let Some(piece) = &mut self.board[x][y] else { continue };

                // update en passant
                match (piece.get_color(), piece.get_piece_type_mut()) {
                    (color, PieceType::Pawn { en_passant })
                        if color == self.current_turn_color.get_enemy_color() =>
                    {
                        *en_passant = false;
                    }
                    _ => {}
                }

                // update promotion
                let promotionable_row = match piece.get_color() {
                    PieceColor::White => 0,
                    PieceColor::Black => BOARD_HEIGHT - 1,
                };

                if matches!(piece.get_piece_type(), PieceType::Pawn { en_passant: _ })
                    && y == promotionable_row
                {
                    piece.promote(PieceType::Queen);
                }
            }
        }
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
        let zipped = iter::zip(
            self.is_movable.iter_mut().flatten(),
            self.legal_moves.iter().flatten(),
        );

        // set is_movable[x][y] to true iff legal_moves[x][y] contains true
        for (is_movable, legal_moves) in zipped {
            *is_movable = legal_moves.iter().any(|col| col.contains(&true));
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

        let current_state_text = match self.turn_state {
            TurnState::Normal => "Normal",
            TurnState::Check => "Check",
            TurnState::Checkmate => "Checkmate",
            TurnState::Stalemate => "Stalemate",
        };

        let check_text = graphics::Text::new(current_state_text)
            .set_scale(32.)
            .set_layout(TextLayout {
                // right align
                h_align: TextAlign::End,
                v_align: TextAlign::Begin,
            })
            .clone();

        canvas.draw(
            &turn_text,
            graphics::DrawParam::from(vec2(15., 15.)).color(graphics::Color::from((0, 0, 0, 255))),
        );

        if self.turn_state != TurnState::Normal {
            canvas.draw(
                &check_text,
                graphics::DrawParam::from(vec2(WINDOW_WIDTH - 15., 15.))
                    .color(graphics::Color::from((0, 0, 0, 255))),
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
                    self.legal_moves[selected.0][selected.1][cell_x][cell_y]
                });

                let is_movable_piece = self.is_movable[cell_x][cell_y];

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
                let Some(piece) = &self.board[cell_x][cell_y] else { continue };

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
