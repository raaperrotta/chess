use crate::{
    between, magic::get_sense_mask, movegen::PieceType, BitBoard, Board, ChessMove, Color, MoveGen,
    Piece, Square, EMPTY,
};

pub const SENSE_SQUARES: [Square;36] = [
    Square::B2,
    Square::B3,
    Square::B4,
    Square::B5,
    Square::B6,
    Square::B7,
    Square::C2,
    Square::C3,
    Square::C4,
    Square::C5,
    Square::C6,
    Square::C7,
    Square::D2,
    Square::D3,
    Square::D4,
    Square::D5,
    Square::D6,
    Square::D7,
    Square::E2,
    Square::E3,
    Square::E4,
    Square::E5,
    Square::E6,
    Square::E7,
    Square::F2,
    Square::F3,
    Square::F4,
    Square::F5,
    Square::F6,
    Square::F7,
    Square::G2,
    Square::G3,
    Square::G4,
    Square::G5,
    Square::G6,
    Square::G7,
];

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct SenseResult {
    pawn: BitBoard,
    rook: BitBoard,
    knight: BitBoard,
    bishop: BitBoard,
    queen: BitBoard,
    king: BitBoard,
}

#[derive(Debug, PartialEq)]
pub struct MoveResult {
    pub taken_move: Option<ChessMove>,
    pub capture_square: Option<Square>,
}

pub trait Player {
    fn handle_opponent_capture(&mut self, capture: &Option<Square>);
    fn choose_sense(&mut self) -> Square;
    fn handle_sense_result(&mut self, sense_result: &SenseResult);
    fn choose_move(&mut self) -> Option<ChessMove>;
    fn handle_move_result(&mut self, result: &MoveResult);
}

fn simulate_simple_move(board: &Board, requested_move: ChessMove) -> MoveResult {
    let dest = requested_move.get_dest();
    let capture_square =
        match BitBoard::from_square(dest) & board.color_combined(!board.side_to_move()) {
            EMPTY => None,
            _ => Some(dest),
        };
    MoveResult {
        taken_move: Some(requested_move),
        capture_square: capture_square,
    }
}

fn simulate_sliding_move(board: &Board, requested_move: ChessMove) -> MoveResult {
    let source = requested_move.get_source();
    let dest = requested_move.get_dest();
    let source_bb = BitBoard::from_square(source);
    let dest_bb = BitBoard::from_square(dest);
    let between_bb = between(source, dest);
    let combined_bb = source_bb ^ dest_bb ^ between_bb;
    let blockers = board.color_combined(!board.side_to_move());
    let mut squares: Vec<_> = combined_bb.collect();
    if *squares.get(0).unwrap() != source {
        squares.reverse();
    }
    for square in squares.into_iter().skip(1) {
        if BitBoard::from_square(square) & blockers != EMPTY {
            return MoveResult {
                taken_move: Some(ChessMove::new(source, square, None)),
                capture_square: Some(square),
            };
        }
    }
    MoveResult {
        taken_move: Some(requested_move),
        capture_square: None,
    }
}

fn simulate_pawn_move(board: &Board, requested_move: ChessMove) -> MoveResult {
    let source = requested_move.get_source();
    let dest = requested_move.get_dest();
    if source.get_file() == dest.get_file() {
        simulate_sliding_move(board, requested_move)
    } else {
        let ep_sq = board.en_passant();
        if board.piece_on(dest).is_some() {
            MoveResult {
                taken_move: Some(requested_move),
                capture_square: Some(dest),
            }
        } else if ep_sq.is_some() && dest.ubackward(board.side_to_move()) == ep_sq.unwrap() {
            MoveResult {
                taken_move: Some(requested_move),
                capture_square: ep_sq,
            }
        } else {
            MoveResult {
                taken_move: None,
                capture_square: None,
            }
        }
    }
}

fn simulate_king_move(board: &Board, requested_move: ChessMove) -> MoveResult {
    let source = requested_move.get_source();
    let dest = requested_move.get_dest();
    let bitboard = between(source, dest);
    if bitboard == EMPTY {
        // Not a castling move
        simulate_simple_move(board, requested_move)
    } else if (bitboard & board.combined()) == EMPTY {
        // Castling with nothing in the way
        MoveResult {
            taken_move: Some(requested_move),
            capture_square: None,
        }
    } else {
        // Castling with something in the way
        MoveResult {
            taken_move: None,
            capture_square: None,
        }
    }
}

/// This assumes the move is a valid blind move! Behavior otherwise is not defined!
pub fn capture_square(board: &Board, chess_move: Option<ChessMove>) -> Option<Square> {
    let Some(chess_move) = chess_move else {
        return None;
    };
    let source = chess_move.get_source();
    let dest = chess_move.get_dest();
    if board.en_passant().is_some() && board.piece_on(source).unwrap() == Piece::Pawn && dest == board.en_passant().unwrap() {
        board.en_passant()
    } else if board.piece_on(dest).is_some() {
        Some(dest)
    } else {
        None
    }
}

