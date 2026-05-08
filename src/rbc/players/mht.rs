use rayon::prelude::*;
use rustc_hash::FxHashMap;

use crate::{
    simulate_move_unchecked, simulate_sense, Board, ChessMove, MoveGen, MoveResult, Player,
    SenseResult, Square, SENSE_SQUARES,
};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;

/// Maximum belief-set size before particle-filter sampling kicks in.
/// Above this, we randomly drop boards down to this cap to keep the inner
/// loops bounded. Set generously; tune later.
const MAX_BELIEF_SIZE: usize = 5_000;

pub struct MhtPlayer {
    rng: ThreadRng,
    boards: Vec<Board>,
    sense_partition: FxHashMap<SenseResult, Vec<usize>>,
    requested_move: Option<ChessMove>,
    /// If true, log belief-set transitions to stderr. Off by default so
    /// running tournaments doesn't drown stdout.
    pub verbose: bool,
}

impl MhtPlayer {
    /// Construct a fresh `MhtPlayer` that believes the game just started
    /// from the standard initial position (a single hypothesis equal to
    /// `Board::default()`).
    pub fn new() -> Self {
        Self::with_belief(vec![Board::default()])
    }

    /// Construct a `MhtPlayer` whose initial belief set is `boards`.
    /// Use this to start a player at an arbitrary mid-game position, or
    /// to seed the belief set with multiple plausible positions
    /// (e.g. after parsing a saved game where some opponent moves were
    /// not observed).
    ///
    /// `boards` should be non-empty; if empty, the player will behave as
    /// though every move is impossible.
    pub fn with_belief(boards: Vec<Board>) -> Self {
        Self {
            rng: rand::rng(),
            boards,
            sense_partition: FxHashMap::default(),
            requested_move: None,
            verbose: false,
        }
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Number of boards in the current belief set. Useful for tests/metrics.
    pub fn belief_size(&self) -> usize {
        self.boards.len()
    }

    fn cap_belief_set(&mut self) {
        if self.boards.len() > MAX_BELIEF_SIZE {
            // Random sample down to the cap. This is the simplest particle
            // filter; better strategies (e.g. weight by recency, by piece
            // count) can replace it later.
            let sampled: Vec<Board> = self
                .boards
                .iter()
                .cloned()
                .choose_multiple(&mut self.rng, MAX_BELIEF_SIZE);
            if self.verbose {
                eprintln!(
                    "MhtPlayer: capped belief set {} -> {}",
                    self.boards.len(),
                    sampled.len()
                );
            }
            self.boards = sampled;
        }
    }
}

impl Player for MhtPlayer {
    fn handle_opponent_capture(&mut self, capture: &Option<Square>) {
        if self.verbose {
            eprintln!(
                "MhtPlayer: expanding {} boards after capture {:?}",
                self.boards.len(),
                capture.map(|s| s.to_string())
            );
        }
        let mut next: FxHashMap<u64, Board> = FxHashMap::default();
        for board in self.boards.iter() {
            // Enumerate every possible "taken move" the opponent could have
            // made on this hypothesis board. The opponent's request set is
            // the blind-move list; the resulting taken move is whatever
            // simulate_move returns. We also consider the pass option
            // (requested = None or any illegal request -> taken = None).
            for requested in MoveGen::new_blind_moves(board).map(Some).chain(std::iter::once(None)) {
                // Safety: requested is None or comes from new_blind_moves(board).
                let result = simulate_move_unchecked(board, requested);
                if result.capture_square != *capture {
                    continue;
                }
                let new_board = match result.taken_move {
                    Some(m) => board.make_move_new(m),
                    None => {
                        // Opponent passed (or made an illegal request).
                        // We don't observe this directly; if the observed
                        // capture is None, it's a candidate.
                        let mut nb = *board;
                        nb.null_move_mut();
                        nb
                    }
                };
                next.insert(new_board.get_hash(), new_board);
            }
        }
        if self.verbose {
            eprintln!(
                "MhtPlayer: expanded into {} boards (deduped by hash)",
                next.len()
            );
        }
        self.boards = next.into_values().collect();
        self.cap_belief_set();
    }

