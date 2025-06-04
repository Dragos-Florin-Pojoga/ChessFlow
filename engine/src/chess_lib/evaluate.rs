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

        // MOBILITY
        let my_moves = legal_moves.len() as i32;
        let opp_moves = {
            let mut flipped = self.clone();
            flipped.turn = self.turn.opponent();
            flipped.generate_legal_moves(pseudo_legal_moves, legal_moves);
            legal_moves.len() as i32
        };
        score += (my_moves - opp_moves) * 50;

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

        // Orient final score from White's perspective
        if self.turn == Color::White { score } else { -score }
    }
}
