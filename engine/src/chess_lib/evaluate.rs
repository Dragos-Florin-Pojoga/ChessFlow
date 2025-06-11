use crate::board::*;
use crate::terminal_states::*;
use crate::simple_pst::*;

impl Board {
    /// Evaluates the current board from White’s perspective
    #[cfg_attr(feature = "tracy", tracing::instrument(skip_all))]
    pub fn evaluate(&self, remaining_depth: u8, pseudo_legal_moves: &mut Vec<ChessMove>, legal_moves: &mut Vec<ChessMove>, game_state : GameState) -> i32 {
        const PAWN_VALUE: i32   = 100;
        const KNIGHT_VALUE: i32 = 320;
        const BISHOP_VALUE: i32 = 330;
        const ROOK_VALUE: i32   = 500;
        const QUEEN_VALUE: i32  = 900;
        const MATE_SCORE: i32   = 10_000_000;

        // Fetch terminal-state first
        match game_state {
            GameState::Checkmate(checkmated) => {
                if checkmated == self.turn {
                    return -MATE_SCORE + remaining_depth as i32;
                } else {
                    return MATE_SCORE - remaining_depth as i32;
                }
            }
            GameState::Stalemate
            | GameState::FiftyMoveDraw
            | GameState::InsufficientMaterialDraw
            | GameState::ThreefoldRepetitionDraw => return 0,
            GameState::Ongoing => {} // fall through
        }

        // MATERIAL + PST
        let mut score = 0;
        for &(pt, val, pst) in &[
            (PieceType::Pawn,   PAWN_VALUE,   &PAWN_PST),
            (PieceType::Knight, KNIGHT_VALUE, &KNIGHT_PST),
            (PieceType::Bishop, BISHOP_VALUE, &BISHOP_PST),
            (PieceType::Rook,   ROOK_VALUE,   &ROOK_PST),
            (PieceType::Queen,  QUEEN_VALUE,  &QUEEN_PST),
        ] {
            let bb = self.piece_bbs[pt as usize];

            let wbb = bb & self.color_bbs[Color::White as usize];
            for sq in wbb.iter() {
                score += val + pst.get(sq as usize).unwrap();
            }

            let bbb = bb & self.color_bbs[Color::Black as usize];
            for sq in bbb.iter() {
                score -= val + pst.get(63 - sq as usize).unwrap();
            }
        }

        const PASSED_PAWN_BASE_BONUS: [i32; 8] = [0, 5, 15, 30, 50, 75, 100, 0]; // Bonus by rank (0 for rank 0/7 as it's not advanced)
                                                                                // Or for white: [0(1st), 5(2nd), 10(3rd), 20(4th), 40(5th), 70(6th), 120(7th), 0(8th-promo)]
                                                                                // Let's use a more common indexing for white pawns (rank 1 is index 1, rank 6 is index 6 for 7th rank)
        const WHITE_PASSED_PAWN_BONUS_BY_RANK: [i32; 8] = [0, 0, 10, 20, 40, 70, 120, 0]; // Index is rank (0-7), bonus for ranks 2-7 (actual ranks 3-8)
        const BLACK_PASSED_PAWN_BONUS_BY_RANK: [i32; 8] = [0, 120, 70, 40, 20, 10, 0, 0]; // Index is rank (0-7), bonus for ranks 0-5 (actual ranks 1-6)


        let white_pawns_bb = self.piece_bbs[PieceType::Pawn as usize] & self.color_bbs[Color::White as usize];
        for sq in white_pawns_bb.iter() {
            if self.is_passed_pawn(sq, Color::White) { // Assuming you add is_passed_pawn to Board
                score += WHITE_PASSED_PAWN_BONUS_BY_RANK[sq.rank() as usize];
            }
        }

        let black_pawns_bb = self.piece_bbs[PieceType::Pawn as usize] & self.color_bbs[Color::Black as usize];
        for sq in black_pawns_bb.iter() {
            if self.is_passed_pawn(sq, Color::Black) {
                score -= BLACK_PASSED_PAWN_BONUS_BY_RANK[sq.rank() as usize]; // Subtract for black's advantage
            }
        }

        // MOBILITY
        let my_moves = legal_moves.len() as i32;
        let opp_moves = {
            let mut flipped = self.clone();
            flipped.turn = self.turn.opponent();
            flipped.en_passant_square = None;
            flipped.generate_legal_moves(pseudo_legal_moves, legal_moves);
            legal_moves.len() as i32
        };
        score += (my_moves - opp_moves) * 2;

        // SYMMETRIC KING SAFETY (always from White's perspective)
        let mut white_king_safety = 0;
        let mut black_king_safety = 0;

        // White king safety
        if let Some(king_sq) = self.find_king_square(Color::White) {
            // Pawn shield
            let pawn_shield_offsets = [
                (-1, 1), (0, 1), (1, 1),
                (-1, 2), (0, 2), (1, 2),
            ];
            for (f, r) in pawn_shield_offsets.iter() {
                if let Some(sq) = king_sq.try_offset(*f, *r) {
                    if self.piece_bbs[PieceType::Pawn as usize].is_set(sq) && 
                       self.color_bbs[Color::White as usize].is_set(sq) {
                        white_king_safety += 15;
                    }
                }
            }

            // Attacks on king zone
            let attacked_by_black = self.get_attacked_squares(Color::Black);
            for sq in king_sq.surrounding_squares() {
                if attacked_by_black.is_set(sq) {
                    white_king_safety -= 10;
                }
            }
        }

        // Black king safety
        if let Some(king_sq) = self.find_king_square(Color::Black) {
            // Pawn shield (using negative ranks for black)
            let pawn_shield_offsets = [
                (-1, -1), (0, -1), (1, -1),
                (-1, -2), (0, -2), (1, -2),
            ];
            for (f, r) in pawn_shield_offsets.iter() {
                if let Some(sq) = king_sq.try_offset(*f, *r) {
                    if self.piece_bbs[PieceType::Pawn as usize].is_set(sq) && 
                       self.color_bbs[Color::Black as usize].is_set(sq) {
                        black_king_safety += 15;
                    }
                }
            }

            // Attacks on king zone
            let attacked_by_white = self.get_attacked_squares(Color::White);
            for sq in king_sq.surrounding_squares() {
                if attacked_by_white.is_set(sq) {
                    black_king_safety -= 10;
                }
            }
        }

        // Add king safety difference (White perspective)
        score += white_king_safety - black_king_safety;




        // --- Determine Game Phase (simplified) ---
        let mut total_material_no_kings_white = 0;
        let mut total_material_no_kings_black = 0;
        // (Assuming you have these constants defined elsewhere)
        // const PAWN_VALUE: i32   = 100; etc.
        for piece_type_val in [PieceType::Pawn, PieceType::Knight, PieceType::Bishop, PieceType::Rook, PieceType::Queen] {
            let piece_val = match piece_type_val {
                PieceType::Pawn => PAWN_VALUE,
                PieceType::Knight => KNIGHT_VALUE,
                PieceType::Bishop => BISHOP_VALUE,
                PieceType::Rook => ROOK_VALUE,
                PieceType::Queen => QUEEN_VALUE,
                _ => 0,
            };
            total_material_no_kings_white += (self.piece_bbs[piece_type_val as usize] & self.color_bbs[Color::White as usize]).popcount() as i32 * piece_val;
            total_material_no_kings_black += (self.piece_bbs[piece_type_val as usize] & self.color_bbs[Color::Black as usize]).popcount() as i32 * piece_val;
        }

        let is_endgame = total_material_no_kings_white < (QUEEN_VALUE + ROOK_VALUE) && // Example threshold: no queen and not too much other material
                        total_material_no_kings_black < (QUEEN_VALUE + ROOK_VALUE);
        // Or, a simpler one: no queens on board.
        // let no_queens = (self.piece_bbs[PieceType::Queen as usize]).is_empty();
        // let is_endgame = no_queens;


        if is_endgame {
            const KING_CENTRALIZATION_BONUS: i32 = 10; // Max bonus for being in the very center
            const KING_EDGE_PENALTY: i32 = -15; // Penalty for king on edge in endgame

            // White King
            if let Some(king_sq) = self.find_king_square(Color::White) {
                let r = king_sq.rank() as i32; // 0-7
                let f = king_sq.file() as i32; // 0-7
                // Bonus for centralization (distance from edges)
                score += (3 - (3 - f).abs() + 3 - (3 - r).abs()) * (KING_CENTRALIZATION_BONUS / 6); // Max 10 at center

                if r == 0 || r == 7 || f == 0 || f == 7 { // King on edge
                    score += KING_EDGE_PENALTY;
                }
            }
            // Black King
            if let Some(king_sq) = self.find_king_square(Color::Black) {
                let r = king_sq.rank() as i32;
                let f = king_sq.file() as i32;
                score -= (3 - (3 - f).abs() + 3 - (3 - r).abs()) * (KING_CENTRALIZATION_BONUS / 6);

                if r == 0 || r == 7 || f == 0 || f == 7 {
                    score -= KING_EDGE_PENALTY; // This becomes a bonus for white if black king is on edge
                }
            }
        }

        // Orient final score from White's perspective
        score
    }

