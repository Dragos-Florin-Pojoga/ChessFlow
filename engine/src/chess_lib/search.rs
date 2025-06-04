use crate::board::*;
use crate::terminal_states::*;
use crate::game::*;
use std::cmp::max;
use std::cmp::min;


impl Game {
    /// The Alpha-Beta search algorithm.
    ///
    /// This function recursively searches the game tree to find the best move.
    /// It uses Alpha-Beta pruning to cut off branches that cannot possibly lead
    /// to a better score than already found.
    ///
    /// Arguments:
    /// - `board`: The current board state.
    /// - `depth`: The remaining search depth.
    /// - `alpha`: The alpha value (best score found so far for the maximizing player).
    /// - `beta`: The beta value (best score found so far for the minimizing player).
    ///
    /// Returns: The evaluated score for the current board state.
    #[cfg_attr(feature = "tracy", tracing::instrument(skip_all))]
    pub fn alphabeta(&mut self, board: Board, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
        let original_alpha = alpha; // Store original alpha for transposition table entry
        let board_hash = board.compute_zobrist_hash();

        // 1. Update board repetition count for the current hash.
        // This must happen *before* any repetition check or TT lookup that might skip it.
        let board_repetition_count = {
            let count_ref = self.board_repetition_counts.entry(board_hash).or_insert(0);
            *count_ref += 1;
            *count_ref
        };

        // 2. Immediate Repetition Draw Check
        // If the current position is a threefold repetition, it's a draw regardless of search depth.
        if board_repetition_count >= 3 {
            // Decrement repetition count before returning.
            *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
            // Store this draw in the transposition table.
            self.transposition_table.insert(
                board_hash,
                TTEntry {
                    score: 0, // Draw score
                    depth,
                    node_type: NodeType::Exact,
                    best_move: None,
                },
            );
            return 0; // Return draw score
        }

        // 3. Transposition Table Lookup (after repetition check)
        // Check if this position has been searched before at a sufficient depth.
        // Suggested Fix Sketch for TT section in alphabeta:
        if let Some(entry) = self.transposition_table.get(&board_hash) {
            if entry.depth >= depth {
                let mut tt_causes_return = false;
                let mut score_from_tt = entry.score;

                match entry.node_type {
                    NodeType::Exact => {
                        tt_causes_return = true;
                    }
                    NodeType::Alpha => { // TT score is a lower bound
                        if entry.score >= beta { // This lower bound causes a cutoff
                            tt_causes_return = true;
                            score_from_tt = entry.score; // Or beta
                        }
                        alpha = max(alpha, entry.score);
                    }
                    NodeType::Beta => { // TT score is an upper bound
                        if entry.score <= alpha { // This upper bound causes a cutoff
                            tt_causes_return = true;
                            score_from_tt = entry.score; // Or alpha
                        }
                        beta = min(beta, entry.score);
                    }
                }

                if tt_causes_return {
                    *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1; // Decrement only if returning
                    // Potentially update TT entry here before returning
                    return score_from_tt;
                }
                // If not returning, alpha/beta might have been updated.
                // The repetition count remains (correctly) incremented from the function start.
                // Check if the updated alpha/beta now cause a cutoff.
                if alpha >= beta {
                    *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1; // Decrement before this return path
                    // This return uses the score from the TT entry that caused the bounds to cross,
                    // which is `entry.score` (the value that was just assigned to alpha or beta to cause the cutoff).
                    return entry.score;
                }
            }
        }

        // Retrieve move containers for the current depth to avoid reallocations.
        let mut pseudo_legal_moves = std::mem::take(&mut self.pseudo_legal_moves_container[depth as usize]);
        let mut legal_moves = std::mem::take(&mut self.legal_moves_container[depth as usize]);

        // Generate legal moves for the current board.
        board.generate_legal_moves(&mut pseudo_legal_moves, &mut legal_moves);

        // Determine the game state (ongoing, checkmate, stalemate, draw).
        // Note: Repetition draw is already handled above. This is for checkmate/stalemate.
        let game_state = board.check_game_state(legal_moves.is_empty(), board_repetition_count);

        // 4. Base Case (Terminal Node - other than repetition draw)
        // If search depth is 0, game is over (checkmate/stalemate), or no legal moves.
        if depth == 0 || game_state != GameState::Ongoing || legal_moves.is_empty() {
            // Evaluate the board statically.
            // IMPORTANT: `board.evaluate` must correctly handle `GameState::Checkmate` (return +/- infinity)
            // and `GameState::Stalemate`/Draws (return 0).
            let eval = board.evaluate(depth, &mut pseudo_legal_moves, &mut legal_moves, game_state);

            // Decrement repetition count before returning.
            *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
            // Return move containers to their respective slots.
            self.pseudo_legal_moves_container[depth as usize] = pseudo_legal_moves;
            self.legal_moves_container[depth as usize] = legal_moves;

            // Store result in transposition table.
            let node_type = if eval <= original_alpha {
                NodeType::Alpha // Score is a lower bound (failed low)
            } else if eval >= beta {
                NodeType::Beta // Score is an upper bound (failed high)
            } else {
                NodeType::Exact // Exact score
            };
            self.transposition_table.insert(
                board_hash,
                TTEntry {
                    score: eval,
                    depth,
                    node_type,
                    best_move: None, // No best move for terminal nodes
                },
            );
            return eval;
        }

        // 5. Move Ordering
        // Get the best move from the transposition table (if available) to try first.
        let tt_best_move = self.transposition_table.get(&board_hash).and_then(|entry| entry.best_move);
        // Sort legal moves based on their heuristic score (descending).
        legal_moves.sort_unstable_by_key(|mv| -self.score_move(mv, depth, tt_best_move));

        let mut result_value;
        let mut best_move_for_tt: Option<ChessMove> = None; // To store the best move found in this node for TT

        // 6. Alpha-Beta Search Loop
        if board.turn == Color::White { // Maximizing player
            let mut value = i32::MIN;
            for mv in &legal_moves {
                let new_board = board.make_move(&mv);
                let score = self.alphabeta(new_board, depth - 1, alpha, beta);
                if score > value {
                    value = score;
                    best_move_for_tt = Some(*mv); // Update best move
                }
                alpha = max(alpha, value); // Update alpha
                if alpha >= beta {
                    // Beta cutoff (fail-high): Current move is too good for the opponent.
                    // Store this move as a killer move and update history.
                    if !mv.is_capture() { // Only for non-capture moves
                        let current_killer_moves = &mut self.killer_moves[depth as usize];
                        if current_killer_moves[0].is_none() || current_killer_moves[0] != Some(*mv) {
                            current_killer_moves[1] = current_killer_moves[0]; // Shift existing killer
                            current_killer_moves[0] = Some(*mv);              // New killer move
                        }
                        // Update history score for this move. Higher depth cutoffs are more valuable.
                        self.history_moves[mv.from as usize][mv.to as usize] += depth as i32;
                    }
                    break; // Prune the rest of the branches
                }
            }
            result_value = value;
        } else { // Color::Black (Minimizing player)
            let mut value = i32::MAX;
            for mv in &legal_moves {
                let new_board = board.make_move(&mv);
                let score = self.alphabeta(new_board, depth - 1, alpha, beta);
                if score < value {
                    value = score;
                    best_move_for_tt = Some(*mv); // Update best move
                }
                beta = min(beta, value); // Update beta
                if beta <= alpha {
                    // Alpha cutoff (fail-low): Current move is too bad for us.
                    // Store this move as a killer move and update history.
                    if !mv.is_capture() { // Only for non-capture moves
                        let current_killer_moves = &mut self.killer_moves[depth as usize];
                        if current_killer_moves[0].is_none() || current_killer_moves[0] != Some(*mv) {
                            current_killer_moves[1] = current_killer_moves[0];
                            current_killer_moves[0] = Some(*mv);
                        }
                        // Update history score for this move.
                        self.history_moves[mv.from as usize][mv.to as usize] += depth as i32;
                    }
                    break; // Prune the rest of the branches
                }
            }
            result_value = value;
        }

        // 7. Cleanup and Transposition Table Store
        // Decrement repetition count for the current hash.
        *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
        // Return move containers to their respective slots.
        self.pseudo_legal_moves_container[depth as usize] = pseudo_legal_moves;
        self.legal_moves_container[depth as usize] = legal_moves;

        // Store result in transposition table.
        let node_type = if result_value <= original_alpha {
            NodeType::Alpha
        } else if result_value >= beta {
            NodeType::Beta
        } else {
            NodeType::Exact
        };
        self.transposition_table.insert(
            board_hash,
            TTEntry {
                score: result_value,
                depth,
                node_type,
                best_move: best_move_for_tt,
            },
        );

        result_value
    }

