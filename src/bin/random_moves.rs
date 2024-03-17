use chess;
use num_format::{Locale, ToFormattedString};
use rand::{seq::IteratorRandom, Rng, RngCore};
use std::time::Instant;

fn main() {
    let mut rng = rand::thread_rng();

    let num_games = 10_000;
    let max_moves_per_game = 1_000;

    let start = Instant::now();
    let mut num_moves: usize = 0;
    for _ in 0..num_games {
        let mut board = chess::Board::default();
        for _ in 0..max_moves_per_game {
            let movegen = chess::MoveGen::new_pseudolegal(&board);
            let m = match movegen.choose(&mut rng) {
                Some(m) => m,
                None => break,
            };
            num_moves += 1;
            if m.get_dest() == board.king_square(!board.side_to_move()) {
                break;
            }
            board = board.make_move_new(m);
        }
    }
    println!(
        "Made {} random moves in {:?}",
        num_moves.to_formatted_string(&Locale::en),
        start.elapsed()
    );

    
    let start = Instant::now();
    let mut num_moves: usize = 0;
    for _ in 0..num_games {
        let mut board = chess::Board::default();
        for _ in 0..max_moves_per_game {
            let movegen = chess::MoveGen::new_pseudolegal(&board);
            let m = match movegen.choose(&mut rng) {
                Some(m) => m,
                None => break,
            };
            num_moves += 1;
            if m.get_dest() == board.king_square(!board.side_to_move()) {
                break;
            }
            board.make_move_mut(m);
        }
    }
    println!(
        "Made {} random moves in {:?}",
        num_moves.to_formatted_string(&Locale::en),
        start.elapsed()
    );

    
    let start = Instant::now();
    let mut num_moves: usize = 0;
    for _ in 0..num_games {
        let mut board = chess::Board::default();
        for _ in 0..max_moves_per_game {
            let movegen: Vec<_> = chess::MoveGen::new_pseudolegal(&board).collect();
            let m = match movegen.into_iter().choose(&mut rng) {
                Some(m) => m,
                None => break,
            };
            num_moves += 1;
            if m.get_dest() == board.king_square(!board.side_to_move()) {
                break;
            }
            board.make_move_mut(m);
        }
    }
    println!(
        "Made {} random moves in {:?}",
        num_moves.to_formatted_string(&Locale::en),
        start.elapsed()
    );
}
