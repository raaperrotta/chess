use chess::{self, simulate_move, simulate_sense, Player, SenseResult, Board, EMPTY, Piece};

fn do_sense<T>(board: &mut Board, active: &mut T)
where T: Player {
    let sense = active.choose_sense();
    let result = simulate_sense(&board, sense);
    active.handle_sense_result(&result);
}

fn do_move<T>(board: &mut Board, active: &mut T, passive: &mut T)
where T: Player {
    let requested_move = active.choose_move();
    println!("{:?}", requested_move.map(|m| m.to_string()));
    let result = simulate_move(&board, requested_move);
    println!("{:?} {:?}", result.taken_move.map(|m| m.to_string()), result.capture_square.map(|s| s.to_string()));
    match result.taken_move {
        Some(m) => board.make_move_mut(m),
        None => board.null_move_mut(),
    }
    println!("{}", board.to_string());
    active.handle_move_result(&result);
    passive.handle_opponent_capture(&result.capture_square);
}

fn main() {
    let mut white = chess::RandomPlayer::new();
    let mut black = chess::RandomPlayer::new();
    let mut board = chess::Board::default();
    println!("{}", board.to_string());
    do_move(&mut board, &mut white, &mut black);
    let mut active = &mut black;
    let mut passive = &mut white;
    loop {
        println!("{}", board.to_string());
        do_sense(&mut board, active);
        do_move(&mut board, active, passive);

        if (board.pieces(Piece::King) & board.color_combined(board.side_to_move())) == EMPTY {
            println!("{:?} wins!", !board.side_to_move());
            break
        }

        let tmp = active;
        active = passive;
        passive = tmp;
    }
}
