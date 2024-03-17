use crate::{ChessMove, Color, MoveResult, Player, Rank, SenseResult, Square};

pub struct PassivePlayer {}

impl PassivePlayer {
    pub fn new() -> Self {
        Self {}
    }
}
impl Player for PassivePlayer {
    fn handle_opponent_capture(&mut self, _capture: &Option<Square>) {}
    fn choose_sense(&mut self) -> Square {
        Square::B2
    }
    fn handle_sense_result(&mut self, _sense_result: &SenseResult) {}
    fn choose_move(&mut self) -> Option<ChessMove> {
        None
    }
    fn handle_move_result(&mut self, _result: &MoveResult) {}
}
