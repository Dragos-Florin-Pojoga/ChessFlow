use crate::board::*;

impl Board {
    /// Generates all pseudo-legal moves for the current player.
    /// Pseudo-legal moves are moves that are valid for the piece type,
    /// but do not check if the king is left in check.
    pub fn generate_pseudo_legal_moves(&self, moves: &mut Vec<ChessMove>) {
        let player_color = self.turn;
        
        self.generate_pawn_moves(moves, player_color);
        self.generate_leaper_moves(moves, player_color, PieceType::Knight);
        self.generate_king_moves(moves, player_color); // Includes castling
        self.generate_sliding_moves(moves, player_color, PieceType::Bishop);
        self.generate_sliding_moves(moves, player_color, PieceType::Rook);
        self.generate_sliding_moves(moves, player_color, PieceType::Queen);
    }

    fn generate_pawn_moves(&self, moves: &mut Vec<ChessMove>, color: Color) {
        let pawns_bb = self.piece_bbs[PieceType::Pawn as usize] & self.color_bbs[color as usize];
        let empty_squares = self.empty_squares_bb();
        let opponent_bb = self.opponent_pieces_bb();

        let (push_one_step, push_two_step_rank, promotion_rank, ep_rank) = match color {
            Color::White => (1, PRECOMPUTED.rank_2_bb, Square::A8.rank(), Square::A5.rank()), // White pushes up (positive offset)
            Color::Black => (-1, PRECOMPUTED.rank_7_bb, Square::A1.rank(), Square::A4.rank()), // Black pushes down (negative offset)
        };

        for from_sq in pawns_bb.iter() {
            // Single Push
            if let Some(to_sq) = from_sq.try_offset(0, push_one_step) {
                if empty_squares.is_set(to_sq) {
                    if to_sq.rank() == promotion_rank {
                        for &promo_piece in PieceType::PROMOTION_PIECES.iter() {
                            moves.push(ChessMove::new(from_sq, to_sq, Some(promo_piece)));
                        }
                    } else {
                        moves.push(ChessMove::new(from_sq, to_sq, None));
                    }

                    // Double Push (only if single push was possible and pawn is on starting rank)
                    if (Bitboard::from_square(from_sq) & push_two_step_rank).is_not_empty() {
                         if let Some(double_to_sq) = to_sq.try_offset(0, push_one_step) {
                            if empty_squares.is_set(double_to_sq) {
                                moves.push(ChessMove::new(from_sq, double_to_sq, None));
                            }
                        }
                    }
                }
            }

            // Captures
            let pawn_attacks_bb = PRECOMPUTED.pawn_attacks[color as usize][from_sq.to_u8() as usize];
            let possible_captures = pawn_attacks_bb & opponent_bb;
            for to_sq in possible_captures.iter() {
                if to_sq.rank() == promotion_rank {
                    for &promo_piece in PieceType::PROMOTION_PIECES.iter() {
                        moves.push(ChessMove::new_capture(from_sq, to_sq, Some(promo_piece)));
                    }
                } else {
                    moves.push(ChessMove::new_capture(from_sq, to_sq, None));
                }
            }
            
            // En Passant
            if let Some(ep_sq) = self.en_passant_square {
                if from_sq.rank() == ep_rank { // Pawn must be on the correct rank for EP
                    if (pawn_attacks_bb & Bitboard::from_square(ep_sq)).is_not_empty() {
                        moves.push(ChessMove::new_capture(from_sq, ep_sq, None)); // NOTE: EP is a special capture but is not flagged in the move
                    }
                }
            }
        }
    }
    
    fn generate_leaper_moves(&self, moves: &mut Vec<ChessMove>, color: Color, piece_type: PieceType) {
        let pieces_bb = self.piece_bbs[piece_type as usize] & self.color_bbs[color as usize];
        let friendly_bb = self.color_bbs[color as usize];
        
        let attack_lut = match piece_type {
            PieceType::Knight => &PRECOMPUTED.knight_attacks,
            _ => unreachable!("Not a leaper"),
        };

        for from_sq in pieces_bb.iter() {
            let attacks = attack_lut[from_sq.to_u8() as usize] & !friendly_bb; // Cannot move to friendly occupied square
            for to_sq in attacks.iter() {
                let is_capture = self.occupied_bb.is_set(to_sq); // guaranteed to be opponent because of the above condition
                let mut mv = ChessMove::new(from_sq, to_sq, None);
                mv.is_capture = is_capture;
                moves.push(mv);
            }
        }
    }

