use chess;
use rand::seq::IteratorRandom;

fn main() {
    let mut rng = rand::thread_rng();

    let mut board = chess::Board::default();
    println!("{}", board.to_fancy_string());
    for _ in 0..100 {
        let movegen = chess::MoveGen::new_pseudolegal(&board);
        let m = match movegen.choose(&mut rng) {
            Some(m) => m,
            None => break,
        };
        println!("{}", m);
        if m.get_dest() == board.king_square(!board.side_to_move()) {
            // println!("{board} {m}");
            break;
        }
        board.make_move_mut(m);
        println!("{}", board.to_fancy_string());
    }
}
