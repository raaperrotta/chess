//! Reconnaissance Blind Chess (RBC) rules engine and player framework.
//!
//! RBC is a chess variant in which players cannot see their opponent's
//! pieces. Each turn, a player chooses a 3×3 region of the board to
//! "sense", then proposes a chess move (or passes). The move may be
//! modified by the rules engine if it runs into an unseen opponent piece.
//! The game ends when a king is captured.
//!
//! This module's design and rules implementation track the canonical
//! [reconchess](https://github.com/reconnaissanceblindchess/reconchess)
//! Python reference. The integration test
//! `tests/rbc_corpus.rs` cross-validates [`simulate_move`] against
//! reference outputs computed by reconchess on a corpus of positions; see
//! `tests/data/generate_corpus.py` for the generator.
//!
//! # Key types
//! - [`Player`] trait: the contract every RBC player implements.
//! - [`MoveResult`], [`SenseResult`]: outputs of the rules engine.
//! - [`GameOverReason`]: why a game ended.
//!
//! # Key functions
//! - [`simulate_move`] / [`simulate_move_unchecked`]: run a single
//!   requested move through the RBC rules to produce a `MoveResult`.
//! - [`simulate_sense`]: produce the `SenseResult` for sensing at a square.
//! - [`capture_square`]: convenience wrapper returning only the capture
//!   square from `simulate_move`.
//! - [`add_pawn_queen_promotion`]: mirror of reconchess's auto-queen rule.
//! - [`do_sense`], [`do_move`], [`do_half_turn`]: building blocks for
//!   driving a single turn.
//! - [`play_rbc`]: drive a complete game between two players.
//!
//! # Example
//!
//! ```no_run
//! use chess::{play_rbc, Color, GameOverReason, MhtPlayer, RandomPlayer};
//!
//! let mut white = RandomPlayer::new();
//! let mut black = MhtPlayer::new();
//! match play_rbc(&mut white, &mut black) {
//!     GameOverReason::KingCapture(c) => println!("{:?} wins", c),
//!     GameOverReason::IllegalMove(c) => println!("{:?} disqualified", c),
//!     GameOverReason::FiftyMoveDraw => println!("draw"),
//! }
//! # let _ = Color::White;
//! ```
//!
//! # Player implementations
//! - [`RandomPlayer`]: picks a random blind move (or pass) each turn.
//!   Useful as a baseline.
//! - [`PassivePlayer`]: always passes. Useful as a sanity check.
//! - [`AttackerPlayer`]: plays one of several scripted opening attack
//!   sequences.
//! - [`MhtPlayer`]: multi-hypothesis tracker that maintains a belief set
//!   of possible true boards and picks senses that maximize information
//!   gain. Currently picks moves greedily from a sampled belief.

mod rbc;
pub use self::rbc::*;

mod players;
pub use self::players::*;
