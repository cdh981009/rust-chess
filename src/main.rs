use std::fmt::{self, Display};

use ggez::*;

fn main() {
    let c = conf::Conf::new();
    let (ctx, event_loop) = ContextBuilder::new("rust_chess", "cdh981009")
        .default_conf(c)
        .build()
        .unwrap();

    let state = GameState::new(&ctx);

    event::run(ctx, event_loop, state);
}

struct GameState {
    screen_width: f32,
    screen_height: f32,
    board: Board,
}

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

struct Board {
    pieces: Vec<Vec<Option<Piece>>>,
}

#[derive(Clone)]
struct Piece {
    piece_type: PieceType,
    color: Color,
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

#[derive(Clone)]
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

#[derive(Clone, PartialEq)]
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

impl GameState {
    fn new(ctx: &Context) -> GameState {
        let (screen_width, screen_height) = ctx.gfx.drawable_size();
        let board = Board::new();

        GameState {
            screen_width,
            screen_height,
            board,
        }
    }
}

impl Board {
    fn new() -> Self {
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

                pieces[row][col] = Some(Piece { piece_type, color });
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

        Board { pieces }
    }

    fn draw(&self, canvas: &mut graphics::Canvas, (x, y): (f32, f32)) -> GameResult {
        let cell_size = 50.0;

        // 1. draw board
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                let color = if (i + j) % 2 == 0 {
                    graphics::Color::WHITE
                } else {
                    graphics::Color::BLACK
                };

                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::default()
                        .color(color)
                        .scale([cell_size, cell_size])
                        .dest([x + cell_size * j as f32, y + cell_size * i as f32]),
                );
            }
        }

        // 2. draw pieces on the board
        // TODO

        Ok(())
    }
}

impl ggez::event::EventHandler<GameError> for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::BLUE);
        let offset = (100.0, 100.0);

        self.board.draw(&mut canvas, offset)?;

        canvas.finish(ctx)?;

        Ok(())
    }
}
