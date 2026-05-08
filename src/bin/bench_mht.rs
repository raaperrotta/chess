//! Microbenchmark: drive an MhtPlayer through a small number of
//! belief-expansion + sense + filter cycles to measure inner-loop perf.
//!
//! We construct the worst case: both players always pass. Belief sets grow
//! roughly geometrically (capped by `MAX_BELIEF_SIZE`) until the
//! particle-filter cap activates. This stresses
//! `simulate_move`/`simulate_sense` in the inner loops.
//!
//! Total work scales with (belief_size × blind_moves) per turn, so the
//! reported per-turn time is the most useful number.

use std::time::Instant;

use chess::{do_half_turn, ChessMove, MhtPlayer, MoveResult, Player, SenseResult, Square};

/// A player that always senses at e4 and always passes. Worst case for
/// belief growth: every legal opponent move (including pass) survives.
struct PassPlayer;
impl Player for PassPlayer {
    fn handle_opponent_capture(&mut self, _: &Option<Square>) {}
    fn choose_sense(&mut self) -> Square {
        Square::E4
    }
    fn handle_sense_result(&mut self, _: &SenseResult) {}
    fn choose_move(&mut self) -> Option<ChessMove> {
        None
    }
    fn handle_move_result(&mut self, _: &MoveResult) {}
}

fn main() {
    const NUM_TURNS: usize = 9;

    let start = Instant::now();
    let mut white = MhtPlayer::new();
    let mut black = PassPlayer;

    let mut board = chess::Board::default();
    chess::do_move(&mut board, &mut white, &mut black).unwrap();
    println!(
        "after white t1: belief = {} (took {:.3}s)",
        white.belief_size(),
        start.elapsed().as_secs_f64()
    );

    for t in 0..NUM_TURNS {
        let t_iter = Instant::now();
        do_half_turn(&mut board, &mut black, &mut white).unwrap();
        do_half_turn(&mut board, &mut white, &mut black).unwrap();
        println!(
            "turn {}: belief = {} ({:.3}s, total {:.3}s)",
            t + 2,
            white.belief_size(),
            t_iter.elapsed().as_secs_f64(),
            start.elapsed().as_secs_f64()
        );
    }
    println!("total: {:.3}s", start.elapsed().as_secs_f64());
}
