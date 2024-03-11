use std::fs::File;
use std::io::Write;

use crate::bitboard::{BitBoard, EMPTY};
use crate::color::ALL_COLORS;
use crate::file::File as ChessFile;
use crate::square::{Square, ALL_SQUARES};

// Given a square, what are the valid king moves?
static mut SENSE_MASKS: [BitBoard; 64] = [EMPTY; 64];

// Generate the KING_MOVES array.
pub fn gen_sense_masks() {
    for src in ALL_SQUARES.iter() {
        unsafe {
            SENSE_MASKS[src.to_index()] = ALL_SQUARES
                .iter()
                .filter(|dest| {
                    let src_rank = src.get_rank().to_index() as i8;
                    let src_file = src.get_file().to_index() as i8;
                    let dest_rank = dest.get_rank().to_index() as i8;
                    let dest_file = dest.get_file().to_index() as i8;

                    ((src_rank - dest_rank).abs() == 1 || (src_rank - dest_rank).abs() == 0)
                        && ((src_file - dest_file).abs() == 1 || (src_file - dest_file).abs() == 0)
                })
                .fold(EMPTY, |b, s| b | BitBoard::from_square(*s));
        }
    }
}

// Write the SENSE_MASKS array to the specified file.
pub fn write_sense_masks(f: &mut File) {
    write!(f, "const SENSE_MASKS: [BitBoard; 64] = [\n").unwrap();
    for i in 0..64 {
        unsafe { write!(f, "    BitBoard({}),\n", SENSE_MASKS[i].0).unwrap() };
    }
    write!(f, "];\n").unwrap();
}
