"""
Generate a JSON corpus of (FEN, requested_move) -> (taken_move, capture_square)
records using the reconchess Python reference implementation as ground truth.

Usage:
    pip install reconchess
    python3 generate_corpus.py > rbc_corpus.json

The Rust test in tests/rbc_corpus.rs will read this file and assert that
chess::simulate_move produces the same results.

This script does NOT need to run as part of the normal build. The committed
rbc_corpus.json captures a fixed reference set; regenerate only when adding
new test cases or upgrading reconchess.
"""

import json
import sys

import chess
from reconchess.utilities import (
    revise_move,
    capture_square_of_move,
    add_pawn_queen_promotion,
)


def case(fen: str, uci_move: str | None, comment: str = "") -> dict:
    """Build a corpus entry.

    The request must be either None (pass) or a move that is in
    reconchess.move_actions(board). For requests that aren't in
    move_actions, reconchess raises ValueError; for symmetry, our Rust
    simulate_move returns Err(InvalidGeometry). Such cases should use
    `error_case` instead.
    """
    from reconchess.utilities import move_actions

    board = chess.Board(fen)
    if uci_move is None:
        return {
            "fen": fen,
            "requested": None,
            "taken": None,
            "capture_square": None,
            "expect_error": False,
            "comment": comment or "pass",
        }
    requested = chess.Move.from_uci(uci_move)
    requested = add_pawn_queen_promotion(board, requested)
    if requested not in move_actions(board):
        raise ValueError(
            f"case `{comment}`: {uci_move} is not in move_actions for {fen}; "
            f"use error_case() instead."
        )
    taken = revise_move(board, requested)
    capture_sq = capture_square_of_move(board, taken)
    return {
        "fen": fen,
        "requested": requested.uci(),
        "taken": taken.uci() if taken is not None else None,
        "capture_square": chess.square_name(capture_sq) if capture_sq is not None else None,
        "expect_error": False,
        "comment": comment,
    }


def error_case(fen: str, uci_move: str, comment: str = "") -> dict:
    """Build an entry for a request that is not in move_actions, i.e. one
    where the Rust simulate_move should return Err."""
    return {
        "fen": fen,
        "requested": uci_move,
        "taken": None,
        "capture_square": None,
        "expect_error": True,
        "comment": comment,
    }


# Standard test positions.
START = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"

CASES = []

# ---- Pass ----
CASES.append(case(START, None, "pass at start"))

# ---- Pawn forward pushes ----
CASES.append(case(START, "e2e4", "double push, clear path"))
CASES.append(case(START, "e2e3", "single push, clear path"))
CASES.append(
    case("4k3/8/8/8/8/4n3/4P3/4K3 w - - 0 1", "e2e4", "double push blocked at mid")
)
CASES.append(
    case("4k3/8/8/8/4n3/8/4P3/4K3 w - - 0 1", "e2e4", "double push blocked at far -> revise to single")
)
CASES.append(
    case("4k3/8/8/8/8/4n3/4P3/4K3 w - - 0 1", "e2e3", "single push blocked")
)
CASES.append(
    case("4k3/8/8/8/4n3/8/4P3/4K3 w - - 0 1", "e2e3", "single push clear (mid is far for single)")
)

# ---- Pawn diagonal captures ----
CASES.append(
    case("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", "e4d5", "pawn captures pawn diagonally")
)
CASES.append(
    case("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1", "e2f3", "pawn diagonal to empty square: illegal request -> pass")
)

# ---- En passant ----
CASES.append(
    case(
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3",
        "e5f6",
        "en passant capture"
    )
)
CASES.append(
    case(
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3",
        "e5f6",
        "diagonal to empty square with no en passant: pass"
    )
)

# ---- Pawn promotion ----
CASES.append(
    case("7k/P7/8/8/8/8/8/4K3 w - - 0 1", "a7a8", "pawn push to back rank with no promo -> queen")
)
CASES.append(
    case("7k/P7/8/8/8/8/8/4K3 w - - 0 1", "a7a8q", "explicit queen promotion")
)
CASES.append(
    case("7k/P7/8/8/8/8/8/4K3 w - - 0 1", "a7a8n", "explicit knight promotion")
)
CASES.append(
    case("7k/P7/8/8/8/8/8/4K3 w - - 0 1", "a7a8r", "explicit rook promotion")
)
CASES.append(
    case("7k/P7/8/8/8/8/8/4K3 w - - 0 1", "a7a8b", "explicit bishop promotion")
)

