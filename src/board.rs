use std::fmt;

use ggez::{
    glam::Vec2,
    graphics::{self, Image},
    *,
};

use crate::game::*;

const BOARD_WIDTH: usize = 8;
const BOARD_HEIGHT: usize = 8;

const PIECE_TYPES: usize = 6;

pub struct Board {
    pieces: Vec<Vec<Option<Piece>>>,
}

impl Board {
    pub fn new() -> Self {
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

    pub fn draw(
        &self,
        canvas: &mut graphics::Canvas,
        assets: &Assets,
        (x, y): (f32, f32),
    ) -> GameResult {
        let cell_size = 80.0;

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

        let sprite_original_size = 440.0;
        // 2. draw pieces on the board
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                if let Some(piece) = &self.pieces[i][j] {
                    let pos: Vec2 =
                        <[f32; 2] as Into<Vec2>>::into([
                            x + cell_size * j as f32,
                            y + cell_size * i as f32,
                        ]) + <[f32; 2] as Into<Vec2>>::into([cell_size / 2.0, cell_size / 2.0]);

                    let image = piece.get_image(assets);
                    let drawparams = graphics::DrawParam::new()
                        .dest(pos)
                        .offset([0.5, 0.5])
                        .scale([
                            cell_size / sprite_original_size,
                            cell_size / sprite_original_size,
                        ]);
                    canvas.draw(image, drawparams);
                }
            }
        }

        Ok(())
    }
}

#[derive(Clone)]
struct Piece {
    piece_type: PieceType,
    color: Color,
}

impl Piece {
    fn get_image<'a>(&self, assets: &'a Assets) -> &'a Image {
        let piece_type: usize = (self.piece_type as u64).try_into().unwrap();
        let color: usize = (self.color as u64).try_into().unwrap();

        &(assets.piece_images[color * PIECE_TYPES + piece_type])
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