    /// Scores a chess move based on various heuristics for move ordering.
    /// Higher scores mean the move should be tried earlier.
    ///
    /// Arguments:
    /// - `mv`: The `ChessMove` to score.
    /// - `depth`: The current search depth.
    /// - `tt_best_move`: The best move found for this position from the transposition table (if any).
    ///
    /// Returns: An `i32` score representing the move's priority.
    fn score_move(&mut self, mv: &ChessMove, depth: u8, tt_best_move: Option<ChessMove>) -> i32 {
        // 1. Prioritize the best move from the transposition table (highest priority)
        if let Some(tt_mv) = tt_best_move {
            if *mv == tt_mv {
                return 2_000_000;
            }
        }

        // 2. MVV-LVA table
        const MVV_LVA: [[u16; 6]; 6] = [
 // Attacker: P,   N,   B,   R,   Q,   K
            [105, 104, 103, 102, 101, 100], // Victim Pawn
            [205, 204, 203, 202, 201, 200], // Victim Knight
            [305, 304, 303, 302, 301, 300], // Victim Bishop
            [405, 404, 403, 402, 401, 400], // Victim Rook
            [505, 504, 503, 502, 501, 500], // Victim Queen
            [  0,   0,   0,   0,   0,   0], // Victim King (should not be captured directly in legal moves)
        ];

        if mv.is_capture() {
            let victim = self.board.piece_on_square(mv.to);
            let attacker = self.board.piece_on_square(mv.from);

            match (victim, attacker) {
                (Some(victim_piece), Some(attacker_piece)) => {
                    // Add a large offset to make captures generally higher priority than non-captures
                    MVV_LVA[victim_piece.0 as usize][attacker_piece.0 as usize] as i32 + 1_000_000
                },
                _ => 0, // Should ideally not happen for a legal capture, but a fallback.
            }
        } else if let Some(_) = mv.promotion {
            // 3. Promotions (high priority)
            1_000_000 // High score for promotions
        } else {
            // 4. Non-capture moves: Killer moves and History heuristic
            let mut score = 0;

            // Killer moves: Check if this move is one of the killer moves for the current depth
            let killers = &self.killer_moves[depth as usize];
            if let Some(k1) = killers[0] {
                if *mv == k1 {
                    score += 900_000; // High score for first killer move
                }
            }
            if let Some(k2) = killers[1] {
                if *mv == k2 {
                    score += 800_000; // High score for second killer move
                }
            }

            // History heuristic: Score based on past success of this move in causing cutoffs
            score += self.history_moves[mv.from as usize][mv.to as usize];

            // 5. Check-giving moves (Optional, requires `board.is_check_after_move` or similar)
            // if self.board.is_check_after_move(mv) {
            //     score += 500_000; // Add a bonus for giving check
            // }

            score
        }
    }

