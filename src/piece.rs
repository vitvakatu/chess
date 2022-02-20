use crate::pos::{File, Pos, Rank};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceColor {
    White,
    Black,
}

impl PieceColor {
    pub fn king_rank(&self) -> Rank {
        match self {
            Self::White => Rank::new(1),
            Self::Black => Rank::new(8),
        }
    }

    pub fn king_home(&self) -> Pos {
        Pos::new(File::E, self.king_rank())
    }
}

impl std::fmt::Display for PieceColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceColor::White => write!(f, "White"),
            PieceColor::Black => write!(f, "Black"),
        }
    }
}

impl std::ops::Not for PieceColor {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PieceType {
    Pawn,
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Piece {
    pub color: PieceColor,
    pub kind: PieceType,
}

impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PieceType::King => write!(f, "K"),
            PieceType::Pawn => write!(f, "P"),
            PieceType::Queen => write!(f, "Q"),
            PieceType::Rook => write!(f, "R"),
            PieceType::Bishop => write!(f, "B"),
            PieceType::Knight => write!(f, "N"),
        }
    }
}
