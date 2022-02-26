#![feature(bool_to_option)]

use move_list::MoveList;
use moves::PromotedTo;
use yew::prelude::*;

mod board;
mod fen;
mod move_list;
mod moves;
mod piece;
mod pos;
mod take_while;

use crate::board::Board;
use crate::board::BoardState;
use crate::board::GameResult;
use crate::board::Square;
use crate::moves::internal::Move;
use crate::moves::CastlingSide;
use crate::piece::{Piece, PieceColor, PieceType};
use crate::pos::{File, Pos, Rank};

pub enum Msg {
    ClickOnSquare(Pos),
}

struct Model {}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
             <Board />
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}

#[derive(Debug, Clone)]
pub enum Message {
    ClickOnSquare(Pos),
    ShowPromotionMenu(File, PieceColor),
    ClosePromotionMenu(Option<PromotedTo>),
}

struct App {
    board: BoardState,
    moves_list: MoveList,
}

impl App {

    fn title(&self) -> String {
        let side_to_move = self.board.turn;
        let game_result = match self.board.game_result {
            Some(GameResult::WinByCheckmate { .. }) => format!("Checkmated"),
            Some(GameResult::DrawByStalemate) => format!("Draw by stalemate"),
            _ => String::new(),
        };
        let move_number = self.board.move_number();
        format!("Chess Explorer | {move_number}. {side_to_move} {game_result}")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ClickOnSquare(pos) => {
                self.board.stop_highlighting();
                if let Some((piece, from)) = self.board.selected_piece.take() {
                    let mv = if piece.kind == PieceType::King {
                        let king_rank = piece.color.king_rank();
                        let king_home = piece.color.king_home();
                        if from == king_home
                            && pos == Pos::new(File::G, king_rank)
                            && self.board.is_castling_possible(CastlingSide::Short)
                        {
                            Move::castling(CastlingSide::Short)
                        } else if from == king_home
                            && pos == Pos::new(File::C, king_rank)
                            && self.board.is_castling_possible(CastlingSide::Long)
                        {
                            Move::castling(CastlingSide::Long)
                        } else {
                            Move::new(from, pos)
                        }
                    } else if piece.kind == PieceType::Pawn
                        && (pos.rank == (!piece.color).king_rank())
                    {
                        Move::new_with_promoted(from, pos, Some(PromotedTo::Queen))
                    } else {
                        Move::new(from, pos)
                    };
                    if self.board.available_moves(piece, from).contains(&mv) {
                        if self.board.is_legal_move(piece, mv) {
                            let san_move = self.board.to_san_move(mv);
                            // check roundtrip
                            let expected = self.board.from_san_move(san_move);
                            assert_eq!(expected, mv);
                            self.board.make_move(mv);
                            self.moves_list.add_move(san_move);
                            if self.board.is_checkmate(self.board.turn) {
                                self.board.game_result = Some(GameResult::WinByCheckmate {
                                    checkmated_side: self.board.turn,
                                });
                            }
                            if self.board.is_stalemate() {
                                self.board.game_result = Some(GameResult::DrawByStalemate);
                            }
                        }
                    }
                } else if let Square::Piece(piece) = self.board.square_by_pos(pos) {
                    if piece.color == self.board.turn {
                        self.board.selected_piece = Some((piece, pos));
                        self.board.hightlight_legal_moves(piece, pos);
                    }
                }
            }
            Message::ShowPromotionMenu(file, side) => {}
            Message::ClosePromotionMenu(prototed) => {}
        }
        ()
    }

}