    // This is a simplified version. More advanced versions would check adjacent files more accurately.
    fn is_passed_pawn(&self, sq: Square, pawn_color: Color) -> bool {
        let pawn_file = sq.file(); // Assuming 0-7
        let pawn_rank = sq.rank(); // Assuming 0-7

        let opponent_color = pawn_color.opponent();
        let opponent_pawns_bb = self.piece_bbs[PieceType::Pawn as usize] & self.color_bbs[opponent_color as usize];

        let mut forward_rank_iter = if pawn_color == Color::White {
            (pawn_rank + 1)..8
        } else {
            0..(pawn_rank) // Ranks 0 to pawn_rank-1
        };

        for r_ahead in forward_rank_iter {
            // Check same file
            if opponent_pawns_bb.is_set(Square::from_file_rank(pawn_file, r_ahead)) {
                return false;
            }
            // Check adjacent files (simplified: could be more precise about pawn attack squares)
            if pawn_file > 0 { // Check left adjacent file
                if opponent_pawns_bb.is_set(Square::from_file_rank(pawn_file - 1, r_ahead)) {
                    return false;
                }
            }
            if pawn_file < 7 { // Check right adjacent file
                if opponent_pawns_bb.is_set(Square::from_file_rank(pawn_file + 1, r_ahead)) {
                    return false;
                }
            }
        }
        true
    }
}
