#![feature(bool_to_option)]

use move_list::MoveList;
use moves::PromotedTo;
use std::cell::RefCell;
use std::rc::Rc;
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
use crate::moves::san;
use crate::moves::CastlingSide;
use crate::piece::{Piece, PieceColor, PieceType};
use crate::pos::{File, Pos, Rank};

#[derive(Debug, PartialEq)]
pub enum Msg {
    ClickOnSquare(Pos),
    AddMoveToMovelist(san::Move),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Moves {
    pub inner: Rc<RefCell<Vec<san::Move>>>,
}

impl Moves {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn push(&self, mv: san::Move) {
        self.inner.borrow_mut().push(mv);
    }
}

#[function_component(Model)]
fn model() -> Html {
    let moves = Moves::new();
    html! {
        <div class={classes!("h-full", "w-full", "flex", "flex-row")}>
            <div class={classes!("w-8/12", "h-full", "basis-3/4")}>
                <Board move_list={moves.clone()}/>
            </div>
            <div class={classes!("w-4/12", "h-full", "basis-1/4")}>
                <MoveList move_list={moves.clone()}/>
            </div>
        </div>
    }
}

fn main() {
    yew::start_app::<Model>();
}
