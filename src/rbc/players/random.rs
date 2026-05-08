use crate::{Board, ChessMove, MoveGen, MoveResult, Player, SenseResult, Square};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;

pub struct RandomPlayer {
    rng: ThreadRng,
    board: Board,
}
impl RandomPlayer {
    /// Construct a `RandomPlayer` starting from the standard initial
    /// chess position.
    pub fn new() -> Self {
        Self::with_board(Board::default())
    }

    /// Construct a `RandomPlayer` starting from an arbitrary mid-game
    /// position. Useful for tournament play where you want the player to
    /// resume from a known board state, e.g. after replaying a saved game.
    pub fn with_board(board: Board) -> Self {
        Self {
            rng: rand::rng(),
            board,
        }
    }
}
impl Player for RandomPlayer {
    fn handle_opponent_capture(&mut self, capture: &Option<Square>) {
        self.board.null_move_mut();
        if let Some(square) = capture {
            // clear_square is marked deprecated by upstream because it
            // doesn't validate the resulting position, but for RandomPlayer
            // we don't care about validity; the player only uses its
            // internal board to enumerate move requests.
            #[allow(deprecated)]
            {
                self.board = self.board.clear_square(*square).unwrap();
            }
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
