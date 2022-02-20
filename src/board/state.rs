use crate::board::castling::CastlingState;
use crate::board::Handles;
use crate::board::Square;
use crate::fen::parse_fen;
use crate::fen::STARTING_POSITION;
use crate::moves::internal::Move;
use crate::moves::san::Move as SanMove;
use crate::moves::CastlingSide;
use crate::moves::MoveNumber;
use crate::moves::Ply;
use crate::moves::PromotedTo;
use crate::piece::{Piece, PieceColor, PieceType};
use crate::pos::{
    DiagonalDirection, File, HorizontalDirection, Pos, Rank, UnboundedPos, VerticalDirection,
};
use crate::take_while::TakeWhileInclusiveExt;
use iced::Point;

pub type IsHighlighted = bool;

pub type Turn = PieceColor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameResult {
    WinByCheckmate { checkmated_side: PieceColor },
    DrawByStalemate,
}

#[derive(Debug, Clone)]
pub struct BoardState {
    pub squares: Vec<(Square, IsHighlighted)>,
    pub selected_piece: Option<(Piece, Pos)>,
    pub turn: Turn,
    castling_white: CastlingState,
    castling_black: CastlingState,
    plies_since_last_non_repeatable_move: Ply,
    move_number: MoveNumber,
    pub game_result: Option<GameResult>,
    pub handles: Handles,
}

impl BoardState {
    pub fn new() -> Self {
        let squares = parse_fen(STARTING_POSITION)
            .into_iter()
            .map(|s| (s, false))
            .collect();
        Self {
            squares,
            selected_piece: None,
            turn: Turn::White,
            castling_white: CastlingState::new(),
            castling_black: CastlingState::new(),
            plies_since_last_non_repeatable_move: 0,
            move_number: 1,
            game_result: None,
            handles: Handles::new(),
        }
    }

    fn switch_turn(&mut self) {
        self.turn = !self.turn;
    }

    pub fn is_castling_possible(&self, side: CastlingSide) -> bool {
        let castling = self.castling(self.turn);
        if side == CastlingSide::Short {
            !castling.king_moved && !castling.rook_moved_short
        } else {
            !castling.king_moved && !castling.rook_moved_long
        }
    }

    pub fn plies_since_last_non_repeatable_move(&self) -> Ply {
        self.plies_since_last_non_repeatable_move
    }

    pub fn move_number(&self) -> MoveNumber {
        self.move_number
    }

    fn castling(&self, color: PieceColor) -> &CastlingState {
        if color == PieceColor::White {
            &self.castling_white
        } else {
            &self.castling_black
        }
    }

    fn castling_mut(&mut self, color: PieceColor) -> &mut CastlingState {
        if color == PieceColor::White {
            &mut self.castling_white
        } else {
            &mut self.castling_black
        }
    }

    pub fn make_move(&mut self, mv: Move) {
        match mv {
            Move::Regular { from, to, promoted } => {
                let piece = if let Square::Piece(piece) = self.square_by_pos(from) {
                    piece
                } else {
                    panic!("Invalid move (no piece): {mv:?}");
                };
                if piece.kind == PieceType::King {
                    self.castling_mut(piece.color).king_moved = true;
                }
                if piece.kind == PieceType::Rook && from.file == File::A {
                    self.castling_mut(piece.color).rook_moved_long = true;
                }
                if piece.kind == PieceType::Rook && from.file == File::H {
                    self.castling_mut(piece.color).rook_moved_short = true;
                }
                if let Some(promoted) = promoted {
                    println!("Promoting: {piece:?}, {to:?}");
                    self.make_move_promote(from, to, promoted);
                } else {
                    self.make_move_inner(from, to);
                }

                let is_capture = self.is_square_occupied(to);
                let is_pawn_move = piece.kind == PieceType::Pawn;
                if is_capture || is_pawn_move {
                    self.plies_since_last_non_repeatable_move = 0;
                } else {
                    self.plies_since_last_non_repeatable_move += 1;
                }
            }
            Move::Castling { side } => {
                use CastlingSide::*;
                use PieceColor::*;
                self.plies_since_last_non_repeatable_move += 1;
                self.castling_mut(self.turn).king_moved = true;
                let rank = if self.turn == White {
                    Rank::new(1)
                } else {
                    Rank::new(8)
                };
                let king_home = Pos::new(File::E, rank);
                let king_dest_short = Pos::new(File::G, rank);
                let king_dest_long = Pos::new(File::C, rank);
                let rook_home_short = Pos::new(File::H, rank);
                let rook_home_long = Pos::new(File::A, rank);
                let rook_dest_short = Pos::new(File::F, rank);
                let rook_dest_long = Pos::new(File::D, rank);
                match side {
                    Short => {
                        self.castling_mut(self.turn).rook_moved_short = true;
                        self.make_move_inner(king_home, king_dest_short);
                        self.make_move_inner(rook_home_short, rook_dest_short);
                    }
                    Long => {
                        self.castling_mut(self.turn).rook_moved_long = true;
                        self.make_move_inner(king_home, king_dest_long);
                        self.make_move_inner(rook_home_long, rook_dest_long);
                    }
                }
            }
        }
        self.switch_turn();
        if self.turn == Turn::White {
            self.move_number += 1;
        }
    }

