use std::collections::HashMap;

use crate::{capture_square, simulate_move, simulate_sense, Board, ChessMove, MoveGen, MoveResult, Player, SenseResult, Square, SENSE_SQUARES};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;

pub struct MhtPlayer {
    rng: ThreadRng,
    boards: Vec<Board>,
    requested_sense: Square,
    sense_partition: HashMap<SenseResult, Vec<usize>>,
    requested_move: Option<ChessMove>,
}
impl MhtPlayer {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            boards: vec!(Board::default()),
            requested_sense: Square::B2,
            sense_partition: HashMap::new(),
            requested_move: None,
        }
    }
}
impl Player for MhtPlayer {
    fn handle_opponent_capture(&mut self, capture: &Option<Square>) {
        println!("Expanding {} boards after capture square {:?}", self.boards.len(), capture.map(|m| m.to_string()));
        if self.boards.len() > 10_000 { panic!() }
        let mut boards = HashMap::new();
        for board in self.boards.iter() {
            for m in MoveGen::new_pseudolegal(&board) {
                let simulated_capture = capture_square(board, Some(m));
                if simulated_capture == *capture {
                    let new_board = board.make_move_new(m);
                    boards.insert(new_board.get_hash(), new_board);
                }
            }
        }
        println!("Expanded {} boards into {}", self.boards.len(), boards.len());
        self.boards = boards.values().cloned().collect();
    }
    fn choose_sense(&mut self) -> Square {
        let mut min_max_num_boards = usize::MAX;
        let mut best_square = Square::A1;
        for square in SENSE_SQUARES {
            let mut partition = HashMap::new();
            for (i, board) in self.boards.iter().enumerate() {
                let simulated_sense = simulate_sense(board, square);
                let part = partition.entry(simulated_sense).or_insert_with(|| Vec::new());
                part.push(i)
            }
            let max_num_boards = partition.iter().map(|(_, boards)| boards.len()).max().unwrap();
            if max_num_boards < min_max_num_boards {
                min_max_num_boards = max_num_boards;
                best_square = square;
                self.sense_partition = partition;
            }
        }
        best_square
    }
    fn handle_sense_result(&mut self, sense_result: &SenseResult) {
        let indices = self.sense_partition.get(sense_result).unwrap();
        self.boards = indices.into_iter().map(|&i| self.boards[i]).collect();
        println!("Filtered down to {} boards", self.boards.len());
    }
    fn choose_move(&mut self) -> Option<ChessMove> {
        let board = self.boards.first().unwrap();
        let mut moves: Vec<_> = MoveGen::new_blind_moves(&board)
            .map(|m| Some(m))
            .collect();
        moves.push(None);
        self.requested_move = *moves.iter().choose(&mut self.rng).unwrap();
        self.requested_move
    }
    fn handle_move_result(&mut self, result: &MoveResult) {
        println!("Filtering {} boards", self.boards.len());
        let mut boards = Vec::new();
        for board in self.boards.iter_mut() {
            let simulated_result = simulate_move(board, self.requested_move);
            let matches = *result == simulated_result;
            if matches {
                match result.taken_move {
                    Some(m) => board.make_move_mut(m),
                    None => board.null_move_mut(),
                }
                boards.push(*board);
            }
        };
        // self.boards.retain(|board| {
        //     let simulated_result = simulate_move(board, self.requested_move);
        //     let matches = *result == simulated_result;
        //     if matches {
        //         match result.taken_move {
        //             Some(m) => board.make_move_mut(m),
        //             None => board.null_move_mut(),
        //         }
        //         boards.push(*board);
        //     }
        //     matches
        // });
        println!("Filtered {} boards down to {}", self.boards.len(), boards.len());
        self.boards = boards;
    }
}