    fn generate_king_moves(&self, moves: &mut Vec<ChessMove>, color: Color) {
        let king_bb = self.piece_bbs[PieceType::King as usize] & self.color_bbs[color as usize];
        if king_bb.is_empty() { unreachable!("No king on board!"); }
        let from_sq = king_bb.lsb().unwrap(); // King's current square

        let friendly_bb = self.color_bbs[color as usize];
        let king_normal_moves = PRECOMPUTED.king_attacks[from_sq.to_u8() as usize] & !friendly_bb;
        for to_sq in king_normal_moves.iter() {
            let is_capture = self.occupied_bb.is_set(to_sq); // guaranteed to be opponent because of the above condition
            let mut mv = ChessMove::new(from_sq, to_sq, None);
            mv.is_capture = is_capture;
            moves.push(mv);
        }

        // Castling
        // 1. King and relevant rook must not have moved (checked by castling_rights).
        // 2. Squares between king and rook must be empty.
        // 3. King must not be in check.
        // 4. King must not pass through or land on a square attacked by the opponent.
        // (Conditions 3 and 4 are for legal move generation, but we check 1 and 2 for pseudo-legal)

        let (king_side_sq, queen_side_sq_c, queen_side_sq_b) = match color {
            Color::White => (Square::G1, Square::C1, Square::B1),
            Color::Black => (Square::G8, Square::C8, Square::B8),
        };
        let (king_side_empty_mask, queen_side_empty_mask) = match color {
            Color::White => (Bitboard::from_square(Square::F1) | Bitboard::from_square(Square::G1), 
                             Bitboard::from_square(Square::D1) | Bitboard::from_square(Square::C1) | Bitboard::from_square(Square::B1)),
            Color::Black => (Bitboard::from_square(Square::F8) | Bitboard::from_square(Square::G8),
                             Bitboard::from_square(Square::D8) | Bitboard::from_square(Square::C8) | Bitboard::from_square(Square::B8)),
        };

        if self.castling_rights.can_castle_kingside(color) {
            if (self.occupied_bb & king_side_empty_mask).is_empty() {
                // For pseudo-legal, we don't check for attacks here. That's for legal move gen.
                moves.push(ChessMove::new(from_sq, king_side_sq, None)); // King moves two squares
            }
        }
        if self.castling_rights.can_castle_queenside(color) {
            if (self.occupied_bb & queen_side_empty_mask).is_empty() {
                moves.push(ChessMove::new(from_sq, queen_side_sq_c, None)); // King moves two squares
            }
        }
    }
    
    fn generate_sliding_moves(&self, moves: &mut Vec<ChessMove>, color: Color, piece_type: PieceType) {
        let pieces_bb = self.piece_bbs[piece_type as usize] & self.color_bbs[color as usize];
        let friendly_bb = self.color_bbs[color as usize];
        
        for from_sq in pieces_bb.iter() {
            // TODO: potential improvement if match outside for loop?
            let attacks = match piece_type {
                PieceType::Bishop => self.get_bishop_attacks(from_sq, self.occupied_bb),
                PieceType::Rook => self.get_rook_attacks(from_sq, self.occupied_bb),
                PieceType::Queen => self.get_bishop_attacks(from_sq, self.occupied_bb) | 
                                    self.get_rook_attacks(from_sq, self.occupied_bb),
                _ => unreachable!("Not a slider"),
            };
            
            let valid_moves = attacks & !friendly_bb; // Cannot move to friendly occupied square
            for to_sq in valid_moves.iter() {
                let is_capture = self.occupied_bb.is_set(to_sq); // guaranteed to be opponent because of the above condition
                let mut mv = ChessMove::new(from_sq, to_sq, None);
                mv.is_capture = is_capture;
                moves.push(mv);
            }
        }
    }

