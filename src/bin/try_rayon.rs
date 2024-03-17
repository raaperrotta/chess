use chess;
use indicatif::ParallelProgressIterator;
use num_format::{Locale, ToFormattedString};
use rayon::prelude::*;
use std::time::Instant;

fn main() {
    let mut white_wins = 0;
    let mut black_wins = 0;
    let start = Instant::now();
    let results: Vec<_> = (0..100_000)
        .into_par_iter()
        .progress()
        .map(|_| {
            let mut white = chess::RandomPlayer::new();
            let mut black = chess::RandomPlayer::new();
            chess::play_rbc(&mut white, &mut black)
        })
        .collect();
    for result in results {
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