    fn make_move_inner(&mut self, from: Pos, to: Pos) {
        let piece = self.square_by_pos(from);
        *self.square_by_pos_mut(from) = Square::Empty;
        *self.square_by_pos_mut(to) = piece;
    }

    fn make_move_promote(&mut self, from: Pos, to: Pos, promote: PromotedTo) {
        *self.square_by_pos_mut(from) = Square::Empty;
        *self.square_by_pos_mut(to) = Square::Piece(promote.to_piece(self.turn));
    }

    pub fn hightlight_legal_moves(&mut self, piece: Piece, pos: Pos) {
        let legal_moves = self.legal_moves(piece, pos);
        for mv in legal_moves {
            match mv {
                Move::Regular { to, .. } => {
                    let index = Self::square_index_by_pos(to);
                    self.squares[index].1 = true;
                }
                Move::Castling { side } => {
                    let pos = match (side, piece.color) {
                        (CastlingSide::Short, PieceColor::White) => Pos::new(File::G, Rank::new(1)),
                        (CastlingSide::Short, PieceColor::Black) => Pos::new(File::G, Rank::new(8)),
                        (CastlingSide::Long, PieceColor::White) => Pos::new(File::C, Rank::new(1)),
                        (CastlingSide::Long, PieceColor::Black) => Pos::new(File::C, Rank::new(8)),
                    };
                    let index = Self::square_index_by_pos(pos);
                    self.squares[index].1 = true;
                }
            }
        }
    }

    pub fn stop_highlighting(&mut self) {
        for (_, is_highlighted) in self.squares.iter_mut() {
            *is_highlighted = false;
        }
    }

