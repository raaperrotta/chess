use crate::bitboard::{BitBoard, EMPTY};
use crate::board::Board;
use crate::chess_move::ChessMove;
use crate::magic::between;
use crate::movegen::piece_type::*;
use crate::piece::{Piece, NUM_PROMOTION_PIECES, PROMOTION_PIECES};
use crate::square::Square;
use arrayvec::ArrayVec;
use nodrop::NoDrop;
use std::iter::ExactSizeIterator;

#[derive(Copy, Clone, PartialEq, PartialOrd)]
pub struct SquareAndBitBoard {
    square: Square,
    bitboard: BitBoard,
    promotion: bool,
}

impl SquareAndBitBoard {
    pub fn new(sq: Square, bb: BitBoard, promotion: bool) -> Self {
        Self {
            square: sq,
            bitboard: bb,
            promotion: promotion,
        }
    }
}

pub type MoveList = NoDrop<ArrayVec<SquareAndBitBoard, 18>>;

/// An incremental move generator
///
/// This structure enumerates moves slightly slower than board.enumerate_moves(...),
/// but has some extra features, such as:
///
/// * Being an iterator
/// * Not requiring you to create a buffer
/// * Only iterating moves that match a certain pattern
/// * Being iterable multiple times (such as, iterating once for all captures, then iterating again
///   for all quiets)
/// * Doing as little work early on as possible, so that if you are not going to look at every move, the
///   struture moves faster
/// * Being able to iterate pseudo legal moves, while keeping the (nearly) free legality checks in
///   place
///
/// # Examples
///
/// ```
/// use chess::MoveGen;
/// use chess::Board;
/// use chess::EMPTY;
/// use chess::construct;
///
/// // create a board with the initial position
/// let board = Board::default();
///
/// // create an iterable
/// let mut iterable = MoveGen::new_pseudolegal(&board);
///
/// // make sure .len() works.
/// assert_eq!(iterable.len(), 20); // the .len() function does *not* consume the iterator
///
/// // lets iterate over targets.
/// let targets = board.color_combined(!board.side_to_move());
/// iterable.set_iterator_mask(*targets);
///
/// // count the number of targets
/// let mut count = 0;
/// for _ in &mut iterable {
///     count += 1;
///     // This move captures one of my opponents pieces (with the exception of en passant)
/// }
///
/// // now, iterate over the rest of the moves
/// iterable.set_iterator_mask(!EMPTY);
/// for _ in &mut iterable {
///     count += 1;
///     // This move does not capture anything
/// }
///
/// // make sure it works
/// assert_eq!(count, 20);
///
/// ```
pub struct MoveGen {
    moves: MoveList,
    promotion_index: usize,
    iterator_mask: BitBoard,
    index: usize,
}
// TODO should I implement a builder pattern here or bake an immutable version of MoveGen that makes it faster to extract random samples?
// e.g. store the length and implement a get and/or get_random function?
impl MoveGen {
    fn enumerate_pseudolegal_moves(board: &Board) -> MoveList {
        let mask = !board.color_combined(board.side_to_move());
        let mut movelist = NoDrop::new(ArrayVec::<SquareAndBitBoard, 18>::new());

        PawnType::legals::<NotInCheckType>(&mut movelist, &board, mask);
        KnightType::legals::<NotInCheckType>(&mut movelist, &board, mask);
        BishopType::legals::<NotInCheckType>(&mut movelist, &board, mask);
        RookType::legals::<NotInCheckType>(&mut movelist, &board, mask);
        QueenType::legals::<NotInCheckType>(&mut movelist, &board, mask);
        KingType::legals::<NotInCheckType>(&mut movelist, &board, mask);

        movelist
    }

    /// Create a new `MoveGen` structure, only generating pseudolegal moves
    #[inline(always)]
    pub fn new_pseudolegal(board: &Board) -> MoveGen {
        MoveGen {
            moves: MoveGen::enumerate_pseudolegal_moves(board),
            promotion_index: 0,
            iterator_mask: !EMPTY,
            index: 0,
        }
    }

    #[inline(always)]
    fn enumerate_blind_moves(board: &Board) -> MoveList {
        let mut movelist = NoDrop::new(ArrayVec::<SquareAndBitBoard, 18>::new());

        PawnType::blind_moves(&mut movelist, &board);
        KnightType::blind_moves(&mut movelist, &board);
        BishopType::blind_moves(&mut movelist, &board);
        RookType::blind_moves(&mut movelist, &board);
        QueenType::blind_moves(&mut movelist, &board);
        KingType::blind_moves(&mut movelist, &board);

        movelist
    }

    /// Create a new `MoveGen` structure, only generating legal moves
    #[inline(always)]
    pub fn new_blind_moves(board: &Board) -> MoveGen {
        MoveGen {
            moves: MoveGen::enumerate_blind_moves(board),
            promotion_index: 0,
            iterator_mask: !EMPTY,
            index: 0,
        }
    }

    /// Never, ever, iterate any moves that land on the following squares
    pub fn remove_mask(&mut self, mask: BitBoard) {
        for x in 0..self.moves.len() {
            self.moves[x].bitboard &= !mask;
        }
    }

    /// Never, ever, iterate this move
    pub fn remove_move(&mut self, chess_move: ChessMove) -> bool {
        for x in 0..self.moves.len() {
            if self.moves[x].square == chess_move.get_source() {
                self.moves[x].bitboard &= !BitBoard::from_square(chess_move.get_dest());
                return true;
            }
        }
        false
    }

