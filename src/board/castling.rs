use crate::pos::{Pos, File, Rank};
use crate::piece::PieceColor;
use crate::board::BoardState;

#[derive(Debug, Clone)]
pub struct CastlingState {
    pub king_moved: bool,
    pub rook_moved_short: bool,
    pub rook_moved_long: bool
}

impl CastlingState {
    pub fn new() -> Self {
        Self {
            king_moved: false,
            rook_moved_short: false,
            rook_moved_long: false,
        }
    }

    pub fn is_short_possible(&self) -> bool {
        !self.king_moved && !self.rook_moved_short
    }

    pub fn is_long_possible(&self) -> bool {
        !self.king_moved && !self.rook_moved_long
    }
}