    /// Depends on self.turn
    pub fn from_san_move(&self, mv: SanMove) -> Move {
        match mv {
            SanMove::Castling { side } => Move::Castling { side },
            SanMove::Piece {
                from, to, piece, ..
            } => {
                use crate::moves::san::FromPos;
                let find_possible_solution = |file, rank| -> Option<Pos> {
                    let restrict: Box<dyn Fn(&(Piece, Pos)) -> bool> = match (file, rank) {
                        (Some(file), None) => Box::new(move |(_, pos)| pos.file == file),
                        (None, Some(rank)) => Box::new(move |(_, pos)| pos.rank == rank),
                        (None, None) => Box::new(|(_, _)| true),
                        _ => unreachable!(),
                    };
                    self.pieces()
                        .filter(|(Piece { kind, color }, pos)| {
                            *kind == piece && *color == self.turn
                        })
                        .filter(restrict)
                        .map(|(piece, pos)| {
                            self.legal_moves(piece, pos)
                                .iter()
                                .map(|m| (*m, pos))
                                .collect::<Vec<_>>()
                        })
                        .flatten()
                        .filter(|(m, pos)| m.to() == Some(to))
                        .map(|(_, pos)| pos)
                        .next()
                };
                let from = match from {
                    Some(FromPos::Square(pos)) => pos,
                    Some(FromPos::File(file)) => {
                        find_possible_solution(Some(file), None).expect("Couldn't find move")
                    }
                    Some(FromPos::Rank(rank)) => {
                        find_possible_solution(None, Some(rank)).expect("Couldn't find move")
                    }
                    None => find_possible_solution(None, None).expect("Couldn't find move"),
                };
                Move::new(from, to)
            }
            SanMove::PawnPush { to, promoted } => {
                let from = self
                    .pieces()
                    .filter(|(Piece { kind, color }, pos)| {
                        *kind == PieceType::Pawn && *color == self.turn
                    })
                    .filter(|(piece, pos)| {
                        self.legal_moves(*piece, *pos)
                            .iter()
                            .any(|m| m.to() == Some(to))
                    })
                    .map(|(_, pos)| pos)
                    .next()
                    .expect("Can't find pawn move");
                Move::new_with_promoted(from, to, promoted)
            }
            SanMove::PawnCapture {
                from_file,
                from_rank,
                to,
                promoted,
            } => {
                let from = from_rank
                    .map(|r| Pos::new(from_file, r))
                    .unwrap_or_else(|| {
                        self.pieces()
                            .filter(|(Piece { kind, color }, pos)| {
                                *kind == PieceType::Pawn
                                    && *color == self.turn
                                    && pos.file == from_file
                            })
                            .filter(|(piece, pos)| {
                                self.legal_moves(*piece, *pos)
                                    .iter()
                                    .any(|m| m.to() == Some(to))
                            })
                            .map(|(_, pos)| pos)
                            .next()
                            .expect("Can't find pawn capture")
                    });
                Move::new_with_promoted(from, to, promoted)
            }
        }
    }

    /// Depends on self.turn
    pub fn to_san_move(&self, mv: Move) -> SanMove {
        use PieceType::*;
        let (from, to, promoted) = match mv {
            Move::Regular { from, to, promoted } => (from, to, promoted),
            Move::Castling { side } => return SanMove::Castling { side },
        };
        match self.square_by_pos(from) {
            Square::Empty => panic!("Invalid move (no piece): {mv:?}. Can't produce SAN move"),
            Square::Piece(
                piece @ Piece {
                    kind: King | Queen | Rook | Bishop | Knight,
                    ..
                },
            ) => {
                use crate::moves::san::FromPos;
                let is_capture = self.is_square_occupied(to);
                let count_same_pieces = self
                    .pieces()
                    .filter(|(Piece { kind, color }, _)| {
                        *kind == piece.kind && *color == piece.color
                    })
                    .count();
                let from = if count_same_pieces == 1 {
                    None
                } else {
                    let other_pieces = self.pieces().filter(|(Piece { kind, color }, pos)| {
                        *pos != from && *kind == piece.kind && *color == piece.color
                    });
                    let legal_moves =
                        other_pieces.map(|(piece, pos)| (pos, self.legal_moves(piece, pos)));
                    let matching_moves: Vec<_> = legal_moves
                        .map(|(pos, moves)| {
                            moves
                                .iter()
                                .filter(|m| m.to() == Some(to))
                                .map(|_| pos)
                                .collect::<Vec<_>>()
                        })
                        .flatten()
                        .collect();
                    if matching_moves.is_empty() {
                        None
                    } else if matching_moves.iter().any(|pos| pos.file != from.file) {
                        Some(FromPos::File(from.file))
                    } else if matching_moves.iter().all(|pos| pos.file == from.file) {
                        Some(FromPos::Rank(from.rank))
                    } else {
                        Some(FromPos::Square(from))
                    }
                };
                SanMove::Piece {
                    piece: piece.kind,
                    is_capture,
                    from,
                    to,
                }
            }
            Square::Piece(pawn) => {
                let is_capture = self.is_square_occupied(to);
                if is_capture {
                    let from_file = from.file;
                    let pawns_on_this_file = self
                        .pieces()
                        .filter(|(Piece { kind, color }, pos)| {
                            *kind == Pawn && *color == pawn.color && pos.file == from_file
                        })
                        .count();
                    let from_rank = if pawns_on_this_file > 1 {
                        Some(from.rank)
                    } else {
                        None
                    };
                    SanMove::PawnCapture {
                        from_file,
                        from_rank,
                        to,
                        promoted,
                    }
                } else {
                    SanMove::PawnPush { to, promoted }
                }
            }
        }
    }