    // Helper: Get rook attacks (horizontal/vertical rays)
    fn get_rook_attacks(&self, sq: Square, occupied: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::EMPTY;
        let directions = [(0, 1), (0, -1), (1, 0), (-1, 0)]; // N, S, E, W (rank_offset, file_offset)
        for (dr, df) in directions.iter() {
            for i in 1..8 {
                if let Some(target_sq) = sq.try_offset(*df * i, *dr * i) {
                    attacks.set(target_sq);
                    if occupied.is_set(target_sq) { break; } // Stop ray if square is occupied
                } else {
                    break; // Off board
                }
            }
        }
        attacks
    }

    // Helper: Get bishop attacks (diagonal rays)
    fn get_bishop_attacks(&self, sq: Square, occupied: Bitboard) -> Bitboard {
        let mut attacks = Bitboard::EMPTY;
        let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)]; // NE, NW, SE, SW
        for (dr, df) in directions.iter() {
            for i in 1..8 {
                 if let Some(target_sq) = sq.try_offset(*df * i, *dr * i) {
                    attacks.set(target_sq);
                    if occupied.is_set(target_sq) { break; }
                } else {
                    break; 
                }
            }
        }
        attacks
    }

    /// Checks if a given square is attacked by the opponent.
    pub fn is_square_attacked(&self, target_sq: Square, attacker_color: Color) -> bool {
        let opponent_pieces = self.color_bbs[attacker_color as usize];
        
        // Pawn attacks
        let pawn_attacks_to_sq = PRECOMPUTED.pawn_attacks[attacker_color.opponent() as usize][target_sq.to_u8() as usize];
        if (pawn_attacks_to_sq & self.piece_bbs[PieceType::Pawn as usize] & opponent_pieces).is_not_empty() {
            return true;
        }
        
        // Knight attacks
        if (PRECOMPUTED.knight_attacks[target_sq.to_u8() as usize] & self.piece_bbs[PieceType::Knight as usize] & opponent_pieces).is_not_empty() {
            return true;
        }
        
        // King attacks
        if (PRECOMPUTED.king_attacks[target_sq.to_u8() as usize] & self.piece_bbs[PieceType::King as usize] & opponent_pieces).is_not_empty() {
            return true;
        }

        // Sliding piece attacks (Rook, Bishop, Queen)
        let rook_like_attackers = (self.piece_bbs[PieceType::Rook as usize] | self.piece_bbs[PieceType::Queen as usize]) & opponent_pieces;
        if (self.get_rook_attacks(target_sq, self.occupied_bb) & rook_like_attackers).is_not_empty() {
            return true;
        }
        
        let bishop_like_attackers = (self.piece_bbs[PieceType::Bishop as usize] | self.piece_bbs[PieceType::Queen as usize]) & opponent_pieces;
        if (self.get_bishop_attacks(target_sq, self.occupied_bb) & bishop_like_attackers).is_not_empty() {
            return true;
        }
        
        false
    }


    pub fn get_attacked_squares(&self, attacker_color: Color) -> Bitboard {
        let mut attacked_squares = Bitboard::new();
        let opponent_pieces = self.color_bbs[attacker_color as usize];

        // Pawn attacks
        let pawns = self.piece_bbs[PieceType::Pawn as usize] & opponent_pieces;
        for sq in pawns.iter() {
            attacked_squares |= PRECOMPUTED.pawn_attacks[attacker_color as usize][sq.to_u8() as usize];
        }

        // Knight attacks
        let knights = self.piece_bbs[PieceType::Knight as usize] & opponent_pieces;
        for sq in knights.iter() {
            attacked_squares |= PRECOMPUTED.knight_attacks[sq.to_u8() as usize];
        }

        // King attacks
        let king = self.piece_bbs[PieceType::King as usize] & opponent_pieces;
        // Assuming there's only one king of a given color on the board for simplicity
        if let Some(king_sq) = king.iter().next() {
            attacked_squares |= PRECOMPUTED.king_attacks[king_sq.to_u8() as usize];
        }

        // Rook and Queen attacks (rook-like)
        let rook_like_attackers = (self.piece_bbs[PieceType::Rook as usize] | self.piece_bbs[PieceType::Queen as usize]) & opponent_pieces;
        for sq in rook_like_attackers.iter() {
            attacked_squares |= self.get_rook_attacks(sq, self.occupied_bb);
        }

        // Bishop and Queen attacks (bishop-like)
        let bishop_like_attackers = (self.piece_bbs[PieceType::Bishop as usize] | self.piece_bbs[PieceType::Queen as usize]) & opponent_pieces;
        for sq in bishop_like_attackers.iter() {
            attacked_squares |= self.get_bishop_attacks(sq, self.occupied_bb);
        }

        attacked_squares
    }
}







