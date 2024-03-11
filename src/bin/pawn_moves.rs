use chess::BitBoard;
use chess::EMPTY;
use chess::Color;
use chess::Square;
use chess::{get_pawn_moves, get_blind_pawn_moves};

fn main() {
    let moves = get_pawn_moves(Square::B2, Color::White, EMPTY);
    println!("{moves}");
    let moves = get_pawn_moves(Square::B2, Color::White, !EMPTY);
    println!("{moves}");
    let moves = get_blind_pawn_moves(Square::B2, Color::White, EMPTY);
    println!("{moves}");
    let moves = get_blind_pawn_moves(Square::B3, Color::White, EMPTY);
    println!("{moves}");
    let moves = get_blind_pawn_moves(Square::B3, Color::White, BitBoard::from_square(Square::C4));
    println!("{moves}");
}
