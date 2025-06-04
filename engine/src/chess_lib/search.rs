use crate::board::*;
use crate::terminal_states::*;
use std::cmp::max;
use std::cmp::min;
use std::collections::HashMap;

pub struct Game {
    pub board: Board,
    pub pseudo_legal_moves_container: Vec<Vec<ChessMove>>,
    pub legal_moves_container: Vec<Vec<ChessMove>>,
    pub max_search_depth: usize,
    pub board_repetition_counts: HashMap<u64, u8>,
}

impl Game {
    pub fn new(max_search_depth: usize) -> Self {
        let mut game = Self {
            board: Board::new_start_pos(),
            pseudo_legal_moves_container: Vec::with_capacity(max_search_depth + 1),
            legal_moves_container: Vec::with_capacity(max_search_depth + 1),
            max_search_depth: max_search_depth,
            board_repetition_counts: HashMap::new(),
        };

        for _ in 0..=game.max_search_depth {
            game.pseudo_legal_moves_container.push(Vec::with_capacity(200));
            game.legal_moves_container.push(Vec::with_capacity(100));
        }

        game.board_repetition_counts.entry(game.board.compute_zobrist_hash()).or_insert(0);

        game
    }

    pub fn new_from_fen(fen: &Fen, max_search_depth: usize) -> Result<Self, FenParseError> {
        let board = Board::from_fen(fen)?;
        let mut game = Game {
            board,
            pseudo_legal_moves_container: Vec::with_capacity(max_search_depth + 1),
            legal_moves_container: Vec::with_capacity(max_search_depth + 1),
            max_search_depth: max_search_depth,
            board_repetition_counts: HashMap::new(),
        };
        for _ in 0..=game.max_search_depth {
            game.pseudo_legal_moves_container.push(Vec::with_capacity(200));
            game.legal_moves_container.push(Vec::with_capacity(100));
        }
        game.board_repetition_counts.entry(game.board.compute_zobrist_hash()).or_insert(0);
        Ok(game)
    }




    // RECURSIVE MUTABLE METHODS ARE THE WORST
    #[cfg_attr(feature = "tracy", tracing::instrument(skip_all))]
    pub fn alphabeta(&mut self, board: Board, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
        let mut pseudo_legal_moves = std::mem::take(&mut self.pseudo_legal_moves_container[depth as usize]);
        let mut legal_moves = std::mem::take(&mut self.legal_moves_container[depth as usize]);

        board.generate_legal_moves(&mut pseudo_legal_moves, &mut legal_moves);

        let board_hash = board.compute_zobrist_hash();
        let board_repetition_count = {
            let count_ref = self.board_repetition_counts.entry(board_hash).or_insert(0);
            *count_ref += 1;
            *count_ref
        };

        let game_state = board.check_game_state(legal_moves.is_empty(), board_repetition_count);
        if depth == 0 || game_state != GameState::Ongoing || legal_moves.is_empty() {
            let eval = board.evaluate(depth, &mut pseudo_legal_moves, &mut legal_moves, game_state);
            *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
            self.pseudo_legal_moves_container[depth as usize] = pseudo_legal_moves;
            self.legal_moves_container[depth as usize] = legal_moves;
            return eval;
        }

        let result_value;

        if board.turn == Color::White {
            let mut value = i32::MIN;
            for mv in &legal_moves {
                let new_board = board.make_move(&mv);
                let score = self.alphabeta(new_board, depth - 1, alpha, beta);
                value = max(value, score);
                alpha = max(alpha, value);
                if alpha >= beta {
                    break;
                }
            }
            result_value = value;
        }
        else {
            let mut value = i32::MAX;
            for mv in &legal_moves {
                let new_board = board.make_move(&mv);
                let score = self.alphabeta(new_board, depth - 1, alpha, beta);
                value = min(value, score);
                beta = min(beta, value);
                if beta <= alpha {
                    break;
                }
            }
            result_value = value;
        }

        *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
        self.pseudo_legal_moves_container[depth as usize] = pseudo_legal_moves;
        self.legal_moves_container[depth as usize] = legal_moves;

        result_value
    }


