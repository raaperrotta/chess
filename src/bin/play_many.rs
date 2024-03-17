use chess;
use indicatif::ProgressIterator;
use num_format::{Locale, ToFormattedString};
use std::time::Instant;

fn main() {
    let mut white_wins = 0;
    let mut black_wins = 0;
    let start = Instant::now();
    for _ in (0..100_000).progress() {
        let mut white = chess::RandomPlayer::new();
        let mut black = chess::RandomPlayer::new();
        let result = chess::play_rbc(&mut white, &mut black);
        match result {
            chess::GameOverReason::KingCapture(chess::Color::White) => white_wins += 1,
            chess::GameOverReason::KingCapture(chess::Color::Black) => black_wins += 1,
            chess::GameOverReason::IllegalMove(chess::Color::White) => black_wins += 1,
            chess::GameOverReason::IllegalMove(chess::Color::Black) => white_wins += 1,
            chess::GameOverReason::FiftyMoveDraw => (),
        };
    }
    println!(
        "White won {} and Black won {} in {:?}",
        white_wins.to_formatted_string(&Locale::en),
        black_wins.to_formatted_string(&Locale::en),
        start.elapsed()
    );
}
