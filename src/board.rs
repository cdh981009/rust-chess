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
    selected: Option<(usize, usize)>,
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

        let selected = None;
        let cell_size = 80.0;

        Board {
            pieces,
            selected,
            x,
            y,
            cell_size,
        }
    }

    pub fn update(&mut self, mouse: &Mouse) {}

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
        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                let light_color = 0x9699A1;
                let dark_color = 0x434347;

                let color = graphics::Color::from_rgb_u32(if (i + j) % 2 == 0 {
                    light_color
                } else {
                    dark_color
                });

                canvas.draw(
                    &graphics::Quad,
                    graphics::DrawParam::default()
                        .color(color)
                        .scale([cell_size, cell_size])
                        .dest([x + cell_size * j as f32, y + cell_size * i as f32]),
                );
            }
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

        for i in 0..BOARD_HEIGHT {
            for j in 0..BOARD_WIDTH {
                if let Some(piece) = &self.pieces[i][j] {
                    // set pos to the center of the cell
                    let pos: Vec2 =
                        <[f32; 2] as Into<Vec2>>::into([
                            x + cell_size * j as f32,
                            y + cell_size * i as f32,
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