impl Board {
    /// Makes a move on the board and returns a new board state.
    pub fn make_move(&self, mv: &ChessMove) -> Board { // TODO: make the move inplace and add option to undo
        let mut new_board = self.clone();
        let moving_piece_color = self.turn;
        let opponent_color = moving_piece_color.opponent();

        // 1. Identify the piece being moved on the original board
        let (moving_piece_type, _) = self.piece_on_square(mv.from)
            .expect("There should be a piece on the 'from' square");

        // Reset en passant square for the next turn unless it's a pawn double push
        new_board.en_passant_square = None;

        // Reset halfmove clock if it's a pawn move or a capture
        if moving_piece_type == PieceType::Pawn {
            new_board.halfmove_clock = 0;
        } else {
            new_board.halfmove_clock += 1;
        }

        // Increment fullmove number after Black moves
        if moving_piece_color == Color::Black {
            new_board.fullmove_number += 1;
        }

        // 2. Handle Captures (including En Passant)
        let captured_piece_sq = if let Some(ep_sq) = self.en_passant_square {
            // Check for En Passant Capture: Is it a pawn moving to the EP square?
            if moving_piece_type == PieceType::Pawn && mv.to == ep_sq {
                // The captured pawn is on the rank of the EP square, same file as the EP square
                let captured_pawn_sq = match moving_piece_color {
                    Color::White => ep_sq.try_offset(0, -1).unwrap(), // Captured black pawn is one rank below EP square
                    Color::Black => ep_sq.try_offset(0, 1).unwrap(),  // Captured white pawn is one rank above EP square
                };
                let captured_pawn_type = PieceType::Pawn;
                
                // Remove the captured pawn from the new board
                new_board.piece_bbs[captured_pawn_type as usize].clear(captured_pawn_sq);
                new_board.color_bbs[opponent_color as usize].clear(captured_pawn_sq);
                
                new_board.halfmove_clock = 0; // EP capture resets halfmove clock
                Some(captured_pawn_sq) // Indicate a piece was captured (for occupied_bb update)
            } else {
                // Not an en passant move, check for standard capture at destination
                if self.occupied_bb.is_set(mv.to) {
                    if let Some((captured_pt, captured_color)) = self.piece_on_square(mv.to) {
                         // Standard capture
                         new_board.piece_bbs[captured_pt as usize].clear(mv.to);
                         new_board.color_bbs[captured_color as usize].clear(mv.to);
                         new_board.halfmove_clock = 0; // Capture resets halfmove clock
                         Some(mv.to) // Indicate a piece was captured
                    } else {
                         None // Should not happen if occupied_bb is correct
                    }
                } else {
                    None // No capture
                }
            }
        } else {
            // No en passant square was set, check for standard capture at destination
             if self.occupied_bb.is_set(mv.to) {
                if let Some((captured_pt, captured_color)) = self.piece_on_square(mv.to) {
                     // Standard capture
                     new_board.piece_bbs[captured_pt as usize].clear(mv.to);
                     new_board.color_bbs[captured_color as usize].clear(mv.to);
                     new_board.halfmove_clock = 0; // Capture resets halfmove clock
                     Some(mv.to) // Indicate a piece was captured
                } else {
                     None // Should not happen if occupied_bb is correct
                }
            } else {
                None // No capture
            }
        };
        
        // 3. Move the piece
        // Remove from 'from' square
        new_board.piece_bbs[moving_piece_type as usize].clear(mv.from);
        new_board.color_bbs[moving_piece_color as usize].clear(mv.from);
        // Add to 'to' square
        new_board.piece_bbs[moving_piece_type as usize].set(mv.to);
        new_board.color_bbs[moving_piece_color as usize].set(mv.to);

        // 4. Handle Special Moves (Promotion, Castling)

        // Promotion
        if let Some(promoted_piece_type) = mv.promotion {
            // Remove the pawn that arrived at the promotion square
            new_board.piece_bbs[PieceType::Pawn as usize].clear(mv.to);
            // Add the promoted piece
            new_board.piece_bbs[promoted_piece_type as usize].set(mv.to);
        }

        // Castling (handled by checking King move of 2 squares horizontally)
        if moving_piece_type == PieceType::King && (mv.to.file() as i8 - mv.from.file() as i8).abs() == 2 {
            // This is a castling move, move the corresponding rook
            match (moving_piece_color, mv.to) {
                (Color::White, Square::G1) => { // White Kingside Castle
                    let rook_from = Square::H1;
                    let rook_to = Square::F1;
                    new_board.piece_bbs[PieceType::Rook as usize].clear(rook_from);
                    new_board.color_bbs[Color::White as usize].clear(rook_from);
                    new_board.piece_bbs[PieceType::Rook as usize].set(rook_to);
                    new_board.color_bbs[Color::White as usize].set(rook_to);
                },
                (Color::White, Square::C1) => { // White Queenside Castle
                     let rook_from = Square::A1;
                    let rook_to = Square::D1;
                    new_board.piece_bbs[PieceType::Rook as usize].clear(rook_from);
                    new_board.color_bbs[Color::White as usize].clear(rook_from);
                    new_board.piece_bbs[PieceType::Rook as usize].set(rook_to);
                    new_board.color_bbs[Color::White as usize].set(rook_to);
                },
                (Color::Black, Square::G8) => { // Black Kingside Castle
                     let rook_from = Square::H8;
                    let rook_to = Square::F8;
                    new_board.piece_bbs[PieceType::Rook as usize].clear(rook_from);
                    new_board.color_bbs[Color::Black as usize].clear(rook_from);
                    new_board.piece_bbs[PieceType::Rook as usize].set(rook_to);
                    new_board.color_bbs[Color::Black as usize].set(rook_to);
                },
                (Color::Black, Square::C8) => { // Black Queenside Castle
                     let rook_from = Square::A8;
                    let rook_to = Square::D8;
                    new_board.piece_bbs[PieceType::Rook as usize].clear(rook_from);
                    new_board.color_bbs[Color::Black as usize].clear(rook_from);
                    new_board.piece_bbs[PieceType::Rook as usize].set(rook_to);
                    new_board.color_bbs[Color::Black as usize].set(rook_to);
                },
                _ => {}, // Not a castling destination for a king move of 2 squares
            }
        }

        // 5. Update Castling Rights
        // Moving the king removes castling rights for that color
        if moving_piece_type == PieceType::King {
            match moving_piece_color {
                Color::White => {
                    new_board.castling_rights.remove_right(CastlingRights::WHITE_KINGSIDE);
                    new_board.castling_rights.remove_right(CastlingRights::WHITE_QUEENSIDE);
                },
                Color::Black => {
                    new_board.castling_rights.remove_right(CastlingRights::BLACK_KINGSIDE);
                    new_board.castling_rights.remove_right(CastlingRights::BLACK_QUEENSIDE);
                },
            }
        }
        // Moving a rook from its starting square removes that side's castling right
        if moving_piece_type == PieceType::Rook {
            match mv.from {
                Square::A1 => new_board.castling_rights.remove_right(CastlingRights::WHITE_QUEENSIDE),
                Square::H1 => new_board.castling_rights.remove_right(CastlingRights::WHITE_KINGSIDE),
                Square::A8 => new_board.castling_rights.remove_right(CastlingRights::BLACK_QUEENSIDE),
                Square::H8 => new_board.castling_rights.remove_right(CastlingRights::BLACK_KINGSIDE),
                _ => {}, // Not a starting rook square
            }
        }
        // Capturing a rook on its starting square removes the opponent's castling right
        if let Some(captured_sq) = captured_piece_sq {
            if let Some((captured_pt, captured_color)) = self.piece_on_square(captured_sq) {
                if captured_pt == PieceType::Rook {
                    match (captured_color, captured_sq) {
                        (Color::White, Square::A1) => new_board.castling_rights.remove_right(CastlingRights::WHITE_QUEENSIDE),
                        (Color::White, Square::H1) => new_board.castling_rights.remove_right(CastlingRights::WHITE_KINGSIDE),
                        (Color::Black, Square::A8) => new_board.castling_rights.remove_right(CastlingRights::BLACK_QUEENSIDE),
                        (Color::Black, Square::H8) => new_board.castling_rights.remove_right(CastlingRights::BLACK_KINGSIDE),
                        _ => {}, // Not a starting rook square
                    }
                }
            }
        }


        // 6. Set En Passant Square for the next turn if it was a pawn double push
        if moving_piece_type == PieceType::Pawn && (mv.from.rank() as i8 - mv.to.rank() as i8).abs() == 2 {
            let ep_rank = match moving_piece_color {
                Color::White => 2, // White pawn moved from rank 1 to 3, EP target is rank 2
                Color::Black => 5, // Black pawn moved from rank 6 to 4, EP target is rank 5
            };
            // The EP target square is on the file of the destination square, at the EP rank
            new_board.en_passant_square = Some(Square::from_file_rank(mv.to.file(), ep_rank));
        }


        // 7. Switch Turn
        new_board.turn = opponent_color;

        // 8. Update Occupied Bitboard
        new_board.update_occupied_bb();

        new_board
    }

