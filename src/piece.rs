use std::fmt;

use ggez::{graphics::Image, Context};

use crate::game::Assets;

#[derive(Copy, Clone)]
pub struct Piece {
    piece_type: PieceType,
    color: PieceColor,
    has_moved: bool,
}

impl Piece {
    pub fn new(piece_type: PieceType, color: PieceColor) -> Self {
        Self {
            piece_type,
            color,
            has_moved: false,
        }
    }

    pub fn get_image<'a>(&self, ctx: &mut Context, assets: &'a mut Assets) -> &'a Image {
        let sprite = self.color.to_string() + &self.piece_type.to_string();

        assets.try_get_image(ctx, &sprite).unwrap()
    }

    pub fn get_piece_type(&self) -> PieceType {
        self.piece_type
    }

    pub fn get_piece_type_mut(&mut self) -> &mut PieceType {
        &mut self.piece_type
    }

    pub fn get_color(&self) -> PieceColor {
        self.color
    }

    pub fn has_moved(&self) -> bool {
        self.has_moved
    }

    pub fn set_has_moved(&mut self, has_moved: bool) {
        self.has_moved = has_moved;
    }

    pub fn promote(&mut self, promote_to: PieceType) {
        if !matches!(self.get_piece_type(), PieceType::Pawn { .. }) {
            panic!("{self} cannot promote");
        }

        if promote_to == PieceType::King {
            panic!("cannot promote to King");
        }

        self.piece_type = promote_to;
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
    Pawn { en_passant: bool },
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
            Pawn { .. } => 'p',
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
