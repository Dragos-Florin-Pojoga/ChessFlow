use crate::board::*;
use crate::terminal_states::*;
use crate::game::*;
use std::cmp::{max, min};


pub fn get_qsearch_piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 100,
        PieceType::Knight => 320,
        PieceType::Bishop => 330,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 0,
    }
}

impl Game {
    /// The Alpha-Beta search algorithm.
    #[cfg_attr(feature = "tracy", tracing::instrument(skip_all))]
    pub fn alphabeta(&mut self, board: Board, depth: u8, mut alpha: i32, mut beta: i32) -> i32 {
        let original_alpha = alpha;
        let board_hash = board.compute_zobrist_hash();

        let board_repetition_count = {
            let count_ref = self.board_repetition_counts.entry(board_hash).or_insert(0);
            *count_ref += 1;
            *count_ref
        };

        if board_repetition_count >= 3 {
            *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
            self.transposition_table.insert(
                board_hash,
                TTEntry {
                    score: 0, depth, node_type: NodeType::Exact, best_move: None,
                },
            );
            return 0;
        }

        if let Some(entry) = self.transposition_table.get(&board_hash) {
            if entry.depth >= depth {
                // Check if TT hit causes immediate return
                let mut tt_causes_return = false;
                let score_from_tt = entry.score;

                match entry.node_type {
                    NodeType::Exact => tt_causes_return = true,
                    NodeType::Alpha => { // TT score is a lower bound
                        if entry.score >= beta { tt_causes_return = true; }
                        alpha = max(alpha, entry.score);
                    }
                    NodeType::Beta => { // TT score is an upper bound
                        if entry.score <= alpha { tt_causes_return = true; }
                        beta = min(beta, entry.score);
                    }
                }

                if tt_causes_return {
                    *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
                    return score_from_tt;
                }
                // If TT only updated bounds, check if they now cross
                if alpha >= beta {
                     *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
                    // Return the TT score that caused the cutoff
                    return entry.score;
                }
            }
        }

        let mut pseudo_legal_moves = std::mem::take(&mut self.pseudo_legal_moves_container[depth as usize]);
        let mut legal_moves = std::mem::take(&mut self.legal_moves_container[depth as usize]);
        board.generate_legal_moves(&mut pseudo_legal_moves, &mut legal_moves);

        let game_state = board.check_game_state(legal_moves.is_empty(), board_repetition_count);

        // 4. Base Case (Terminal Node - other than repetition draw)
        if game_state != GameState::Ongoing {
            let eval = board.evaluate(depth, &mut pseudo_legal_moves, &mut legal_moves, game_state);
            *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
            self.pseudo_legal_moves_container[depth as usize] = pseudo_legal_moves;
            self.legal_moves_container[depth as usize] = legal_moves;
            let node_type = if eval <= original_alpha { NodeType::Alpha }
                            else if eval >= beta { NodeType::Beta }
                            else { NodeType::Exact };
            self.transposition_table.insert(
                board_hash,
                TTEntry { score: eval, depth, node_type, best_move: None },
            );
            return eval;
        }

        // 4b. Base Case: Depth Limit Reached (but game is ongoing) -> Call Quiescence Search
        if depth == 0 {
            // Quiescence search will perform its own evaluation.
            // The `board` is passed by value to qsearch.
            // `alpha` and `beta` are passed along.
            let q_score = self.qsearch(board, alpha, beta, self.q_search_max_ply);

            // Cleanup for the depth 0 node of alphabeta
            *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
            // Return the original containers, qsearch doesn't use these specific instances
            self.pseudo_legal_moves_container[depth as usize] = pseudo_legal_moves;
            self.legal_moves_container[depth as usize] = legal_moves;

            let node_type_for_tt = if q_score <= original_alpha { NodeType::Alpha }
                                   else if q_score >= beta { NodeType::Beta }
                                   else { NodeType::Exact };
            self.transposition_table.insert(
                board_hash,
                TTEntry {
                    score: q_score,
                    depth, // depth is 0 here
                    node_type: node_type_for_tt,
                    best_move: None, // No single "best move" from qsearch evaluation
                },
            );
            return q_score;
        }

        // 5. Move Ordering (for depth > 0)
        let tt_best_move = self.transposition_table.get(&board_hash).and_then(|entry| entry.best_move);
        legal_moves.sort_unstable_by_key(|mv| -self.score_move(mv, depth, tt_best_move));

        let mut result_value;
        let mut best_move_for_tt: Option<ChessMove> = None;

        // 6. Alpha-Beta Search Loop
        if board.turn == Color::White { // Maximizing player
            let mut value = i32::MIN; // Negative infinity
            for mv in &legal_moves {
                let new_board = board.make_move(&mv);
                let score = self.alphabeta(new_board, depth - 1, alpha, beta);
                if score > value {
                    value = score;
                    best_move_for_tt = Some(*mv);
                }
                alpha = max(alpha, value);
                if alpha >= beta {
                    if !mv.is_capture() {
                        let current_killer_moves = &mut self.killer_moves[depth as usize];
                        if current_killer_moves[0].is_none() || current_killer_moves[0] != Some(*mv) {
                            current_killer_moves[1] = current_killer_moves[0];
                            current_killer_moves[0] = Some(*mv);
                        }
                        self.history_moves[mv.from() as usize][mv.to() as usize] += depth as i32;
                    }
                    break;
                }
            }
            result_value = value;
        } else { // Color::Black (Minimizing player)
            let mut value = i32::MAX; // Positive infinity
            for mv in &legal_moves {
                let new_board = board.make_move(&mv);
                let score = self.alphabeta(new_board, depth - 1, alpha, beta);
                if score < value {
                    value = score;
                    best_move_for_tt = Some(*mv);
                }
                beta = min(beta, value);
                if beta <= alpha {
                    if !mv.is_capture() {
                        let current_killer_moves = &mut self.killer_moves[depth as usize];
                        if current_killer_moves[0].is_none() || current_killer_moves[0] != Some(*mv) {
                            current_killer_moves[1] = current_killer_moves[0];
                            current_killer_moves[0] = Some(*mv);
                        }
                        self.history_moves[mv.from() as usize][mv.to() as usize] += depth as i32;
                    }
                    break;
                }
            }
            result_value = value;
        }

        // 7. Cleanup and Transposition Table Store
        *self.board_repetition_counts.get_mut(&board_hash).unwrap() -= 1;
        self.pseudo_legal_moves_container[depth as usize] = pseudo_legal_moves;
        self.legal_moves_container[depth as usize] = legal_moves;

        let node_type = if result_value <= original_alpha { NodeType::Alpha }
                        else if result_value >= beta { NodeType::Beta }
                        else { NodeType::Exact };
        self.transposition_table.insert(
            board_hash,
            TTEntry {
                score: result_value, depth, node_type, best_move: best_move_for_tt,
            },
        );

        result_value
    }

