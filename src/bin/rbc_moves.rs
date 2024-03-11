use std::str::FromStr;

use chess::{self, simulate_move};

fn main() {
    let mut board = chess::Board::default();
    let m = chess::ChessMove::from_str("b1c3").unwrap();
    let t = simulate_move(&board, Some(m));
    println!("{:?}", t);
    let m = chess::ChessMove::from_str("a1a8").unwrap();
    let t = simulate_move(&board, Some(m));
    println!("{:?}", t);
    board.make_move_mut(chess::ChessMove::from_str("b2b3").unwrap());
    board.make_move_mut(chess::ChessMove::from_str("d7d5").unwrap());
    board.make_move_mut(chess::ChessMove::from_str("c1a3").unwrap());
    board.make_move_mut(chess::ChessMove::from_str("h7h5").unwrap());
    let m = chess::ChessMove::from_str("a3f8").unwrap();
    let t = simulate_move(&board, Some(m));
    println!("{:?}", t);
}