    /// For now, Only iterate moves that land on the following squares
    /// Note: Once iteration is completed, you can pass in a mask of ! `EMPTY`
    ///       to get the remaining moves, or another mask
    pub fn set_iterator_mask(&mut self, mask: BitBoard) {
        self.iterator_mask = mask;
        self.index = 0;

        // the iterator portion of this struct relies on the invariant that
        // the bitboards at the beginning of the moves[] array are the only
        // ones used.  As a result, we must partition the list such that the
        // assumption is true.

        // first, find the first non-used moves index, and store that in i
        let mut i = 0;
        while i < self.moves.len() && self.moves[i].bitboard & self.iterator_mask != EMPTY {
            i += 1;
        }

        // next, find each element past i where the moves are used, and store
        // that in i.  Then, increment i to point to a new unused slot.
        for j in (i + 1)..self.moves.len() {
            if self.moves[j].bitboard & self.iterator_mask != EMPTY {
                let backup = self.moves[i];
                self.moves[i] = self.moves[j];
                self.moves[j] = backup;
                i += 1;
            }
        }
    }

    /// This function checks the legality *only for moves generated by `MoveGen`*.
    ///
    /// Calling this function for moves not generated by `MoveGen` will result in possibly
    /// incorrect results, and making that move on the `Board` will result in undefined behavior.
    /// This function may panic! if these rules are not followed.
    ///
    /// If you are validating a move from a user, you should call the .legal() function.
    pub fn legal_quick(board: &Board, chess_move: ChessMove) -> bool {
        let piece = board.piece_on(chess_move.get_source()).unwrap();
        match piece {
            Piece::Rook => true,
            Piece::Bishop => true,
            Piece::Knight => true,
            Piece::Queen => true,
            Piece::Pawn => {
                if chess_move.get_source().get_file() != chess_move.get_dest().get_file()
                    && board.piece_on(chess_move.get_dest()).is_none()
                {
                    // en-passant
                    PawnType::legal_ep_move(board, chess_move.get_source(), chess_move.get_dest())
                } else {
                    true
                }
            }
            Piece::King => {
                let bb = between(chess_move.get_source(), chess_move.get_dest());
                if bb.popcnt() == 1 {
                    // castles
                    if !KingType::legal_king_move(board, bb.to_square()) {
                        false
                    } else {
                        KingType::legal_king_move(board, chess_move.get_dest())
                    }
                } else {
                    KingType::legal_king_move(board, chess_move.get_dest())
                }
            }
        }
    }

    /// Fastest perft test with this structure
    pub fn movegen_perft_test(board: &Board, depth: usize) -> usize {
        let iterable = MoveGen::new_pseudolegal(board);

        let mut result: usize = 0;
        if depth == 1 {
            iterable.len()
        } else {
            for m in iterable {
                let bresult = board.make_move_new(m);
                result += MoveGen::movegen_perft_test(&bresult, depth - 1);
            }
            result
        }
    }
}

impl ExactSizeIterator for MoveGen {
    /// Give the exact length of this iterator
    fn len(&self) -> usize {
        self.moves.iter().map(|m| m.bitboard.popcnt() as usize * (if m.promotion { NUM_PROMOTION_PIECES } else { 1 })).sum()
        // let mut result = 0;
        // for i in 0..self.moves.len() {
        //     if self.moves[i].bitboard & self.iterator_mask == EMPTY {
        //         break;
        //     }
        //     if self.moves[i].promotion {
        //         result += ((self.moves[i].bitboard & self.iterator_mask).popcnt() as usize)
        //             * NUM_PROMOTION_PIECES;
        //     } else {
        //         result += (self.moves[i].bitboard & self.iterator_mask).popcnt() as usize;
        //     }
        // }
        // result
    }
}

impl Iterator for MoveGen {
    type Item = ChessMove;

    /// Give a size_hint to some functions that need it
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }

    /// Find the next chess move.
    fn next(&mut self) -> Option<ChessMove> {
        if self.index >= self.moves.len()
            || self.moves[self.index].bitboard & self.iterator_mask == EMPTY
        {
            // are we done?
            None
        } else if self.moves[self.index].promotion {
            let moves = &mut self.moves[self.index];

            let dest = (moves.bitboard & self.iterator_mask).to_square();

            // deal with potential promotions for this pawn
            let result = ChessMove::new(
                moves.square,
                dest,
                Some(PROMOTION_PIECES[self.promotion_index]),
            );
            self.promotion_index += 1;
            if self.promotion_index >= NUM_PROMOTION_PIECES {
                moves.bitboard ^= BitBoard::from_square(dest);
                self.promotion_index = 0;
                if moves.bitboard & self.iterator_mask == EMPTY {
                    self.index += 1;
                }
            }
            Some(result)
        } else {
            // not a promotion move, so its a 'normal' move as far as this function is concerned
            let moves = &mut self.moves[self.index];
            let dest = (moves.bitboard & self.iterator_mask).to_square();

            moves.bitboard ^= BitBoard::from_square(dest);
            if moves.bitboard & self.iterator_mask == EMPTY {
                self.index += 1;
            }
            Some(ChessMove::new(moves.square, dest, None))
        }
    }
}
