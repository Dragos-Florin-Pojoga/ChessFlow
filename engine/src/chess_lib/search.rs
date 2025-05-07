use crate::bitboard::*;
use crate::board::*;
use crate::moves::*;
use crate::terminal_states::*;

impl Board {
    /// Minimax with alpha–beta pruning.
    /// `alpha` is the best score that the maximizer (White) can guarantee so far.
    /// `beta`  is the best score that the minimizer (Black) can guarantee so far.
    pub fn alphabeta(&self, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
        // Base case
        let game_state = self.check_game_state();
        if depth == 0 || game_state != GameState::Ongoing {
            return self.evaluate(depth);
        }

        let legal_moves = self.generate_legal_moves();

        if self.turn == Color::White {
            let mut value = i32::MIN;
            for mv in legal_moves {
                let new_board = self.make_move(&mv);
                let score = new_board.alphabeta(depth - 1, alpha, beta);
                value = value.max(score);
                alpha = alpha.max(value);
                // β-cutoff: the minimizer won’t let us get anything ≥ β
                if alpha >= beta {
                    break;
                }
            }
            value
        } else {
            let mut value = i32::MAX;
            for mv in legal_moves {
                let new_board = self.make_move(&mv);
                let score = new_board.alphabeta(depth - 1, alpha, beta);
                value = value.min(score);
                beta = beta.min(value);
                // α-cutoff: the maximizer won’t let us get anything ≤ α
                if beta <= alpha {
                    break;
                }
            }
            value
        }
    }

    // WIP
    fn score_move(&self, mv: &ChessMove) -> i32 {
        // if mv.is_capture() {
            // victim_value - attacker_value
            let victim = self.piece_on_square(mv.to);
            let attacker = self.piece_on_square(mv.from);

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

    pub fn find_best_move(&self, depth: u8) -> Option<ChessMove> {
        let mut legal_moves = self.generate_legal_moves();
        if legal_moves.is_empty() { return None; }
        legal_moves.sort_unstable_by_key(|mv| -self.score_move(mv)); 

        let mut best = None;
        // We initialize alpha, beta to the worst possible bounds
        let mut alpha = i32::MIN;
        let mut beta  = i32::MAX;

        if self.turn == Color::White {
            let mut best_score = i32::MIN;
            for mv in legal_moves {
                let score = self.make_move(&mv)
                                 .alphabeta(depth - 1, alpha, beta);
                if score > best_score {
                    best_score = score;
                    best = Some(mv);
                }
                alpha = alpha.max(best_score);
            }
        } else {
            let mut best_score = i32::MAX;
            for mv in legal_moves {
                let score = self.make_move(&mv)
                                 .alphabeta(depth - 1, alpha, beta);
                if score < best_score {
                    best_score = score;
                    best = Some(mv);
                }
                beta = beta.min(best_score);
            }
        }

        best
    }
}

// depth = 4
// 155745 ms
//  26599 ms
//  20784 ms