    pub fn available_moves(&self, piece: Piece, pos: Pos) -> Vec<Move> {
        use PieceType::*;
        let mut result = Vec::new();
        match piece.kind {
            Pawn => {
                let is_white = piece.color == PieceColor::White;
                let direction = if is_white {
                    VerticalDirection::Up
                } else {
                    VerticalDirection::Down
                };
                // pawns can move up/down one square if not blocked
                // they also can move up/down two squares if it's their first move
                let is_first_move = if is_white {
                    pos.rank.get() == 2
                } else {
                    pos.rank.get() == 7
                };
                let moves =
                    UnboundedPos::vertical(pos, if is_first_move { 2 } else { 1 }, direction)
                        .into_iter()
                        .filter_map(|p| p.to_pos())
                        .take_while(|p| !self.is_square_occupied(*p));
                // pawns can take other pieces in diagonals
                let diagonal_directions = if is_white {
                    [DiagonalDirection::UpLeft, DiagonalDirection::UpRight]
                } else {
                    [DiagonalDirection::DownLeft, DiagonalDirection::DownRight]
                };
                let moves = moves.chain(
                    diagonal_directions
                        .into_iter()
                        .map(|dir| {
                            UnboundedPos::diagonal(pos, 1, dir)
                                .filter_map(|p| p.to_pos())
                                .filter(|p| self.is_square_occupied_by_color(*p, !piece.color))
                        })
                        .flatten(),
                );
                let moves = moves
                    .map(|to| {
                        if to.rank == (!piece.color).king_rank() {
                            vec![
                                Move::new_with_promoted(pos, to, Some(PromotedTo::Queen)),
                                Move::new_with_promoted(pos, to, Some(PromotedTo::Bishop)),
                                Move::new_with_promoted(pos, to, Some(PromotedTo::Rook)),
                                Move::new_with_promoted(pos, to, Some(PromotedTo::Knight)),
                            ]
                        } else {
                            vec![Move::new(pos, to)]
                        }
                    })
                    .flatten();
                result.extend(moves);
            }
            King => {
                let castling = self.castling(piece.color);
                let short_castling = castling
                    .is_short_possible()
                    .then(|| Move::castling(CastlingSide::Short));
                let long_castling = castling
                    .is_long_possible()
                    .then(|| Move::castling(CastlingSide::Long));

                let moves = UnboundedPos::vertical(pos, 1, VerticalDirection::Up)
                    .into_iter()
                    .chain(UnboundedPos::vertical(pos, 1, VerticalDirection::Down))
                    .chain(UnboundedPos::horizontal(pos, 1, HorizontalDirection::Left))
                    .chain(UnboundedPos::horizontal(pos, 1, HorizontalDirection::Right))
                    .chain(UnboundedPos::diagonal(pos, 1, DiagonalDirection::UpLeft))
                    .chain(UnboundedPos::diagonal(pos, 1, DiagonalDirection::UpRight))
                    .chain(UnboundedPos::diagonal(pos, 1, DiagonalDirection::DownLeft))
                    .chain(UnboundedPos::diagonal(pos, 1, DiagonalDirection::DownRight))
                    .filter_map(|p| p.to_pos())
                    .filter(|p| !self.is_square_occupied_by_color(*p, piece.color))
                    .map(|to| Move::new(pos, to))
                    .chain(short_castling)
                    .chain(long_castling);
                result.extend(moves);
            }
            Bishop => {
                let directions = [
                    DiagonalDirection::UpLeft,
                    DiagonalDirection::UpRight,
                    DiagonalDirection::DownLeft,
                    DiagonalDirection::DownRight,
                ];
                let moves = directions
                    .into_iter()
                    .map(|dir| {
                        UnboundedPos::diagonal(pos, 8, dir)
                            .filter_map(|p| p.to_pos())
                            .take_while_inclusive(|p| !self.is_square_occupied(*p))
                            .filter(|p| !self.is_square_occupied_by_color(*p, piece.color))
                    })
                    .flatten()
                    .map(|to| Move::new(pos, to));
                result.extend(moves);
            }
            Knight => {
                let moves = [
                    UnboundedPos::from_pos(pos).up(2).left(1),
                    UnboundedPos::from_pos(pos).up(2).right(1),
                    UnboundedPos::from_pos(pos).down(2).left(1),
                    UnboundedPos::from_pos(pos).down(2).right(1),
                    UnboundedPos::from_pos(pos).up(1).left(2),
                    UnboundedPos::from_pos(pos).up(1).right(2),
                    UnboundedPos::from_pos(pos).down(1).left(2),
                    UnboundedPos::from_pos(pos).down(1).right(2),
                ]
                .into_iter()
                .filter_map(|p| p.to_pos())
                .filter(|p| !self.is_square_occupied_by_color(*p, piece.color))
                .map(|to| Move::new(pos, to));
                result.extend(moves);
            }
            Rook => {
                let directions = [VerticalDirection::Up, VerticalDirection::Down];
                let moves = directions
                    .into_iter()
                    .map(|dir| {
                        UnboundedPos::vertical(pos, 8, dir)
                            .filter_map(|p| p.to_pos())
                            .take_while_inclusive(|p| !self.is_square_occupied(*p))
                            .filter(|p| !self.is_square_occupied_by_color(*p, piece.color))
                    })
                    .flatten()
                    .chain(
                        [HorizontalDirection::Left, HorizontalDirection::Right]
                            .into_iter()
                            .map(|dir| {
                                UnboundedPos::horizontal(pos, 8, dir)
                                    .filter_map(|p| p.to_pos())
                                    .take_while_inclusive(|p| !self.is_square_occupied(*p))
                                    .filter(|p| !self.is_square_occupied_by_color(*p, piece.color))
                            })
                            .flatten(),
                    )
                    .map(|to| Move::new(pos, to));
                result.extend(moves);
            }
            Queen => {
                let directions = [VerticalDirection::Up, VerticalDirection::Down];
                let moves = directions
                    .into_iter()
                    .map(|dir| {
                        UnboundedPos::vertical(pos, 8, dir)
                            .filter_map(|p| p.to_pos())
                            .take_while_inclusive(|p| !self.is_square_occupied(*p))
                            .filter(|p| !self.is_square_occupied_by_color(*p, piece.color))
                    })
                    .flatten()
                    .chain(
                        [HorizontalDirection::Left, HorizontalDirection::Right]
                            .into_iter()
                            .map(|dir| {
                                UnboundedPos::horizontal(pos, 8, dir)
                                    .filter_map(|p| p.to_pos())
                                    .take_while_inclusive(|p| !self.is_square_occupied(*p))
                                    .filter(|p| !self.is_square_occupied_by_color(*p, piece.color))
                            })
                            .flatten(),
                    )
                    .chain(
                        [
                            DiagonalDirection::UpLeft,
                            DiagonalDirection::UpRight,
                            DiagonalDirection::DownLeft,
                            DiagonalDirection::DownRight,
                        ]
                        .into_iter()
                        .map(|dir| {
                            UnboundedPos::diagonal(pos, 8, dir)
                                .filter_map(|p| p.to_pos())
                                .take_while_inclusive(|p| !self.is_square_occupied(*p))
                                .filter(|p| !self.is_square_occupied_by_color(*p, piece.color))
                        })
                        .flatten(),
                    )
                    .map(|to| Move::new(pos, to));
                result.extend(moves);
            }
        }
        result
    }

