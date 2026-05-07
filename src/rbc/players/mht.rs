use std::collections::HashMap;

use crate::{
    simulate_move, simulate_sense, Board, ChessMove, MoveGen, MoveResult, Player, SenseResult,
    Square, SENSE_SQUARES,
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
    sense_partition: HashMap<SenseResult, Vec<usize>>,
    requested_move: Option<ChessMove>,
    /// If true, log belief-set transitions to stderr. Off by default so
    /// running tournaments doesn't drown stdout.
    pub verbose: bool,
}

impl MhtPlayer {
    pub fn new() -> Self {
        Self {
            rng: rand::rng(),
            boards: vec![Board::default()],
            sense_partition: HashMap::new(),
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
        let mut next: HashMap<u64, Board> = HashMap::new();
        for board in self.boards.iter() {
            // Enumerate every possible "taken move" the opponent could have
            // made on this hypothesis board. The opponent's request set is
            // the blind-move list; the resulting taken move is whatever
            // simulate_move returns. We also consider the pass option
            // (requested = None or any illegal request -> taken = None).
            for requested in MoveGen::new_blind_moves(board).map(Some).chain(std::iter::once(None)) {
                let result = simulate_move(board, requested);
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
        // Tie-break by smaller mean partition size (Shannon-like preference).
        let mut best_square = Square::A1;
        let mut best_max = usize::MAX;
        let mut best_mean_x_2: usize = usize::MAX;
        for square in SENSE_SQUARES {
            let mut partition: HashMap<SenseResult, Vec<usize>> = HashMap::new();
            for (i, board) in self.boards.iter().enumerate() {
                let r = simulate_sense(board, square);
                partition.entry(r).or_default().push(i);
            }
            let max = partition.values().map(|v| v.len()).max().unwrap_or(0);
            // Use sum-of-squares as a proxy for expected partition size.
            let sumsq: usize = partition.values().map(|v| v.len() * v.len()).sum();
            let better = max < best_max || (max == best_max && sumsq < best_mean_x_2);
            if better {
                best_square = square;
                best_max = max;
                best_mean_x_2 = sumsq;
                self.sense_partition = partition;
            }
        }
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
            let sim = simulate_move(board, self.requested_move);
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
