use crate::chess::{Chess, BOARD_SIZE_1D};
use crate::piece::*;

pub fn get_moves(
    board: &[Option<Piece>; BOARD_SIZE_1D],
    pos: (usize, usize),
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    let Some(piece) = &board[Chess::to_index1d(pos)] else { return };

    use PieceType::*;
    match piece.get_piece_type() {
        Pawn { .. } => get_pawn_moves(piece, board, pos, moves),
        Knight => get_knight_moves(piece, board, pos, moves),
        Bishop => get_bishop_moves(piece, board, pos, moves),
        Rook => get_rook_moves(piece, board, pos, moves),
        Queen => get_queen_moves(piece, board, pos, moves),
        King => get_king_moves(piece, board, pos, moves),
    }
}

fn get_pawn_moves(
    piece: &Piece,
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    // pawn move rule:
    // only can move forward. 2 if it's the first time, 1 otherwise.
    // must attack diagonally.

    let y_direction = if piece.get_color() == PieceColor::White {
        -1
    } else {
        1
    };
    let reach = if piece.has_moved() { 1 } else { 2 };

    // move
    for move_y in 1..=reach {
        let (nx, ny) = (x as i32, y as i32 + move_y * y_direction);

        if Chess::is_position_in_bound((nx, ny))
            && Chess::is_empty_on(board, (nx as usize, ny as usize))
        {
            moves[Chess::to_index1d((nx as usize, ny as usize))] = true;
        } else {
            break;
        }
    }

    // attack
    let enemy_color = piece.get_color().get_enemy_color();

    for move_x in [-1, 1] {
        let (nx, ny) = (x as i32 + move_x, y as i32 + y_direction);

        if !Chess::is_position_in_bound((nx, ny)) {
            continue;
        }

        let is_directly_attackable =
            Chess::is_color_on(board, (nx as usize, ny as usize), enemy_color);
        let can_en_passant = Chess::get_piece(board, (nx as usize, y)).is_some_and(|piece| {
            matches!(piece.get_piece_type(), PieceType::Pawn { en_passant: true })
        });

        if is_directly_attackable || can_en_passant {
            moves[Chess::to_index1d((nx as usize, ny as usize))] = true;
        }
    }
}

fn get_knight_moves(
    piece: &Piece,
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    moves: &mut [bool; BOARD_SIZE_1D],
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

    let enemy_color = piece.get_color().get_enemy_color();

    for (move_x, move_y) in DIRS {
        let (nx, ny) = (x as i32 + move_x, y as i32 + move_y);

        if Chess::is_position_in_bound((nx, ny))
            && (Chess::is_empty_on(board, (nx as usize, ny as usize))
                || Chess::is_color_on(board, (nx as usize, ny as usize), enemy_color))
        {
            moves[Chess::to_index1d((nx as usize, ny as usize))] = true;
        }
    }
}

fn get_bishop_moves(
    piece: &Piece,
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    let enemy_color = piece.get_color().get_enemy_color();

    get_diagonal_moves(board, (x, y), enemy_color, moves)
}

fn get_rook_moves(
    piece: &Piece,
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    let enemy_color = piece.get_color().get_enemy_color();

    get_orthogonal_moves(board, (x, y), enemy_color, moves)
}

fn get_queen_moves(
    piece: &Piece,
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    let enemy_color = piece.get_color().get_enemy_color();

    get_diagonal_moves(board, (x, y), enemy_color, moves);
    get_orthogonal_moves(board, (x, y), enemy_color, moves);
}

fn get_king_moves(
    piece: &Piece,
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    // todo: castling

    let enemy_color = piece.get_color().get_enemy_color();

    for move_x in -1..=1 {
        for move_y in -1..=1 {
            if (move_x, move_y) == (0, 0) {
                continue;
            }

            let (nx, ny) = (x as i32 + move_x, y as i32 + move_y);

            if Chess::is_position_in_bound((nx, ny))
                && (Chess::is_empty_on(board, (nx as usize, ny as usize))
                    || Chess::is_color_on(board, (nx as usize, ny as usize), enemy_color))
            {
                moves[Chess::to_index1d((nx as usize, ny as usize))] = true;
            }
        }
    }
}

fn get_moves_in_direction(
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    (x_dir, y_dir): (i32, i32),
    enemy_color: PieceColor,
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    let (mut nx, mut ny) = (x as i32, y as i32);

    loop {
        nx += x_dir;
        ny += y_dir;

        if !Chess::is_position_in_bound((nx, ny)) {
            break;
        }

        let is_empty = Chess::is_empty_on(board, (nx as usize, ny as usize));
        let is_enemy = Chess::is_color_on(board, (nx as usize, ny as usize), enemy_color);

        if is_empty || is_enemy {
            moves[Chess::to_index1d((nx as usize, ny as usize))] = true;
        }

        if !is_empty {
            break;
        }
    }
}

fn get_orthogonal_moves(
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    enemy_color: PieceColor,
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    static DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for (x_dir, y_dir) in DIRS {
        get_moves_in_direction(board, (x, y), (x_dir, y_dir), enemy_color, moves);
    }
}

fn get_diagonal_moves(
    board: &[Option<Piece>; BOARD_SIZE_1D],
    (x, y): (usize, usize),
    enemy_color: PieceColor,
    moves: &mut [bool; BOARD_SIZE_1D],
) {
    for x_dir in [-1, 1] {
        for y_dir in [-1, 1] {
            get_moves_in_direction(board, (x, y), (x_dir, y_dir), enemy_color, moves);
        }
    }
}
