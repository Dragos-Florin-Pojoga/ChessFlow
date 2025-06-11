use crate::board::*;


#[derive(Debug, PartialEq, Eq)]
pub enum GameState {
    Ongoing,
    Checkmate(Color), // Indicates the color of the player who is checkmated
    Stalemate,
    FiftyMoveDraw,
    InsufficientMaterialDraw,
    ThreefoldRepetitionDraw, // TODO: technically, this ALLOWS the player to claim a draw... FivefoldRepetitionDraw would be the one always enforced...
}

impl Board {

    pub fn check_game_state(&self, has_no_legal_moves: bool, board_repetition_count: u8) -> GameState {
        if board_repetition_count >= 3 {
            return GameState::ThreefoldRepetitionDraw;
        }
        
        if self.halfmove_clock >= 100 { // 100 half-moves = 50 full moves
            return GameState::FiftyMoveDraw;
        }

        if self.is_insufficient_material() {
            return GameState::InsufficientMaterialDraw;
        }

        // If there are no legal moves, it's either checkmate or stalemate
        if has_no_legal_moves {
            let king_square = self.find_king_square(self.turn);
            if let Some(king_sq) = king_square {
                if self.is_square_attacked(king_sq, self.turn.opponent()) {
                    // King is attacked and no legal moves
                    return GameState::Checkmate(self.turn);
                } else {
                    // King is not attacked and no legal moves
                    return GameState::Stalemate;
                }
            } else {
                unreachable!("Current player's king not found.");
            }
        }

        // If none of the above, the game is ongoing
        GameState::Ongoing
    }

    pub fn find_king_square(&self, color: Color) -> Option<Square> {
        let king_bb = self.piece_bbs[PieceType::King as usize] & self.color_bbs[color as usize];
        king_bb.iter().next()
    }

    /// This is a simplified check covering common cases.
    fn is_insufficient_material(&self) -> bool {
        let white_pieces = self.color_bbs[Color::White as usize];
        let black_pieces = self.color_bbs[Color::Black as usize];

        let white_knights = self.piece_bbs[PieceType::Knight as usize] & white_pieces;
        let white_bishops = self.piece_bbs[PieceType::Bishop as usize] & white_pieces;
        let white_rooks = self.piece_bbs[PieceType::Rook as usize] & white_pieces;
        let white_queens = self.piece_bbs[PieceType::Queen as usize] & white_pieces;
        let white_pawns = self.piece_bbs[PieceType::Pawn as usize] & white_pieces;

        let black_knights = self.piece_bbs[PieceType::Knight as usize] & black_pieces;
        let black_bishops = self.piece_bbs[PieceType::Bishop as usize] & black_pieces;
        let black_rooks = self.piece_bbs[PieceType::Rook as usize] & black_pieces;
        let black_queens = self.piece_bbs[PieceType::Queen as usize] & black_pieces;
        let black_pawns = self.piece_bbs[PieceType::Pawn as usize] & black_pieces;

        // If either side has a pawn, rook, or queen, there is sufficient material
        if !white_pawns.is_empty() || !black_pawns.is_empty() ||
           !white_rooks.is_empty() || !black_rooks.is_empty() ||
           !white_queens.is_empty() || !black_queens.is_empty() {
            return false;
        }

        // King and minor pieces only cases:

        let white_minor_pieces_count = white_knights.popcount() + white_bishops.popcount();
        let black_minor_pieces_count = black_knights.popcount() + black_bishops.popcount();

        // King vs King
        if white_minor_pieces_count == 0 && black_minor_pieces_count == 0 {
            return true;
        }

        // King and one minor piece vs King
        if (white_minor_pieces_count == 1 && black_minor_pieces_count == 0) ||
           (white_minor_pieces_count == 0 && black_minor_pieces_count == 1) {
            // King and Knight vs King, or King and Bishop vs King
            return true;
        }

        // King and multiple Knights vs King (can't force checkmate)
        if white_bishops.is_empty() && white_rooks.is_empty() && white_queens.is_empty() && white_pawns.is_empty() &&
        black_bishops.is_empty() && black_rooks.is_empty() && black_queens.is_empty() && black_pawns.is_empty() &&
        white_knights.popcount() > 0 && black_knights.popcount() == 0 {
            // White has only King and Knights, Black has only King
            return true; // Assuming multiple knights without other pieces can't force mate against a lone king
        }

        if black_bishops.is_empty() && black_rooks.is_empty() && black_queens.is_empty() && black_pawns.is_empty() &&
        white_bishops.is_empty() && white_rooks.is_empty() && white_queens.is_empty() && white_pawns.is_empty() &&
        black_knights.popcount() > 0 && white_knights.popcount() == 0 {
            // Black has only King and Knights, White has only King
            return true; // Assuming multiple knights without other pieces can't force mate against a lone king
        }

        // Note: King and two Knights vs King is a theoretical checkmate,
        // but hard to force and often considered insufficient material in practice
        // depending on the specific engine's rules. This simplified check
        // treats K+NN vs K as insufficient.

        // Check for K + same-colored Bishops vs K
        if white_pawns.is_empty() && black_pawns.is_empty() &&
            white_rooks.is_empty() && black_rooks.is_empty() &&
            white_queens.is_empty() && black_queens.is_empty() &&
            white_knights.is_empty() && black_knights.is_empty() 
            { // Only Kings and Bishops remain
                if white_bishops.popcount() > 0 {
                    let mut first_bishop_square_color: Option<bool> = None; // true for light, false for dark
                    let mut all_same_color = true;
                    for sq in white_bishops.iter() {
                        let sq_is_light = sq.is_light();
                        if let Some(first_color) = first_bishop_square_color {
                            if first_color != sq_is_light {
                                all_same_color = false;
                                break;
                            }
                        } else {
                            first_bishop_square_color = Some(sq_is_light);
                        }
                    }
                    if all_same_color {
                        return true; // White has only K and bishops on same-colored squares
                    }
                }
                if black_bishops.popcount() > 0 {
                    // similar logic for black bishops
                    let mut first_bishop_square_color: Option<bool> = None;
                    let mut all_same_color = true;
                    for sq in black_bishops.iter() {
                        let sq_is_light = sq.is_light();
                        if let Some(first_color) = first_bishop_square_color {
                            if first_color != sq_is_light {
                                all_same_color = false;
                                break;
                            }
                        } else {
                            first_bishop_square_color = Some(sq_is_light);
                        }
                    }
                    if all_same_color {
                        return true; // Black has only K and bishops on same-colored squares
                    }
                }
        }

        false // Assume sufficient material otherwise
    }
}
