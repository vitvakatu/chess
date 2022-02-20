use crate::board::Square;
use crate::piece::Piece;
use crate::piece::PieceColor;
use crate::piece::PieceType;

pub const STARTING_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";
pub const FOOLS_MATE: &str = "rnbqkbnr/pppppppp/8/7Q/2B5/8/PPPP1PPP/RNB1K1NR";
pub const TWO_KINGS: &str = "4k3/8/8/8/8/8/8/3K4";

pub fn parse_fen(fen: &str) -> Vec<Square> {
    let mut result = Vec::new();
    let ranks = fen.split("/");
    for rank in ranks {
        for piece in rank.chars() {
            if piece.is_digit(10) {
                let n = piece.to_string().parse().unwrap();
                for _ in 0..n {
                    result.push(Square::Empty);
                }
                continue;
            }
            let color = match piece.is_ascii_lowercase() {
                true => PieceColor::Black,
                false => PieceColor::White,
            };
            let piece = piece.to_ascii_lowercase();
            let kind = match piece {
                'p' => PieceType::Pawn,
                'k' => PieceType::King,
                'q' => PieceType::Queen,
                'r' => PieceType::Rook,
                'b' => PieceType::Bishop,
                'n' => PieceType::Knight,
                other => panic!("Unrecongnized piece type: {other}"),
            };
            result.push(Square::Piece(Piece { kind, color }));
        }
    }
    result
}