    // WIP
    fn score_move(&mut self, mv: &ChessMove) -> i32 {
        // if mv.is_capture() {
            // victim_value - attacker_value
            let victim = self.board.piece_on_square(mv.to);
            let attacker = self.board.piece_on_square(mv.from);

            match (victim, attacker) {
                (Some(victim), Some(attacker)) => {
                    const MVV_LVA: [[u8; PieceType::ALL.len()]; PieceType::ALL.len()] = [
                        [15, 14, 13, 12, 11, 10,], // victim P, attacker P, N, B, R, Q, K,
                        [25, 24, 23, 22, 21, 20,], // victim N, attacker P, N, B, R, Q, K,
                        [35, 34, 33, 32, 31, 30,], // victim B, attacker P, N, B, R, Q, K,
                        [45, 44, 43, 42, 41, 40,], // victim R, attacker P, N, B, R, Q, K,
                        [55, 54, 53, 52, 51, 50,], // victim Q, attacker P, N, B, R, Q, K,
                        [ 0,  0,  0,  0,  0,  0,], // victim K, attacker P, N, B, R, Q, K,
                    ];

                    MVV_LVA[victim.0 as usize][attacker.0 as usize] as i32
                },
                (_,_) => {
                    if let Some(_) = mv.promotion {
                        1000
                    } else {
                        0
                    }
                }
            }
        // } else if mv.is_promotion() {
        //     1000
        // } else if self.gives_check(&mv) {
        //     500
        // } else {
        //     0
        // }
    }

    #[cfg_attr(feature = "tracy", tracing::instrument(skip_all))]
    pub fn find_best_move(&mut self, depth: u8) -> Option<ChessMove> {
        if depth as usize > self.max_search_depth {
            eprintln!("Error: Initial search depth ({}) exceeds the engine's configured max_depth ({}).", depth, self.max_search_depth);
            return None;
        }

        let mut pseudo_legal_moves = std::mem::take(&mut self.pseudo_legal_moves_container[depth as usize]);
        let mut legal_moves = std::mem::take(&mut self.legal_moves_container[depth as usize]);

        self.board.generate_legal_moves(&mut pseudo_legal_moves, &mut legal_moves);

        if legal_moves.is_empty() {
            return None;
        }

        legal_moves.sort_unstable_by_key(|mv| -self.score_move(mv));

        let board_hash = self.board.compute_zobrist_hash();
        *self.board_repetition_counts.entry(board_hash).or_insert(0) += 1;

        let mut best = None;
        let mut alpha = i32::MIN;
        let mut beta = i32::MAX;

        if self.board.turn == Color::White {
            let mut best_score = i32::MIN;
            for mv in legal_moves {
                let score = self.alphabeta(self.board.make_move(&mv), depth - 1, alpha, beta);
                if score > best_score {
                    best_score = score;
                    best = Some(mv);
                }
                alpha = max(alpha, best_score);
            }
        } else {
            let mut best_score = i32::MAX;
            for mv in legal_moves {
                let score = self.alphabeta(self.board.make_move(&mv), depth - 1, alpha, beta);
                if score < best_score {
                    best_score = score;
                    best = Some(mv);
                }
                beta = min(beta, best_score);
            }
        }

        *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;

        best
    }


    pub fn make_move(&mut self, mv: &ChessMove) {
        self.board = self.board.make_move(&mv);
        *self.board_repetition_counts.entry(self.board.compute_zobrist_hash()).or_insert(0) += 1;
    }

    pub fn print(&mut self) {
        println!("{}", self.board);
    }

    pub fn print_end(&mut self) {
        let mut pseudo_legal_moves = std::mem::take(&mut self.pseudo_legal_moves_container[0]);
        let mut legal_moves = std::mem::take(&mut self.legal_moves_container[0]);

        self.board.generate_legal_moves(&mut pseudo_legal_moves, &mut legal_moves);

        let board_hash = self.board.compute_zobrist_hash();
        let board_repetition_count = {
            let count_ref = self.board_repetition_counts.entry(board_hash).or_insert(0);
            *count_ref += 1;
            *count_ref
        };

        let game_state = self.board.check_game_state(legal_moves.is_empty(), board_repetition_count);

        print!("{:?}\n\n", game_state)
    }
}