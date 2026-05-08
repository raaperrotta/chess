use std::str::FromStr;

use chess::{self, simulate_move};

fn main() {
    // Demonstrate simulate_move (the validating wrapper) on a few requests.
    let mut board = chess::Board::default();
    let m = chess::ChessMove::from_str("b1c3").unwrap();
    println!("{:?}", simulate_move(&board, Some(m)));

    // a1a8: rook on a1 with several blockers; in-game illegal -> Err.
    let m = chess::ChessMove::from_str("a1a8").unwrap();
    println!("{:?}", simulate_move(&board, Some(m)));

    // Set up a position where a3f8 is a long bishop slide.
    board.make_move_mut(chess::ChessMove::from_str("b2b3").unwrap());
    board.make_move_mut(chess::ChessMove::from_str("d7d5").unwrap());
    board.make_move_mut(chess::ChessMove::from_str("c1a3").unwrap());
    board.make_move_mut(chess::ChessMove::from_str("h7h5").unwrap());
    let m = chess::ChessMove::from_str("a3f8").unwrap();
    println!("{:?}", simulate_move(&board, Some(m)));
}