    fn choose_sense(&mut self) -> Square {
        // Pick the sense square that minimizes the size of the largest
        // posterior partition (worst-case information gain).
        // Tie-break by smaller sum-of-squares of partition sizes (a
        // proxy for expected/Shannon partition size).
        //
        // Each candidate square's partition is independent of the others,
        // so we evaluate them in parallel and reduce to find the best.
        let boards = &self.boards;
        let candidates: Vec<(Square, FxHashMap<SenseResult, Vec<usize>>, usize, usize)> =
            SENSE_SQUARES
                .par_iter()
                .map(|&square| {
                    let mut partition: FxHashMap<SenseResult, Vec<usize>> =
                        FxHashMap::default();
                    for (i, board) in boards.iter().enumerate() {
                        let r = simulate_sense(board, square);
                        partition.entry(r).or_default().push(i);
                    }
                    let max = partition.values().map(|v| v.len()).max().unwrap_or(0);
                    let sumsq: usize = partition.values().map(|v| v.len() * v.len()).sum();
                    (square, partition, max, sumsq)
                })
                .collect();
        // Reduce: pick (min max, then min sumsq, then first by SENSE_SQUARES order).
        let mut best_square = Square::A1;
        let mut best_max = usize::MAX;
        let mut best_sumsq = usize::MAX;
        let mut best_partition: FxHashMap<SenseResult, Vec<usize>> = FxHashMap::default();
        for (square, partition, max, sumsq) in candidates {
            let better = max < best_max || (max == best_max && sumsq < best_sumsq);
            if better {
                best_square = square;
                best_max = max;
                best_sumsq = sumsq;
                best_partition = partition;
            }
        }
        self.sense_partition = best_partition;
        best_square
    }

    fn handle_sense_result(&mut self, sense_result: &SenseResult) {
        let indices = match self.sense_partition.get(sense_result) {
            Some(v) => v.clone(),
            None => {
                // Observation inconsistent with all hypotheses: belief set
                // is wrong somewhere upstream. Keep all boards rather than
                // crashing; flag in verbose mode.
                if self.verbose {
                    eprintln!(
                        "MhtPlayer: sense observation matched no hypothesis (belief={})",
                        self.boards.len()
                    );
                }
                return;
            }
        };
        self.boards = indices.iter().map(|&i| self.boards[i]).collect();
        if self.verbose {
            eprintln!("MhtPlayer: filtered to {} boards by sense", self.boards.len());
        }
    }

    fn choose_move(&mut self) -> Option<ChessMove> {
        // Baseline: pick a random board from the belief set, then a random
        // blind move (or pass) on it. Improvement opportunities documented
        // in the audit but out of scope for this pass.
        let board = match self.boards.iter().choose(&mut self.rng) {
            Some(b) => b,
            None => {
                // Empty belief set: shouldn't happen, but pass safely.
                self.requested_move = None;
                return None;
            }
        };
        let mut moves: Vec<Option<ChessMove>> = MoveGen::new_blind_moves(board).map(Some).collect();
        moves.push(None);
        self.requested_move = *moves.iter().choose(&mut self.rng).unwrap();
        self.requested_move
    }

    fn handle_move_result(&mut self, result: &MoveResult) {
        // Filter and advance every hypothesis where simulate_move(board, my_request)
        // yields exactly the observed result.
        let mut next = Vec::with_capacity(self.boards.len());
        for board in self.boards.iter() {
            // Safety: self.requested_move was either None or chosen from
            // new_blind_moves on a board with the same own-piece config
            // as `board` (because the MHT belief set is consistent with
            // own observations of own pieces).
            let sim = simulate_move_unchecked(board, self.requested_move);
            if &sim != result {
                continue;
            }
            let mut nb = *board;
            match result.taken_move {
                Some(m) => nb.make_move_mut(m),
                None => nb.null_move_mut(),
            }
            next.push(nb);
        }
        if self.verbose {
            eprintln!(
                "MhtPlayer: filtered {} -> {} by own move result",
                self.boards.len(),
                next.len()
            );
        }
        self.boards = next;
        self.cap_belief_set();
    }
}
