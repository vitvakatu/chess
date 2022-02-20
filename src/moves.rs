use std::fmt;

use crate::piece::{PieceColor, Piece, PieceType};

/// Half-move
pub type Ply = u32;
pub type MoveNumber = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CastlingSide {
    Short,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromotedTo {
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl PromotedTo {
    pub fn to_piece(&self, color: PieceColor) -> Piece {
        match self {
            Self::Knight => Piece { kind: PieceType::Knight, color },
            Self::Bishop => Piece { kind: PieceType::Bishop, color },
            Self::Rook => Piece { kind: PieceType::Rook, color },
            Self::Queen => Piece { kind: PieceType::Queen, color },
        }
    }
}

impl fmt::Display for PromotedTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Knight => write!(f, "=K"),
            Self::Bishop => write!(f, "=B"),
            Self::Rook => write!(f, "=R"),
            Self::Queen => write!(f, "=Q"),
        }
    }
}

pub mod internal {
    use super::CastlingSide;
    use super::PromotedTo;
    use crate::pos::Pos;
    use std::fmt;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Move {
        Regular {
            from: Pos,
            to: Pos,
            promoted: Option<PromotedTo>,
        },
        Castling {
            side: CastlingSide,
        },
    }

    impl Move {
        pub fn new(from: Pos, to: Pos) -> Self {
            Self::Regular {
                from,
                to,
                promoted: None,
            }
        }

        pub fn new_with_promoted(from: Pos, to: Pos, promoted: Option<PromotedTo>) -> Self {
            Self::Regular {
                from, to, promoted
            }
        }

        pub fn castling(side: CastlingSide) -> Self {
            Self::Castling { side }
        }

        pub fn from(&self) -> Option<Pos> {
            match self {
                Self::Regular { from, .. } => Some(*from),
                Self::Castling { .. } => None,
            }
        }

        pub fn to(&self) -> Option<Pos> {
            match self {
                Self::Regular { to, .. } => Some(*to),
                Self::Castling { .. } => None,
            }
        }
    }

    impl fmt::Display for Move {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Regular { from, to, promoted } => write!(
                    f,
                    "{from}{to}{promoted}",
                    promoted = promoted.as_ref().map(ToString::to_string).unwrap_or_default()
                ),
                Self::Castling {
                    side: CastlingSide::Short,
                } => write!(f, "O-O"),
                Self::Castling {
                    side: CastlingSide::Long,
                } => write!(f, "O-O-O"),
            }
        }
    }
}

pub mod san {
    use super::CastlingSide;
    use super::PromotedTo;
    use crate::piece::PieceType;
    use crate::pos::File;
    use crate::pos::Pos;
    use crate::pos::Rank;
    use std::fmt;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum FromPos {
        Square(Pos),
        File(File),
        Rank(Rank),
    }

    impl fmt::Display for FromPos {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Square(pos) => write!(f, "{pos}"),
                Self::File(file) => write!(f, "{file}"),
                Self::Rank(rank) => write!(f, "{rank}", rank = rank.get()),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Move {
        Piece {
            piece: PieceType,
            is_capture: bool,
            from: Option<FromPos>,
            to: Pos,
        },
        PawnPush {
            to: Pos,
            promoted: Option<PromotedTo>,
        },
        PawnCapture {
            from_file: File,
            from_rank: Option<Rank>,
            to: Pos,
            promoted: Option<PromotedTo>,
        },
        Castling {
            side: CastlingSide,
        },
    }

    impl fmt::Display for Move {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::Piece {
                    piece,
                    is_capture,
                    from,
                    to,
                } => write!(
                    f,
                    "{piece}{from}{is_capture}{to}",
                    from = from.as_ref().map(ToString::to_string).unwrap_or_default(),
                    is_capture = if *is_capture { "x" } else { "" },
                ),
                Self::PawnPush { to, promoted } => write!(
                    f,
                    "{to}{promoted}",
                    promoted = promoted
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                ),
                Self::PawnCapture {
                    from_file,
                    from_rank,
                    to,
                    promoted,
                } => write!(
                    f,
                    "{from_file}{from_rank}x{to}{promoted}",
                    from_rank = from_rank
                        .map(|r| r.get().clone())
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default(),
                    promoted = promoted
                        .as_ref()
                        .map(ToString::to_string)
                        .unwrap_or_default()
                ),
                Self::Castling {
                    side: CastlingSide::Short,
                } => write!(f, "O-O"),
                Self::Castling {
                    side: CastlingSide::Long,
                } => write!(f, "O-O-O"),
            }
        }
    }
}
