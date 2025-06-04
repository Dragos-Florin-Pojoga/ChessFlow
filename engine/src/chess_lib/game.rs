use crate::board::*;
use crate::terminal_states::*;

use std::collections::HashMap;



/// Represents the type of node in the transposition table.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NodeType {
    Exact,      // The score is exact (alpha <= score <= beta)
    Alpha,      // The score is a lower bound (score <= alpha)
    Beta,       // The score is an upper bound (score >= beta)
}

/// Entry stored in the transposition table.
#[derive(Debug, Clone, Copy)]
pub struct TTEntry {
    pub score: i32,
    pub depth: u8,
    pub node_type: NodeType,
    pub best_move: Option<ChessMove>, // Store the best move found for this position
    // hash_key: u64, // Can be added if using a fixed-size array for TT and need to check for collisions
}

/// The main chess engine struct containing search state and tables.
pub struct Game {
    pub board: Board,
    pub move_history: Vec<ChessMove>,
    pub pseudo_legal_moves_container: Vec<Vec<ChessMove>>,
    pub legal_moves_container: Vec<Vec<ChessMove>>,
    pub board_repetition_counts: HashMap<u64, u8>,
    pub transposition_table: HashMap<u64, TTEntry>,
    pub killer_moves: Vec<[Option<ChessMove>; 2]>, // Two killer moves per depth
    pub history_moves: [[i32; 64]; 64], // History table for non-capture moves (from_square_index, to_square_index) -> score
    pub max_search_depth: u8,
    pub q_search_max_ply: u8,
}

impl Game {
    pub fn new(board: Board, max_depth: u8, q_search_max_ply: u8) -> Self {
        let mut pseudo_legal_moves_container = Vec::with_capacity(max_depth as usize + 1);
        let mut legal_moves_container = Vec::with_capacity(max_depth as usize + 1);
        let mut killer_moves = Vec::with_capacity(max_depth as usize + 1);
        for _ in 0..=max_depth {
            pseudo_legal_moves_container.push(Vec::new());
            legal_moves_container.push(Vec::new());
            killer_moves.push([None; 2]); // Initialize killer moves
        }


        Game {
            board,
            move_history: Vec::with_capacity(100),
            pseudo_legal_moves_container,
            legal_moves_container,
            board_repetition_counts: HashMap::new(),
            max_search_depth: max_depth,
            transposition_table: HashMap::new(),
            killer_moves: killer_moves,
            history_moves: [[0; 64]; 64], // Initialize history table
            q_search_max_ply,
        }
    }

    pub fn make_move(&mut self, mv: &ChessMove) {
        self.board = self.board.make_move(&mv);
        self.move_history.push(*mv);
        *self.board_repetition_counts.entry(self.board.compute_zobrist_hash()).or_insert(0) += 1;
    }

    pub fn print(&mut self) {
        // let board_hash = self.board.compute_zobrist_hash();
        // println!("={} {} {}", *self.board_repetition_counts.entry(board_hash).or_insert(0), board_hash, self.board.to_fen());
        println!("{}", self.board);
    }

    pub fn get_game_state(&mut self) -> GameState {
        let mut pseudo_legal_moves = std::mem::take(&mut self.pseudo_legal_moves_container[0]);
        let mut legal_moves = std::mem::take(&mut self.legal_moves_container[0]);

        self.board.generate_legal_moves(&mut pseudo_legal_moves, &mut legal_moves);

        let board_hash = self.board.compute_zobrist_hash();
        let board_repetition_count = *self.board_repetition_counts.entry(board_hash).or_insert(0);

        let game_state = self.board.check_game_state(legal_moves.is_empty(), board_repetition_count);

        self.pseudo_legal_moves_container[0] = pseudo_legal_moves;
        self.legal_moves_container[0] = legal_moves;

        game_state
    }

    pub fn print_end(&mut self) {
        let game_state = self.get_game_state();
        print!("\n\n\n\x1b[1;35m{:?}\x1b[0m\n\n", game_state);
        self.print();

        print!("\n\n{}\n\n", self.to_pgn());
    }
}


use std::fmt::Write;

impl Game {
    pub fn to_pgn(&mut self) -> String {
        let mut pseudo_legal_moves = std::mem::take(&mut self.pseudo_legal_moves_container[0]);
        let mut legal_moves = std::mem::take(&mut self.legal_moves_container[0]);

        let mut pgn = String::new();

        // Optional PGN tags
        writeln!(pgn, "[Event \"?\"]").unwrap();
        writeln!(pgn, "[Site \"?\"]").unwrap();
        writeln!(pgn, "[Date \"????.??.??\"]").unwrap();
        writeln!(pgn, "[Round \"?\"]").unwrap();
        writeln!(pgn, "[White \"?\"]").unwrap();
        writeln!(pgn, "[Black \"?\"]").unwrap();
        writeln!(pgn, "[Result \"{}\"]", self.get_result_string()).unwrap();

        writeln!(pgn).unwrap();

        let mut board_repetition_counts = HashMap::new();

        let mut board = Board::new_start_pos();
        for (i, mv) in self.move_history.iter().enumerate() {
            *board_repetition_counts.entry(board.compute_zobrist_hash()).or_insert(0) += 1;
            if i % 2 == 0 {
                write!(pgn, "{}. ", board.fullmove_number).unwrap();
            }

            let san = board.to_san(mv, &mut pseudo_legal_moves, &mut legal_moves, &mut board_repetition_counts);
            write!(pgn, "{} ", san).unwrap();

            board = board.make_move(mv);
        }

        write!(pgn, "{}", self.get_result_string()).unwrap();

        self.pseudo_legal_moves_container[0] = pseudo_legal_moves;
        self.legal_moves_container[0] = legal_moves;

        pgn
    }

    fn get_result_string(&mut self) -> &'static str {
        let game_state = self.get_game_state();

        match game_state {
            GameState::Checkmate(checkmated) => {
                match checkmated {
                    Color::White => "0-1", // Black delivered checkmate
                    Color::Black => "1-0", // White delivered checkmate
                }
            }
            GameState::Stalemate
            | GameState::FiftyMoveDraw
            | GameState::InsufficientMaterialDraw
            | GameState::ThreefoldRepetitionDraw => "1/2-1/2",
            GameState::Ongoing => "*"
        }
    }
}
