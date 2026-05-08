//! Plays a single Reconnaissance Blind Chess game between two random
//! players and prints the outcome.
//!
//! Run with:
//!
//!     cargo run --example play_one_game

use chess::{play_rbc, Color, GameOverReason, RandomPlayer};

fn main() {
    let mut white = RandomPlayer::new();
    let mut black = RandomPlayer::new();
    let result = play_rbc(&mut white, &mut black);

    match result {
        GameOverReason::KingCapture(Color::White) => println!("White wins by king capture"),
        GameOverReason::KingCapture(Color::Black) => println!("Black wins by king capture"),
        GameOverReason::IllegalMove(c) => {
            println!("{:?} disqualified for an invalid move request", c)
        }
        GameOverReason::FiftyMoveDraw => println!("Draw by the 50-move rule"),
    }
}
