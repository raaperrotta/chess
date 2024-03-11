use chess;


fn main() {
    let mut white = chess::RandomPlayer::new();
    let mut black = chess::AttackerPlayer::new(chess::Color::Black);
    let result = chess::play_rbc(&mut white, &mut black);
    match result {
        chess::GameOverReason::KingCapture(color) => println!("{:?} wins by king capture!", color),
        chess::GameOverReason::IllegalMove(color) => println!("{:?} disqualified for requesting an illegal move!", color),
        chess::GameOverReason::FiftyMoveDraw => println!("Draw by the fifty move rule."),
    };
}
