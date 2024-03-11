use chess::BitBoard;
use chess::EMPTY;
use chess::Color;
use chess::Square;
use chess::{get_bishop_moves, get_rook_moves};

fn main() {
    let moves = get_bishop_moves(Square::E4, EMPTY);
    println!("{moves}");
    let moves = get_bishop_moves(Square::E4, BitBoard::from_square(Square::C2));
    println!("{moves}");
    let moves = get_bishop_moves(Square::E4, BitBoard::from_square(Square::C2)) & !BitBoard::from_square(Square::C2);
    println!("{moves}");
    let moves = get_rook_moves(Square::E4, EMPTY);
    println!("{moves}");
    let moves = get_rook_moves(Square::E4, BitBoard::from_square(Square::C4));
    println!("{moves}");

    let board = chess::Board::default();
    let my_pieces = *board.color_combined(board.side_to_move());
    let moves = get_bishop_moves(Square::C1, my_pieces);
    println!("{moves}");
    let moves = get_bishop_moves(Square::C1, my_pieces) & !my_pieces;
    println!("{moves}");

}
