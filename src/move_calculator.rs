use crate::board::*;

pub fn get_moves(pos: (usize, usize), board: &Board, moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT]) {
    let Some(piece) = &board.get_piece(pos) else { return };

    use PieceType::*;
    match piece.get_piece_type() {
        Pawn => get_pawn_moves(piece, pos, board, moves),
        Knight => get_knight_moves(piece, pos, board, moves),
        Bishop => get_bishop_moves(piece, pos, board, moves),
        Rook => get_rook_moves(piece, pos, board, moves),
        Queen => get_queen_moves(piece, pos, board, moves),
        King => get_king_moves(piece, pos, board, moves),
    }
}

fn get_pawn_moves(
    piece: &Piece,
    (x, y): (usize, usize),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
) {
    // pawn move rule:
    // only can move forward. 2 if it's the first time, 1 otherwise.
    // must attack diagonally.

    let y_direction = if piece.get_color() == Color::White {
        -1
    } else {
        1
    };
    let reach = if piece.has_moved() { 1 } else { 2 };

    // move
    for move_y in 1..=reach {
        let (nx, ny) = (x as i32, y as i32 + move_y * y_direction);

        if Board::is_position_in_bound((nx, ny)) && board.is_empty_on((nx as usize, ny as usize)) {
            moves[Board::to_index1d((nx as usize, ny as usize))] = true;
        } else {
            break;
        }
    }

    // attack
    let enemy_color = if piece.get_color() == Color::White {
        Color::Black
    } else {
        Color::White
    };

    for move_x in [-1, 1] {
        let (nx, ny) = (x as i32 + move_x, y as i32 + y_direction);

        if Board::is_position_in_bound((nx, ny))
            && board.is_color_on((nx as usize, ny as usize), enemy_color)
        {
            moves[Board::to_index1d((nx as usize, ny as usize))] = true;
        }
    }
}

fn get_knight_moves(
    piece: &Piece,
    (x, y): (usize, usize),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
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

    let enemy_color = if piece.get_color() == Color::White {
        Color::Black
    } else {
        Color::White
    };

    for (move_x, move_y) in DIRS {
        let (nx, ny) = (x as i32 + move_x, y as i32 + move_y);

        if Board::is_position_in_bound((nx, ny))
            && (board.is_empty_on((nx as usize, ny as usize))
                || board.is_color_on((nx as usize, ny as usize), enemy_color))
        {
            moves[Board::to_index1d((nx as usize, ny as usize))] = true;
        }
    }
}

fn get_bishop_moves(
    piece: &Piece,
    (x, y): (usize, usize),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
) {
    get_diagonal_moves(piece, (x, y), board, moves)
}

fn get_rook_moves(
    piece: &Piece,
    (x, y): (usize, usize),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
) {
    get_orthogonal_moves(piece, (x, y), board, moves)
}

fn get_queen_moves(
    piece: &Piece,
    (x, y): (usize, usize),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
) {
    get_diagonal_moves(piece, (x, y), board, moves);
    get_orthogonal_moves(piece, (x, y), board, moves);
}

fn get_king_moves(
    piece: &Piece,
    (x, y): (usize, usize),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
) {
    // todo: castling

    let enemy_color = if piece.get_color() == Color::White {
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
                && (board.is_empty_on((nx as usize, ny as usize))
                    || board.is_color_on((nx as usize, ny as usize), enemy_color))
            {
                moves[Board::to_index1d((nx as usize, ny as usize))] = true;
            }
        }
    }
}

fn get_moves_in_direction(
    enemy_color: Color,
    (x, y): (usize, usize),
    (x_dir, y_dir): (i32, i32),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
) {
    let (mut nx, mut ny) = (x as i32, y as i32);

    loop {
        nx += x_dir;
        ny += y_dir;

        if !Board::is_position_in_bound((nx, ny)) {
            break;
        }

        let is_empty = board.is_empty_on((nx as usize, ny as usize));
        let is_enemy = board.is_color_on((nx as usize, ny as usize), enemy_color);

        if is_empty || is_enemy {
            moves[Board::to_index1d((nx as usize, ny as usize))] = true;
        }

        if !is_empty {
            break;
        }
    }
}

fn get_orthogonal_moves(
    piece: &Piece,
    (x, y): (usize, usize),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
) {
    static DIRS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    let enemy_color = if piece.get_color() == Color::White {
        Color::Black
    } else {
        Color::White
    };

    for (x_dir, y_dir) in DIRS {
        get_moves_in_direction(enemy_color, (x, y), (x_dir, y_dir), board, moves);
    }
}

fn get_diagonal_moves(
    piece: &Piece,
    (x, y): (usize, usize),
    board: &Board,
    moves: &mut [bool; BOARD_WIDTH * BOARD_HEIGHT],
) {
    let enemy_color = if piece.get_color() == Color::White {
        Color::Black
    } else {
        Color::White
    };

    for x_dir in [-1, 1] {
        for y_dir in [-1, 1] {
            get_moves_in_direction(enemy_color, (x, y), (x_dir, y_dir), board, moves);
        }
    }
}
