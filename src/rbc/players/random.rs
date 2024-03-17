use crate::{Board, ChessMove, MoveGen, MoveResult, Player, SenseResult, Square};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;

pub struct RandomPlayer {
    rng: ThreadRng,
    board: Board,
}
impl RandomPlayer {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            board: Board::default(),
        }
    }
}
impl Player for RandomPlayer {
    fn handle_opponent_capture(&mut self, capture: &Option<Square>) {
        self.board.null_move_mut();
        if let Some(square) = capture {
            self.board = self.board.clear_square(*square).unwrap();
        }
    }
    fn choose_sense(&mut self) -> Square {
        Square::B2
    }
    fn handle_sense_result(&mut self, _sense_result: &SenseResult) {}
    fn choose_move(&mut self) -> Option<ChessMove> {
        let mut moves: Vec<_> = MoveGen::new_blind_moves(&self.board)
            .map(|m| Some(m))
            .collect();
        moves.push(None);
        *moves.iter().choose(&mut self.rng).unwrap()
    }
    fn handle_move_result(&mut self, result: &MoveResult) {
        match result.taken_move {
            Some(taken_move) => self.board.make_move_mut(taken_move),
            None => self.board.null_move_mut(),
        };
    }
}
