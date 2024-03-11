use std::str::FromStr;
use rand::seq::IteratorRandom;
use crate::{ChessMove, Square, SenseResult, MoveResult, Player, Color, Rank};

fn get_attacks(color: Color) -> Vec<ChessMove> {
    let mut rng = rand::thread_rng();
    let mut attacks: Vec<Vec<ChessMove>> = Vec::new();
    for attack in [
        // Queen-side knight attacks
        "b1c3 c3b5 b5d6 d6e8",
        "b1c3 c3e4 e4f6 f6e8",
        // King-side knight attack
        "g1h3 h3f4 f4h5 h5f6 f6e8",
        // Four-move mate
        "e2e4 f1c4 d1h5 c4f7 f7e8 h5e8",
    ] {
        let mut attack_vec: Vec<ChessMove> = attack.to_owned().split(" ").map(|s| ChessMove::from_str(s).unwrap()).collect();
        attack_vec.reverse(); // Flip it so first moves are popped from end of vec
        attacks.push(attack_vec);
    }
    let attacks = match color {
        Color::White => attacks,
        Color::Black => attacks.iter().map(|a| {
            a.iter().map(|m| {
                let source = m.get_source();
                let source = Square::make_square(Rank::from_index(7 - source.get_rank().to_index()), source.get_file());
                let dest = m.get_dest();
                let dest = Square::make_square(Rank::from_index(7 - dest.get_rank().to_index()), dest.get_file());
                ChessMove::new(source, dest, m.get_promotion())
            }).collect()
        }).collect(),
    };
    attacks.iter().choose(&mut rng).unwrap().to_vec()
}

pub struct AttackerPlayer{
    moves: Vec<ChessMove>,
}
impl AttackerPlayer {
    pub fn new(color: Color) -> Self {
        Self {
            moves: get_attacks(color)
        }
    }
}
impl Player for AttackerPlayer {
    fn handle_opponent_capture(&mut self, _capture: &Option<Square>) {}
    fn choose_sense(&mut self) -> Square { Square::B2 }
    fn handle_sense_result(&mut self, _sense_result: &SenseResult) {}
    fn choose_move(&mut self) -> Option<ChessMove> { 
        self.moves.pop()
    }
    fn handle_move_result(&mut self, _result: &MoveResult) {}
}