    /// Quiescence Search: Explores tactical moves (captures, promotions) from a given position.
    #[cfg_attr(feature = "tracy", tracing::instrument(skip_all))]
    fn qsearch(
        &mut self,
        board: Board, // Current board state for this qsearch node
        mut alpha: i32,
        mut beta: i32,
        q_depth: u8, // Remaining quiescence search depth
    ) -> i32 {
        // 1. Check quiescence depth limit
        if q_depth == 0 {
            // Depth limit reached, return static evaluation of the current position.
            // For `board.evaluate`, `game_state` is `Ongoing` because qsearch is only
            // entered if the main search determined the game is ongoing.
            // `remaining_depth` for evaluate's own logic can be 0.
            let mut temp_pseudo = Vec::new(); // Placeholder if evaluate needs them for mobility
            let mut temp_legal = Vec::new();  // Placeholder
            // If evaluate *needs* accurate mobility for stand-pat, generate moves for `board` here:
            // board.generate_legal_moves(&mut temp_pseudo, &mut temp_legal);
            return board.evaluate(0, &mut temp_pseudo, &mut temp_legal, GameState::Ongoing);
        }

        // 2. Stand-pat evaluation
        // Score if the current player makes no tactical move.
        // For an accurate stand-pat that includes mobility, generate all legal moves for the current `board`.
        let mut stand_pat_pseudo_moves = Vec::new();
        let mut stand_pat_legal_moves = Vec::new();
        let stand_pat_score = board.evaluate(0, &mut stand_pat_pseudo_moves, &mut stand_pat_legal_moves, GameState::Ongoing);
        board.generate_legal_moves(&mut stand_pat_pseudo_moves, &mut stand_pat_legal_moves);


        // 3. Alpha-Beta pruning based on stand-pat
        // This is the score that can be achieved if no tactical sequence improves it.
        if board.turn == Color::White { // Maximizing player
            if stand_pat_score >= beta {
                return stand_pat_score; // Fail-high: stand-pat is already too good for opponent
            }
            alpha = max(alpha, stand_pat_score);
        } else { // Minimizing player (Black)
            if stand_pat_score <= alpha {
                return stand_pat_score; // Fail-low: stand-pat is already too good for us
            }
            beta = min(beta, stand_pat_score);
        }

        // 4. Generate and score tactical moves (captures, promotions)
        // `stand_pat_legal_moves` already contains all legal moves for the current `board`.
        // We filter these for tactical moves.
        let mut tactical_moves = stand_pat_legal_moves; // Start with all legal moves
        tactical_moves.retain(|mv| mv.is_capture() || mv.promotion().is_some()); // Keep only captures/promotions

        // Order tactical moves: Promotions first, then MVV-LVA for captures.
        // Higher score_key means better move.
        tactical_moves.sort_unstable_by_key(|mv| {
            let mut score_key = 0;
            if let Some(promoted_piece) = mv.promotion() {
                score_key += 10000 + get_qsearch_piece_value(promoted_piece); // Promotions are high priority
            }
            if mv.is_capture() {
                // MVV-LVA: Most Valuable Victim - Least Valuable Attacker
                // A higher score_key for more valuable captures.
                if let Some(victim_piece_info) = board.piece_on_square(mv.to()) { // Assumes piece_on_square returns Option<(PieceType, Color)>
                    score_key += get_qsearch_piece_value(victim_piece_info.0) * 10; // Victim value weighted higher
                    if let Some(attacker_piece_info) = board.piece_on_square(mv.from()) {
                        score_key -= get_qsearch_piece_value(attacker_piece_info.0); // Subtract attacker value
                    }
                } else {
                    // Should not happen for a valid capture, but as a fallback
                    score_key += 50; // Generic capture bonus
                }
            }
            -score_key // Negate because sort_unstable_by_key sorts by ascending key
        });

        // 5. Iterate through tactical moves
        if board.turn == Color::White { // Maximizing player
            let mut current_best_score = stand_pat_score; // Initialize with stand-pat
            for mv in &tactical_moves {
                let new_board = board.make_move(&mv);
                // Recursively call qsearch for the new board state
                let score = self.qsearch(new_board, alpha, beta, q_depth - 1);
                current_best_score = max(current_best_score, score);
                alpha = max(alpha, current_best_score);
                if alpha >= beta {
                    break; // Beta cutoff
                }
            }
            return current_best_score;
        } else { // Minimizing player (Black)
            let mut current_best_score = stand_pat_score; // Initialize with stand-pat
            for mv in &tactical_moves {
                let new_board = board.make_move(&mv);
                // Recursively call qsearch for the new board state
                let score = self.qsearch(new_board, alpha, beta, q_depth - 1);
                current_best_score = min(current_best_score, score);
                beta = min(beta, current_best_score);
                if beta <= alpha {
                    break; // Alpha cutoff
                }
            }
            return current_best_score;
        }
    }
}


impl Game {
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
            let victim = self.board.piece_on_square(mv.to());
            let attacker = self.board.piece_on_square(mv.from());

            match (victim, attacker) {
                (Some(victim_piece), Some(attacker_piece)) => {
                    // Add a large offset to make captures generally higher priority than non-captures
                    MVV_LVA[victim_piece.0 as usize][attacker_piece.0 as usize] as i32 + 1_000_000
                },
                _ => 0, // Should ideally not happen for a legal capture, but a fallback.
            }
        } else if let Some(_) = mv.promotion() {
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
            score += self.history_moves[mv.from() as usize][mv.to() as usize];

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
        if depth > self.max_search_depth {
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