/// This assumes the move is a valid blind move! Behavior otherwise is not defined!
pub fn simulate_move(board: &Board, requested_move: Option<ChessMove>) -> MoveResult {
    let Some(requested_move) = requested_move else {
        return MoveResult {
            taken_move: None,
            capture_square: None,
        };
    };
    let source = requested_move.get_source();
    let piece = board.piece_on(source).unwrap();
    match piece {
        Piece::Pawn => simulate_pawn_move(board, requested_move),
        Piece::Knight => simulate_simple_move(board, requested_move),
        Piece::Bishop => simulate_sliding_move(board, requested_move),
        Piece::Rook => simulate_sliding_move(board, requested_move),
        Piece::Queen => simulate_sliding_move(board, requested_move),
        Piece::King => simulate_king_move(board, requested_move),
    }
}

pub fn simulate_sense(board: &Board, sense: Square) -> SenseResult {
    let sense_bb = get_sense_mask(sense);
    let opponent_pieces = board.color_combined(!board.side_to_move());
    SenseResult {
        pawn: board.pieces(Piece::Pawn) & opponent_pieces & sense_bb,
        rook: board.pieces(Piece::Rook) & opponent_pieces & sense_bb,
        knight: board.pieces(Piece::Knight) & opponent_pieces & sense_bb,
        bishop: board.pieces(Piece::Bishop) & opponent_pieces & sense_bb,
        queen: board.pieces(Piece::Queen) & opponent_pieces & sense_bb,
        king: board.pieces(Piece::King) & opponent_pieces & sense_bb,
    }
}

impl Board {
    fn sense(&self, square: Square) -> SenseResult {
        simulate_sense(self, square)
    }
    fn correct_move(&self, requested_move: Option<ChessMove>) -> MoveResult {
        simulate_move(self, requested_move)
    }
}

pub enum MoveType {
    NonZeroing,
    Zeroing,
}

pub fn do_sense<T>(board: &mut Board, active: &mut T)
where
    T: Player,
{
    let sense = active.choose_sense();
    let result = board.sense(sense);
    // let result = simulate_sense(&board, sense);
    active.handle_sense_result(&result);
}

pub fn do_move<T, S>(
    board: &mut Board,
    active: &mut T,
    passive: &mut S,
) -> Result<MoveType, &'static str>
where
    T: Player,
    S: Player,
{
    let mut allowed_moves = MoveGen::new_blind_moves(board);
    let requested_move = active.choose_move();
    if requested_move.is_some() && !allowed_moves.any(|m| m == requested_move.unwrap()) {
        return Err("Player requested a move that was not allowed!");
    }
    let result = board.correct_move(requested_move);
    // let result = simulate_move(&board, requested_move);
    // println!("{:?} {:?}", result.taken_move.map(|m| m.to_string()), result.capture_square.map(|s| s.to_string()));
    let move_type = match result.taken_move {
        Some(m) => {
            board.make_move_mut(m);
            if result.capture_square.is_some()
                || (board.piece_on(m.get_source()) == Some(Piece::Pawn))
            {
                MoveType::Zeroing
            } else {
                MoveType::NonZeroing
            }
        }
        None => {
            board.null_move_mut();
            MoveType::NonZeroing
        }
    };
    // println!("{}", board.to_string());
    active.handle_move_result(&result);
    // TODO end early if the game is over?
    passive.handle_opponent_capture(&result.capture_square);
    Ok(move_type)
}

pub fn do_half_turn<T, S>(
    board: &mut Board,
    active: &mut T,
    passive: &mut S,
) -> Result<MoveType, &'static str>
where
    T: Player,
    S: Player,
{
    do_sense(board, active);
    do_move(board, active, passive)
}

pub enum GameOverReason {
    KingCapture(Color), // Color of the player that did the capturing (the winner)
    IllegalMove(Color), // Color of the player that made the illegal move request (the loser)
    FiftyMoveDraw,
}

pub fn play_rbc<T, S>(white: &mut T, black: &mut S) -> GameOverReason
where
    T: Player,
    S: Player,
{
    let mut board = Board::default();
    let mut halfmove_count = 0;
    // TODO add 50 move rule (pawn move or capture resets count)

    // println!("{}", board.to_string());
    let result = do_move(&mut board, white, black);
    match result {
        Err(_) => return GameOverReason::IllegalMove(Color::White),
        Ok(MoveType::Zeroing) => halfmove_count = 0,
        Ok(MoveType::NonZeroing) => halfmove_count += 1,
    }

    loop {
        // println!("{}", board.to_string());
        let result = do_half_turn(&mut board, black, white);
        match result {
            Err(_) => return GameOverReason::IllegalMove(Color::Black),
            Ok(MoveType::Zeroing) => halfmove_count = 0,
            Ok(MoveType::NonZeroing) => halfmove_count += 1,
        }
        if halfmove_count >= 100 {
            return GameOverReason::FiftyMoveDraw;
        }

        if (board.pieces(Piece::King) & board.color_combined(board.side_to_move())) == EMPTY {
            return GameOverReason::KingCapture(Color::Black);
        }

        // println!("{}", board.to_string());
        let result = do_half_turn(&mut board, white, black);
        match result {
            Err(_) => return GameOverReason::IllegalMove(Color::Black),
            Ok(MoveType::Zeroing) => halfmove_count = 0,
            Ok(MoveType::NonZeroing) => halfmove_count += 1,
        }
        if halfmove_count >= 100 {
            return GameOverReason::FiftyMoveDraw;
        }

        if (board.pieces(Piece::King) & board.color_combined(board.side_to_move())) == EMPTY {
            return GameOverReason::KingCapture(Color::White);
        }
    }
}