    pub fn is_legal_move(&self, piece: Piece, mv: Move) -> bool {
        match mv {
            Move::Regular { .. } => !self.is_check(mv),
            Move::Castling { side } => {
                let rank = piece.color.king_rank();
                let (king_path, rook_path) = match side {
                    CastlingSide::Short => (
                        vec![Pos::new(File::F, rank), Pos::new(File::G, rank)],
                        vec![Pos::new(File::G, rank), Pos::new(File::F, rank)],
                    ),
                    CastlingSide::Long => (
                        vec![Pos::new(File::D, rank), Pos::new(File::C, rank)],
                        vec![
                            Pos::new(File::B, rank),
                            Pos::new(File::C, rank),
                            Pos::new(File::D, rank),
                        ],
                    ),
                };
                let is_king_path_blocked =
                    king_path.iter().any(|pos| self.is_square_occupied(*pos));
                let is_rook_path_blocked =
                    rook_path.iter().any(|pos| self.is_square_occupied(*pos));
                let is_king_path_attacked = king_path
                    .iter()
                    .any(|pos| self.is_attacked(*pos, !piece.color));

                !is_king_path_attacked && !is_king_path_blocked && !is_rook_path_blocked
            }
        }
    }

    pub fn legal_moves(&self, piece: Piece, pos: Pos) -> Vec<Move> {
        self.available_moves(piece, pos)
            .into_iter()
            .filter(|mv| self.is_legal_move(piece, *mv))
            .collect()
    }

