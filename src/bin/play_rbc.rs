use chess;


fn main() {
    let mut white = chess::RandomPlayer::new();
    let mut black = chess::RandomPlayer::new();
    let winner = chess::play_rbc(&mut white, &mut black);
    match winner {
        Some(color) => println!("{:?} wins!", color),
        None => println!("Draw!"),
    };
}