    /// Generates all fully legal moves for the current player.
    #[cfg_attr(feature = "tracy", tracing::instrument(skip_all))]
    pub fn generate_legal_moves(&self, pseudo_legal_moves: &mut Vec<ChessMove>, legal_moves: &mut Vec<ChessMove>) {
        pseudo_legal_moves.clear();
        legal_moves.clear();

        self.generate_pseudo_legal_moves(pseudo_legal_moves);
        let current_player_color = self.turn;

        for mv in pseudo_legal_moves.iter() {
            // Special handling for castling: check squares king passes through
            // This is a bit tricky as pseudo-legal castling only checks empty squares, not attacks.
            let is_castle_move = self.piece_bbs[PieceType::King as usize].is_set(mv.from) &&
                                 (mv.to.file() as i8 - mv.from.file() as i8).abs() == 2;

            if is_castle_move {
                let king_start_sq = mv.from;
                let king_mid_sq = if mv.to.file() > king_start_sq.file() { // Kingside
                    king_start_sq.try_offset(1,0).unwrap()
                } else { // Queenside
                    king_start_sq.try_offset(-1,0).unwrap()
                };

                // King cannot be in check initially, or pass through an attacked square
                if self.is_square_attacked(king_start_sq, current_player_color.opponent()) ||
                   self.is_square_attacked(king_mid_sq, current_player_color.opponent()) {
                    continue; // Invalid castle
                }
            }

            let board_after_move = self.make_move(mv);
            
            // Find the king of the player who just moved
            let king_bb_after_move = board_after_move.piece_bbs[PieceType::King as usize] & board_after_move.color_bbs[current_player_color as usize];
            
            if king_bb_after_move.is_empty() {
                // This should ideally not happen if make_move is correct and king is always on board
                // Consider this an error or an impossible position.
                // For robustness, one might skip this move or panic.
                // If a move leads to no king, it's an invalid state.
                continue;
            }
            let king_sq_after_move = king_bb_after_move.lsb().unwrap();

            if !board_after_move.is_square_attacked(king_sq_after_move, board_after_move.turn) { // board_after_move.turn is opponent
                legal_moves.push(*mv);
            }
        }
    }
}

