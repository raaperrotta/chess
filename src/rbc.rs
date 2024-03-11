use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use crate::{between, get_bishop_rays, line, Color, magic::get_sense_mask, square, BitBoard, Board, ChessMove, MoveGen, Piece, Square, EMPTY};


pub struct SenseResult {
    pawn: BitBoard,
    rook: BitBoard,
    knight: BitBoard,
    bishop: BitBoard,
    queen: BitBoard,
    king: BitBoard,
}

#[derive(Debug)]
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

pub struct RandomPlayer{
    rng: ThreadRng,
    board: Board,
}
impl RandomPlayer {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            board: Board::default(),
        }
    }
}
impl Player for RandomPlayer {
    fn handle_opponent_capture(&mut self, capture: &Option<Square>) {
        self.board.null_move_mut();
        if let Some(square) = capture {
            self.board = self.board.clear_square(*square).expect(&format!("expected to find a piece at opponent capture square {square}"));
        }
    }
    fn choose_sense(&mut self) -> Square { Square::B2 }
    fn handle_sense_result(&mut self, _sense_result: &SenseResult) {}
    fn choose_move(&mut self) -> Option<ChessMove> { 
        let mut moves: Vec<_> = MoveGen::new_blind_moves(&self.board).map(|m| Some(m)).collect();
        moves.push(None);
        let move_string: Vec<_> = moves.iter().map(|m| m.map(|m| m.to_string())).collect();
        println!("{:?}", move_string);
        *moves.iter().choose(&mut self.rng).unwrap()
    }
    fn handle_move_result(&mut self, result: &MoveResult) {
        match result.taken_move {
            Some(taken_move) => self.board.make_move_mut(taken_move),
            None => self.board.null_move_mut(),
        };
    }
}

fn simulate_simple_move(board: &Board, requested_move: ChessMove) -> MoveResult {
    let dest = requested_move.get_dest();
    let capture_square = match BitBoard::from_square(dest) & board.color_combined(!board.side_to_move()) {
        EMPTY => None,
        _ => Some(dest)
    };
    MoveResult { taken_move: Some(requested_move), capture_square: capture_square }
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
            return MoveResult { taken_move: Some(ChessMove::new(source, square, None)), capture_square: Some(square) }
        }
    }
    MoveResult { taken_move: Some(requested_move), capture_square: None }
}

fn simulate_pawn_move(board: &Board, requested_move: ChessMove) -> MoveResult {
    let source = requested_move.get_source();
    let dest = requested_move.get_dest();
    if source.get_file() == dest.get_file() {
        simulate_sliding_move(board, requested_move)
    } else {
        let ep_sq = board.en_passant();
        if board.piece_on(dest).is_some() {
            MoveResult { taken_move: Some(requested_move), capture_square: Some(dest) }
        } else if ep_sq.is_some() && dest.ubackward(board.side_to_move()) == ep_sq.unwrap() {
            MoveResult { taken_move: Some(requested_move), capture_square: ep_sq }
        } else {
            MoveResult { taken_move: None, capture_square: None }
        }
    }
}

/// This assumes the move is a valid blind move! Behavior otherwise is not defined!
pub fn simulate_move(board: &Board, requested_move: Option<ChessMove>) -> MoveResult {
    let Some(requested_move) = requested_move else {
        return MoveResult { taken_move: None, capture_square: None };
    };
    let source = requested_move.get_source();
    let piece = board.piece_on(source).expect(&format!("there is no piece to satisfy requested move {requested_move}!"));
    match piece {
        Piece::Pawn => simulate_pawn_move(board, requested_move),
        Piece::Knight => simulate_simple_move(board, requested_move),
        Piece::Bishop => simulate_sliding_move(board, requested_move),
        Piece::Rook => simulate_sliding_move(board, requested_move),
        Piece::Queen => simulate_sliding_move(board, requested_move),
        Piece::King => simulate_simple_move(board, requested_move),
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

pub fn do_sense<T>(board: &mut Board, active: &mut T)
where T: Player {
    let sense = active.choose_sense();
    let result = simulate_sense(&board, sense);
    active.handle_sense_result(&result);
}

pub fn do_move<T>(board: &mut Board, active: &mut T, passive: &mut T)
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

pub fn do_half_turn<T>(board: &mut Board, active: &mut T, passive: &mut T)
where T: Player {
    do_sense(board, active);
    do_move(board, active, passive);
}

pub fn play_rbc<T>(white: &mut T, black: &mut T) -> Option<Color>
where T: Player {
    let mut board = Board::default();
    // TODO add 50 move rule (pawn move or capture resets count)

    println!("{}", board.to_string());
    do_move(&mut board, white, black);

    let mut active = black;
    let mut passive = white;
    loop {
        println!("{}", board.to_string());
        do_half_turn(&mut board, active, passive);

        if (board.pieces(Piece::King) & board.color_combined(board.side_to_move())) == EMPTY {
            return Some(!board.side_to_move());
        }

        let tmp = active;
        active = passive;
        passive = tmp;
    }
}
