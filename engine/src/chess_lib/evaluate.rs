use crate::bitboard::*;
use crate::board::*;
use crate::terminal_states::*;
use crate::simple_pst::*;

impl Board {
    /// Evaluates the current board from White’s perspective
    pub fn evaluate(&self, remaining_depth: u8) -> i32 {
        const PAWN_VALUE: i32   = 100;
        const KNIGHT_VALUE: i32 = 320;
        const BISHOP_VALUE: i32 = 330;
        const ROOK_VALUE: i32   = 500;
        const QUEEN_VALUE: i32  = 900;
        const MATE_SCORE: i32   = 10_000;

        // Fetch terminal-state first
        match self.check_game_state() {
            GameState::Checkmate(checkmated) => {
                // + for a win, – for a loss, scaled by depth-to-go
                if checkmated == self.turn {
                    return -MATE_SCORE + remaining_depth as i32;
                } else {
                    return MATE_SCORE - remaining_depth as i32;
                }
            }
            GameState::Stalemate
            | GameState::FiftyMoveDraw
            | GameState::InsufficientMaterialDraw => return 0,
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
                // we mirror the PST for Black
            }
        }

        // MOBILITY: difference in legal‐move counts × 50 centipawns
        let my_moves = self.generate_legal_moves().len() as i32;
        let opp_moves = {
            let mut flipped = self.clone();
            flipped.turn = self.turn.opponent();
            flipped.generate_legal_moves().len() as i32
        };
        score += (my_moves - opp_moves) * 50;

        // Finally, orient from White’s POV
        if self.turn == Color::White { score } else { -score }
    }
}
