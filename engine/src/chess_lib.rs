// useful resources:
// https://www.chess.com/terms
// https://www.chessprogramming.org/Main_Page


pub mod uci_parser;
pub use uci_parser::*;

pub mod uci_executor;
pub use uci_executor::*;

pub mod bitboard;
pub use bitboard::*;

pub mod board;
pub use board::*;

pub mod moves;
pub use moves::*;