use std::time::Instant;
use chess;
use num_format::{Locale, ToFormattedString};
use rand::seq::IteratorRandom;

fn main() {

    let num_games = 100;
    let max_moves_per_game = 100;
    let mut rng = rand::thread_rng();

    for _ in 0..num_games {
        let mut board = chess::Board::default();
        for _ in 0..max_moves_per_game {
            let movegen = chess::MoveGen::new_pseudolegal(&board);
            let m = match movegen.choose(&mut rng) {
                Some(m) => m,
                None => break,
            };
            if m.get_dest() == board.king_square(!board.side_to_move()) {
                // println!("{board} {m}");
                break
            }
            // board = board.make_move_new(m);
            // println!("{board} {m}");
            let result = board.make_move_new(m);
            board.make_move_mut(m);
            if board != result {
                println!("{result}");
                println!("{board}");
                println!("{result:?}");
                println!("{board:?}");
                return;
            }
        }
    }

    let num_games = 10_000;
    let max_moves_per_game = 100;
    let start = Instant::now();
    for _ in 0..num_games {
        let mut board = chess::Board::default();
        for _ in 0..max_moves_per_game {
            let movegen = chess::MoveGen::new_pseudolegal(&board);
            let m = match movegen.choose(&mut rng) {
                Some(m) => m,
                None => break,
            };
            if m.get_dest() == board.king_square(!board.side_to_move()) {
                break
            }
            board = board.make_move_new(m);
        }
    }
    println!("Made {} random moves in {:?}", (num_games * max_moves_per_game).to_formatted_string(&Locale::en), start.elapsed());
    let start = Instant::now();
    for _ in 0..num_games {
        let mut board = chess::Board::default();
        for _ in 0..max_moves_per_game {
            let movegen = chess::MoveGen::new_pseudolegal(&board);
            let m = match movegen.choose(&mut rng) {
                Some(m) => m,
                None => break,
            };
            if m.get_dest() == board.king_square(!board.side_to_move()) {
                break
            }
            board.make_move_mut(m);
        }
    }
    println!("Made {} random moves in {:?}", (num_games * max_moves_per_game).to_formatted_string(&Locale::en), start.elapsed());
}
