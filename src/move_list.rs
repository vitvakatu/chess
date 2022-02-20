use crate::Message;
use crate::moves::san::Move;
use crate::board::BoardState;
use iced::{Text, Column, Row};

pub struct MoveList {
    moves: Vec<Move>,
    simulator: BoardState,
}

impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: Vec::new(),
            simulator: BoardState::new(),
        }
    }

    pub fn add_move(&mut self, mv: Move) {
        self.moves.push(mv);
    }

    pub fn view(&self) -> iced::Element<Message> {
        let rows = self.moves.chunks(2).enumerate().map(|(i, chunk)| {
            let first = chunk.get(0).map(ToString::to_string).unwrap();
            let second = chunk.get(1).map(ToString::to_string).unwrap_or(String::new());
            let number = format!("{i}. ", i=i+1);
            Row::new().spacing(20).push(Text::new(number)).push(Text::new(first)).push(Text::new(second)).into()
        }).collect();
        Column::with_children(rows).into()
    }
}