    /// Finds the best move for the current board state using the Alpha-Beta search.
    ///
    /// Arguments:
    /// - `depth`: The maximum search depth.
    ///
    /// Returns: An `Option<ChessMove>` representing the best move found, or `None` if no legal moves.
    #[cfg_attr(feature = "tracy", tracing::instrument(skip_all))]
    pub fn find_best_move(&mut self, depth: u8) -> Option<ChessMove> {
        // Error check for initial search depth.
        if depth as usize > self.max_search_depth {
            eprintln!("Error: Initial search depth ({}) exceeds the engine's configured max_depth ({}).", depth, self.max_search_depth);
            return None;
        }

        // Retrieve move containers for the initial depth.
        let mut pseudo_legal_moves = std::mem::take(&mut self.pseudo_legal_moves_container[depth as usize]);
        let mut legal_moves = std::mem::take(&mut self.legal_moves_container[depth as usize]);

        // Generate legal moves for the starting board.
        self.board.generate_legal_moves(&mut pseudo_legal_moves, &mut legal_moves);

        if legal_moves.is_empty() {
            // No legal moves, so no best move can be found.
            return None;
        }

        // Get TT best move for initial move ordering at the root.
        let board_hash = self.board.compute_zobrist_hash();
        let tt_best_move = self.transposition_table.get(&board_hash).and_then(|entry| entry.best_move);

        // Sort moves using the enhanced scoring function to prioritize good moves.
        legal_moves.sort_unstable_by_key(|mv| -self.score_move(mv, depth, tt_best_move));

        let mut best = None;
        let mut alpha = i32::MIN;
        let mut beta = i32::MAX;

        // Perform the Alpha-Beta search for the root node.
        if self.board.turn == Color::White { // Maximizing player
            let mut best_score = i32::MIN;
            for mv in legal_moves {
                let score = self.alphabeta(self.board.make_move(&mv), depth - 1, alpha, beta);
                if score > best_score {
                    best_score = score;
                    best = Some(mv);
                }
                alpha = max(alpha, best_score); // Update alpha for the root node
            }
        } else { // Color::Black (Minimizing player)
            let mut best_score = i32::MAX;
            for mv in legal_moves {
                let score = self.alphabeta(self.board.make_move(&mv), depth - 1, alpha, beta);
                if score < best_score {
                    best_score = score;
                    best = Some(mv);
                }
                beta = min(beta, best_score); // Update beta for the root node
            }
        }

        best
    }
}