use std::str::FromStr;

use chess;
use rand::seq::IteratorRandom;

fn main() {

    let mut rng = rand::thread_rng();
    let mut board = chess::Board::default();
    let blind_moves: Vec<_> = chess::MoveGen::new_blind_moves(&board).map(|m| m.to_string()).collect();
    println!("{} {}", blind_moves.len(), blind_moves.join(" "));
    // // Clear the way for kingside castling but through check to show that blind moves allow it
    // for m in "e2e3 b7b6 f1a6 c8a6 g1f3 c7c6".split(' ') {
    //     board = board.make_move_new(chess::ChessMove::from_str(m).unwrap());
    // }
    // let blind_moves: Vec<_> = chess::MoveGen::new_blind_moves(&board).map(|m| m.to_string()).collect();
    // println!("{} {}", blind_moves.len(), blind_moves.join(" "));
    // // We can make the castling move even though it is through check
    // board = board.make_move_new(chess::ChessMove::from_str("e1g1").unwrap());
    // println!("{}, {board:?}", board.to_string());
    // for _ in 0..10 {
    //     let blind_moves: Vec<_> = chess::MoveGen::new_blind_moves(&board).map(|m| m.to_string()).collect();
    //     println!("{} {}", blind_moves.len(), blind_moves.join(" "));
    //     let movegen = chess::MoveGen::new_pseudolegal(&board);
    //     let m = match movegen.choose(&mut rng) {
    //         Some(m) => m,
    //         None => break,
    //     };
    //     println!("{m}");
    //     if m.get_dest() == board.king_square(!board.side_to_move()) {
    //         // println!("{board} {m}");
    //         break
    //     }
    //     board = board.make_move_new(m);
    //     println!("{}", board.to_string());
    // }
}
