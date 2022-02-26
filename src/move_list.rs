use crate::Message;
use crate::moves::san::Move;
use crate::board::BoardState;

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

}