    pub fn is_checkmate(&self, checkmated_side: PieceColor) -> bool {
        let total_legal_moves = self
            .pieces()
            .filter(|(p, _)| p.color == checkmated_side)
            .map(|(p, pos)| self.legal_moves(p, pos))
            .flatten()
            .count();
        self.is_king_attacked(checkmated_side) && total_legal_moves == 0
    }

    pub fn is_stalemate(&self) -> bool {
        let total_legal_moves = self
            .pieces()
            .filter(|(p, _)| p.color == self.turn)
            .map(|(p, pos)| self.legal_moves(p, pos))
            .flatten()
            .count();
        !self.is_king_attacked(self.turn) && total_legal_moves == 0
    }

    pub fn is_attacked(&self, pos: Pos, by: PieceColor) -> bool {
        let attacks = self
            .pieces()
            .filter(|(p, _)| p.color == by)
            .map(|(p, pos)| self.available_moves(p, pos))
            .flatten()
            .filter(|mv| mv.to() == Some(pos));
        attacks.count() > 0
    }

    pub fn is_king_attacked(&self, king_color: PieceColor) -> bool {
        let (_, king_pos) = self
            .pieces()
            .filter(|(p, _)| p.color == king_color && matches!(p.kind, PieceType::King))
            .next()
            .unwrap();
        self.is_attacked(king_pos, !king_color)
    }

    pub fn is_check(&self, mv: Move) -> bool {
        let side = self.turn;
        let mut next_state = self.clone();
        next_state.make_move(mv);
        next_state.is_king_attacked(side)
    }

    pub fn pieces(&self) -> impl Iterator<Item = (Piece, Pos)> + '_ {
        self.squares.iter().enumerate().filter_map(|(i, (s, _))| {
            if let Square::Piece(piece) = s {
                Some((*piece, self.pos_by_square_index(i)))
            } else {
                None
            }
        })
    }

    pub fn is_square_occupied_by_color(&self, pos: Pos, color: PieceColor) -> bool {
        if let Square::Piece(Piece { color: c, .. }) = self.square_by_pos(pos) {
            c == color
        } else {
            false
        }
    }

    pub fn is_square_occupied(&self, pos: Pos) -> bool {
        matches!(self.square_by_pos(pos), Square::Piece(_))
    }

    fn pos_by_square_index(&self, index: usize) -> Pos {
        assert!(index < 64, "Attempt to get pos by invalid index");
        let file = index % 8 + 1;
        let rank = 8 - index / 8;
        let file = File::from_u8(file as u8);
        let rank = Rank::new(rank as u8);
        Pos::new(file, rank)
    }

    pub fn square_by_pos(&self, pos: Pos) -> Square {
        let index = Self::square_index_by_pos(pos);
        self.squares[index].0
    }

    fn square_by_pos_mut(&mut self, pos: Pos) -> &mut Square {
        let index = Self::square_index_by_pos(pos);
        &mut self.squares[index].0
    }

    fn square_index_by_pos(pos: Pos) -> usize {
        let i = ((64 - (pos.rank.get() * 8)) + pos.file.as_u8() - 1) as usize;
        assert!(i < 64, "Attempt to access square outside bounds");
        i
    }

    pub fn cursor_position_to_pos(&self, cursor_position: Point) -> Pos {
        let file = (cursor_position.x * 8.0).ceil() as u8;
        let file = File::from_u8(file);
        let rank = 9 - (cursor_position.y * 8.0).ceil() as u8;
        let rank = Rank::new(rank);
        Pos::new(file, rank)
    }
}