# ---- Knight ----
CASES.append(case(START, "b1c3", "knight standard move"))
CASES.append(case(START, "g1f3", "knight standard move"))
CASES.append(
    case("rnbqkbnr/ppp1pppp/8/3p4/8/2N5/PPPPPPPP/R1BQKBNR w KQkq - 0 1", "c3d5", "knight capture")
)

# ---- Bishop ----
CASES.append(case("4k3/8/8/8/8/8/8/2B1K3 w - - 0 1", "c1h6", "bishop slide unobstructed"))
CASES.append(
    case("4k3/8/8/8/8/4n3/8/2B1K3 w - - 0 1", "c1h6", "bishop slide stopped early")
)


# ---- Rook ----
CASES.append(case("4k3/8/8/8/8/8/8/R3K3 w - - 0 1", "a1a8", "rook slide unobstructed"))
CASES.append(
    case("4k3/8/8/8/n7/8/8/R3K3 w - - 0 1", "a1a8", "rook slide stopped at a4")
)
CASES.append(
    case("r3k3/8/8/8/8/8/8/4K3 b - - 0 1", "a8a1", "rook slide descending, unobstructed (black)")
)
CASES.append(
    case("r3k3/8/8/8/N7/8/8/4K3 b - - 0 1", "a8a1", "rook slide descending stopped at a4")
)

# ---- Queen ----
CASES.append(case("4k3/8/8/8/8/8/8/3QK3 w - - 0 1", "d1d8", "queen vertical unobstructed"))
CASES.append(
    case("4k3/8/8/8/3n4/8/8/3QK3 w - - 0 1", "d1d8", "queen vertical stopped early")
)
CASES.append(
    case("4k3/8/8/8/4n3/8/8/3QK3 w - - 0 1", "d1h5", "queen diagonal stopped early")
)

# ---- King single moves ----
CASES.append(case("4k3/8/8/8/8/8/8/4K3 w - - 0 1", "e1e2", "king one step"))
CASES.append(
    case("4k3/8/8/8/8/8/8/3qK3 w - - 0 1", "e1d1", "king captures opponent piece adjacent")
)

# ---- Castling ----
CASES.append(case("4k3/8/8/8/8/8/8/4K2R w K - 0 1", "e1g1", "kingside castle clear"))
CASES.append(case("4k3/8/8/8/8/8/8/R3K3 w Q - 0 1", "e1c1", "queenside castle clear"))
CASES.append(
    error_case("4k3/8/8/8/8/8/8/4KB1R w K - 0 1", "e1g1", "kingside castle blocked by own bishop -> not in move_actions")
)
CASES.append(
    error_case("4k3/8/8/8/8/8/8/RN2K3 w Q - 0 1", "e1c1", "queenside castle blocked at b1 -> not in move_actions")
)
CASES.append(
    error_case("4k3/8/8/8/8/8/8/R1N1K3 w Q - 0 1", "e1c1", "queenside castle blocked at c1 -> not in move_actions")
)
CASES.append(
    error_case("4k3/8/8/8/8/8/8/R2NK3 w Q - 0 1", "e1c1", "queenside castle blocked at d1 -> not in move_actions")
)
CASES.append(
    case("4k1r1/8/8/8/8/8/8/4K2R w K - 0 1", "e1g1", "castle through check (RBC allows)")
)
CASES.append(
    error_case("4k3/8/8/8/8/8/8/4K2R w - - 0 1", "e1g1", "castle without rights -> not in move_actions")
)
CASES.append(
    case("r3k3/8/8/8/8/8/8/4K3 b q - 0 1", "e8c8", "black queenside castle")
)
CASES.append(
    error_case("rn2k3/8/8/8/8/8/8/4K3 b q - 0 1", "e8c8", "black queenside castle blocked at b8 -> not in move_actions")
)


def main():
    out = []
    for c in CASES:
        out.append(c)
    json.dump(out, sys.stdout, indent=2)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
