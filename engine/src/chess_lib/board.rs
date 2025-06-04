pub use crate::bitboard::*;
use crate::chess_lib::GameState;
pub use crate::uci_parser::*;

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    /// Returns the opponent's color.
    #[inline]
    pub fn opponent(self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum PieceType {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

impl PieceType {
    pub const ALL: [PieceType; 6] = [
        PieceType::Pawn, PieceType::Knight, PieceType::Bishop,
        PieceType::Rook, PieceType::Queen, PieceType::King
    ];
    pub const PROMOTION_PIECES: [PieceType; 4] = [
        PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight
    ];

    pub fn to_char(&self) -> char {
        match self {
            PieceType::Pawn => 'P',
            PieceType::Knight => 'N',
            PieceType::Bishop => 'B',
            PieceType::Rook => 'R',
            PieceType::Queen => 'Q',
            PieceType::King => 'K',
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CastlingRights(u8);

impl CastlingRights {
    pub const NONE: Self = Self(0);
    pub const WHITE_KINGSIDE: u8 = 1 << 0;
    pub const WHITE_QUEENSIDE: u8 = 1 << 1;
    pub const BLACK_KINGSIDE: u8 = 1 << 2;
    pub const BLACK_QUEENSIDE: u8 = 1 << 3;

    #[inline]
    pub fn new_all() -> Self {
        Self(Self::WHITE_KINGSIDE | Self::WHITE_QUEENSIDE | Self::BLACK_KINGSIDE | Self::BLACK_QUEENSIDE)
    }

    #[inline]
    pub fn has_right(&self, right: u8) -> bool {
        (self.0 & right) != 0
    }

    #[inline]
    pub fn remove_right(&mut self, right: u8) {
        self.0 &= !right;
    }

    #[inline]
    pub fn add_right(&mut self, right: u8) {
        self.0 |= right;
    }
    
    #[inline]
    pub fn can_castle_kingside(&self, color: Color) -> bool {
        match color {
            Color::White => self.has_right(Self::WHITE_KINGSIDE),
            Color::Black => self.has_right(Self::BLACK_KINGSIDE),
        }
    }

    #[inline]
    pub fn can_castle_queenside(&self, color: Color) -> bool {
        match color {
            Color::White => self.has_right(Self::WHITE_QUEENSIDE),
            Color::Black => self.has_right(Self::BLACK_QUEENSIDE),
        }
    }
}


impl fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut fen_string = String::new();

        if self.has_right(CastlingRights::WHITE_KINGSIDE) {
            fen_string.push('K');
        }
        if self.has_right(CastlingRights::WHITE_QUEENSIDE) {
            fen_string.push('Q');
        }
        if self.has_right(CastlingRights::BLACK_KINGSIDE) {
            fen_string.push('k');
        }
        if self.has_right(CastlingRights::BLACK_QUEENSIDE) {
            fen_string.push('q');
        }

        if fen_string.is_empty() {
            write!(f, "-")
        } else {
            write!(f, "{}", fen_string)
        }
    }
}


// 6 + 6 + 3 + 1 bits
type ChessMoveRepr = u16; // for future expansion

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ChessMove(ChessMoveRepr);

impl ChessMove {
    // Bit positions and masks
    const FROM_SHIFT: ChessMoveRepr = 0;
    const TO_SHIFT: ChessMoveRepr = 6;
    const PROMO_SHIFT: ChessMoveRepr = 12;
    const CAPTURE_SHIFT: ChessMoveRepr = 15;

    const MASK_6BITS: ChessMoveRepr = 0b11_1111;
    const MASK_3BITS: ChessMoveRepr = 0b111;
    const MASK_1BIT: ChessMoveRepr = 0b1;

    const PROMOTION_NONE_VALUE: ChessMoveRepr = 0b111;

    #[inline]
    pub fn new(from: Square, to: Square, promotion: Option<PieceType>) -> Self {
        let from_bits = (from as ChessMoveRepr) << Self::FROM_SHIFT;
        let to_bits = (to as ChessMoveRepr) << Self::TO_SHIFT;

        let promo_bits = promotion
            .map(|p| (p as ChessMoveRepr))
            .unwrap_or(Self::PROMOTION_NONE_VALUE) << Self::PROMO_SHIFT;

        let capture_bit = 0 << Self::CAPTURE_SHIFT;

        ChessMove(from_bits | to_bits | promo_bits | capture_bit)
    }

    #[inline]
    pub fn new_capture(from: Square, to: Square, promotion: Option<PieceType>) -> Self {
        let from_bits = (from as ChessMoveRepr) << Self::FROM_SHIFT;
        let to_bits = (to as ChessMoveRepr) << Self::TO_SHIFT;

        let promo_bits = promotion
            .map(|p| (p as ChessMoveRepr))
            .unwrap_or(Self::PROMOTION_NONE_VALUE) << Self::PROMO_SHIFT;

        let capture_bit = 1 << Self::CAPTURE_SHIFT;

        ChessMove(from_bits | to_bits | promo_bits | capture_bit)
    }

    #[inline]
    pub fn from(&self) -> Square {
        Square::from_u8(((self.0 >> Self::FROM_SHIFT) & Self::MASK_6BITS) as u8)
    }

    #[inline]
    pub fn to(&self) -> Square {
        Square::from_u8(((self.0 >> Self::TO_SHIFT) & Self::MASK_6BITS) as u8)
    }

    #[inline]
    pub fn promotion(&self) -> Option<PieceType> {
        let promo = (self.0 >> Self::PROMO_SHIFT) & Self::MASK_3BITS;
        if promo == Self::PROMOTION_NONE_VALUE {
            None
        } else {
            Some(unsafe { *PieceType::ALL.get_unchecked(promo as usize) })
        }
    }

    #[inline]
    pub fn is_capture(&self) -> bool {
        ((self.0 >> Self::CAPTURE_SHIFT) & Self::MASK_1BIT) != 0
    }

    #[inline]
    pub fn set_capture(&mut self, is_capture: bool) {
        let bit = (is_capture as ChessMoveRepr) << Self::CAPTURE_SHIFT;
        self.0 = (self.0 & !(1 << Self::CAPTURE_SHIFT)) | bit;
    }
}

impl ChessMove {
    pub fn to_uci(&self) -> String {
        let mut s = format!("{}{}", self.from().to_algebraic(), self.to().to_algebraic());
        if let Some(promo) = self.promotion() {
            s.push(match promo {
                PieceType::Knight => 'n',
                PieceType::Bishop => 'b',
                PieceType::Rook => 'r',
                PieceType::Queen => 'q',
                _ => ' ',
            });
        }
        s
    }
}



#[derive(Debug, PartialEq, Eq)]
pub enum ParseMoveError {
    InvalidFormat,
    InvalidSquare,
    InvalidPromotionPiece,
}

/// Parses a move string in algebraic notation (e.g., "e2e4", "a7a8q").
/// Returns a ChessMove or a ParseMoveError if the string is invalid.
pub fn parse_move_string(move_str: &str) -> Result<ChessMove, ParseMoveError> {
    // A move string should be 4 or 5 characters long
    if move_str.len() != 4 && move_str.len() != 5 {
        return Err(ParseMoveError::InvalidFormat);
    }

    // Extract the 'from' square part (first 2 characters)
    let from_str = &move_str[0..2];
    let from_square = Square::from_algebraic(from_str)
        .ok_or(ParseMoveError::InvalidSquare)?;

    // Extract the 'to' square part (next 2 characters)
    let to_str = &move_str[2..4];
    let to_square = Square::from_algebraic(to_str)
        .ok_or(ParseMoveError::InvalidSquare)?;

    // Check for promotion (if the string is 5 characters long)
    let promotion = if move_str.len() == 5 {
        let promotion_char = move_str.chars().nth(4).ok_or(ParseMoveError::InvalidFormat)?;
        match promotion_char {
            'n' => Some(PieceType::Knight),
            'b' => Some(PieceType::Bishop),
            'r' => Some(PieceType::Rook),
            'q' => Some(PieceType::Queen),
            _ => return Err(ParseMoveError::InvalidPromotionPiece),
        }
    } else {
        None
    };

    Ok(ChessMove::new(from_square, to_square, promotion))
}








/// Contains all precomputed data, like attack tables.
pub struct PrecomputedData {
    pub knight_attacks: [Bitboard; 64],
    pub king_attacks: [Bitboard; 64],
    pub pawn_attacks: [[Bitboard; 64]; 2], // [Color][Square]
    pub file_masks: [Bitboard; 8],
    pub rank_masks: [Bitboard; 8],
    pub clear_file_masks: [Bitboard; 8], // Masks to clear a file (everything but the file)
    pub rank_2_bb: Bitboard,
    pub rank_7_bb: Bitboard,
}

// Lazy static initialization for precomputed data
pub static PRECOMPUTED: once_cell::sync::Lazy<PrecomputedData> = once_cell::sync::Lazy::new(PrecomputedData::new);

impl PrecomputedData {
    fn new() -> Self {
        let mut knight_attacks = [Bitboard::EMPTY; 64];
        let mut king_attacks = [Bitboard::EMPTY; 64];
        let mut pawn_attacks = [[Bitboard::EMPTY; 64]; 2];
        let mut file_masks = [Bitboard::EMPTY; 8];
        let mut rank_masks = [Bitboard::EMPTY; 8];
        let mut clear_file_masks = [Bitboard::FULL; 8];


        for i in 0..8 {
            file_masks[i] = Bitboard(0x0101010101010101u64 << i);
            clear_file_masks[i] = !file_masks[i];
            rank_masks[i] = Bitboard(0xFFu64 << (i * 8));
        }
        
        let rank_2_bb = rank_masks[1];
        let rank_7_bb = rank_masks[6];


        for sq_idx in 0..64 {
            let sq = Square::from_u8(sq_idx);
            knight_attacks[sq_idx as usize] = Self::generate_knight_attacks(sq);
            king_attacks[sq_idx as usize] = Self::generate_king_attacks(sq);
            pawn_attacks[Color::White as usize][sq_idx as usize] = Self::generate_pawn_attacks(sq, Color::White);
            pawn_attacks[Color::Black as usize][sq_idx as usize] = Self::generate_pawn_attacks(sq, Color::Black);
        }

        PrecomputedData {
            knight_attacks,
            king_attacks,
            pawn_attacks,
            file_masks,
            rank_masks,
            clear_file_masks,
            rank_2_bb,
            rank_7_bb,
        }
    }

    fn generate_knight_attacks(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let offsets = [
            (1, 2), (1, -2), (-1, 2), (-1, -2),
            (2, 1), (2, -1), (-2, 1), (-2, -1),
        ];
        for (df, dr) in offsets.iter() {
            if let Some(to_sq) = sq.try_offset(*df, *dr) {
                bb.set(to_sq);
            }
        }
        bb
    }

    fn generate_king_attacks(sq: Square) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let offsets = [
            (0, 1), (0, -1), (1, 0), (-1, 0),
            (1, 1), (1, -1), (-1, 1), (-1, -1),
        ];
        for (df, dr) in offsets.iter() {
            if let Some(to_sq) = sq.try_offset(*df, *dr) {
                bb.set(to_sq);
            }
        }
        bb
    }
    
    fn generate_pawn_attacks(sq: Square, color: Color) -> Bitboard {
        let mut bb = Bitboard::EMPTY;
        let rank_offset = if color == Color::White { 1 } else { -1 };
        
        if let Some(capture_left) = sq.try_offset(-1, rank_offset) {
            bb.set(capture_left);
        }
        if let Some(capture_right) = sq.try_offset(1, rank_offset) {
            bb.set(capture_right);
        }
        bb
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Board {
    /// Bitboards for each piece type [Pawn, Knight, Bishop, Rook, Queen, King].
    pub piece_bbs: [Bitboard; 6],
    /// Bitboards for each color [White, Black].
    pub color_bbs: [Bitboard; 2],
    /// Bitboard of all occupied squares.
    pub occupied_bb: Bitboard,

    pub turn: Color,
    pub castling_rights: CastlingRights,
    pub en_passant_square: Option<Square>, // Target square for en passant, if any      see: https://www.chessprogramming.org/En_passant
    pub halfmove_clock: u8, // For 50-move rule                                         see: https://www.chessprogramming.org/Halfmove_Clock
    pub fullmove_number: u16,

    // pub zobrist_hash: u64,
}

impl Board {
    pub fn new_start_pos() -> Self {
        let mut board = Self {
            piece_bbs: [Bitboard::EMPTY; 6],
            color_bbs: [Bitboard::EMPTY; 2],
            occupied_bb: Bitboard::EMPTY,
            turn: Color::White,
            castling_rights: CastlingRights::new_all(),
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1,
            // zobrist_hash: 0,
        };

        // Setup pawns                                                                                                              see: https://images.chesscomfiles.com/uploads/v1/images_users/tiny_mce/ColinStapczynski/phpa2wQPr.png
        board.piece_bbs[PieceType::Pawn as usize] = PRECOMPUTED.rank_masks[1] | PRECOMPUTED.rank_masks[6];
        // Setup knights                                                                                                            see: https://images.chesscomfiles.com/uploads/v1/images_users/tiny_mce/ColinStapczynski/php8ewDhz.png
        board.piece_bbs[PieceType::Knight as usize] = Bitboard::from_square(Square::B1) | Bitboard::from_square(Square::G1) |
                                                      Bitboard::from_square(Square::B8) | Bitboard::from_square(Square::G8);
        // Setup bishops                                                                                                            see: https://images.chesscomfiles.com/uploads/v1/images_users/tiny_mce/ColinStapczynski/phpIiYR5R.png
        board.piece_bbs[PieceType::Bishop as usize] = Bitboard::from_square(Square::C1) | Bitboard::from_square(Square::F1) |
                                                      Bitboard::from_square(Square::C8) | Bitboard::from_square(Square::F8);
        // Setup rooks                                                                                                              see: https://images.chesscomfiles.com/uploads/v1/images_users/tiny_mce/ColinStapczynski/phpjq7kSs.png
        board.piece_bbs[PieceType::Rook as usize] = Bitboard::from_square(Square::A1) | Bitboard::from_square(Square::H1) |
                                                    Bitboard::from_square(Square::A8) | Bitboard::from_square(Square::H8);
        // Setup queens                                                                                                             see: https://images.chesscomfiles.com/uploads/v1/images_users/tiny_mce/ColinStapczynski/phpb3ydM9.png
        board.piece_bbs[PieceType::Queen as usize] = Bitboard::from_square(Square::D1) | Bitboard::from_square(Square::D8);
        // Setup kings                                                                                                              see: https://images.chesscomfiles.com/uploads/v1/images_users/tiny_mce/ColinStapczynski/phpoODwO2.png
        board.piece_bbs[PieceType::King as usize] = Bitboard::from_square(Square::E1) | Bitboard::from_square(Square::E8);

        // Setup color bitboards
        board.color_bbs[Color::White as usize] = PRECOMPUTED.rank_masks[0] | PRECOMPUTED.rank_masks[1];
        board.color_bbs[Color::Black as usize] = PRECOMPUTED.rank_masks[6] | PRECOMPUTED.rank_masks[7];
        
        board.update_occupied_bb();
        board
    }

    /// Updates the main occupied bitboard from color bitboards.
    #[inline]
    pub fn update_occupied_bb(&mut self) {
        self.occupied_bb = self.color_bbs[0] | self.color_bbs[1];
    }
    
    /// Gets the bitboard of pieces for the current player.
    #[inline]
    pub fn current_player_pieces_bb(&self) -> Bitboard {
        self.color_bbs[self.turn as usize]
    }

    /// Gets the bitboard of pieces for the opponent.
    #[inline]
    pub fn opponent_pieces_bb(&self) -> Bitboard {
        self.color_bbs[self.turn.opponent() as usize]
    }
    
    /// Gets the bitboard of empty squares.
    #[inline]
    pub fn empty_squares_bb(&self) -> Bitboard {
        !self.occupied_bb
    }

    /// Gets the piece type and color on a given square.
    pub fn piece_on_square(&self, sq: Square) -> Option<(PieceType, Color)> {
        if !self.occupied_bb.is_set(sq) {
            return None;
        }
        for pt_idx in 0..6 {
            if self.piece_bbs[pt_idx].is_set(sq) {
                let color = if self.color_bbs[Color::White as usize].is_set(sq) {
                    Color::White
                } else {
                    Color::Black
                };
                let pt = PieceType::ALL[pt_idx];
                return Some((pt, color));
            }
        }
        None
        // unreachable!("Should not happen if occupied_bb is correct"); // FIXME?
    }

    pub fn piece_type_on_square(&self, sq: Square) -> Option<PieceType> {
        if !self.occupied_bb.is_set(sq) {
            return None;
        }
        for pt_idx in 0..6 {
            if self.piece_bbs[pt_idx].is_set(sq) {
                return Some(PieceType::ALL[pt_idx]);
            }
        }
        None
        // unreachable!("Should not happen if occupied_bb is correct"); // FIXME?
    }
}




impl Board {
    fn new_empty() -> Self {
        Self {
            piece_bbs: [Bitboard::EMPTY; 6],
            color_bbs: [Bitboard::EMPTY; 2],
            occupied_bb: Bitboard::EMPTY,
            turn: Color::White, // Default, will be overwritten by FEN
            castling_rights: CastlingRights::default(),
            en_passant_square: None,
            halfmove_clock: 0,
            fullmove_number: 1, // Default, will be overwritten
            // zobrist_hash: 0,
        }
    }

    /// Creates a new board from a FEN (Forsyth-Edwards Notation) structure.
    pub fn from_fen(fen: &Fen) -> Result<Self, FenParseError> {
        let mut board = Self::new_empty();

        // 1. Piece Placement
        let ranks: Vec<&str> = fen.piece_placement.split('/').collect();
        if ranks.len() != 8 {
            return Err(FenParseError::InvalidRankCount);
        }

        for (rank_idx_fen, rank_str) in ranks.iter().enumerate() {
            let board_rank = 7 - rank_idx_fen as u8; // FEN ranks are 8..1, board ranks are 0..7
            let mut file_idx: u8 = 0;
            for piece_char in rank_str.chars() {
                if file_idx >= 8 {
                    return Err(FenParseError::InvalidFileCountInRank(rank_idx_fen));
                }
                if let Some(skip) = piece_char.to_digit(10) {
                    file_idx += skip as u8;
                } else {
                    let color = if piece_char.is_uppercase() { Color::White } else { Color::Black };
                    let piece_type = match piece_char.to_ascii_lowercase() {
                        'p' => PieceType::Pawn,
                        'n' => PieceType::Knight,
                        'b' => PieceType::Bishop,
                        'r' => PieceType::Rook,
                        'q' => PieceType::Queen,
                        'k' => PieceType::King,
                        _ => return Err(FenParseError::UnknownPieceChar(piece_char)),
                    };
                    let sq = Square::from_file_rank(file_idx, board_rank);
                    board.piece_bbs[piece_type as usize].set(sq);
                    board.color_bbs[color as usize].set(sq);
                    file_idx += 1;
                }
            }
            if file_idx != 8 {
                 return Err(FenParseError::InvalidFileCountInRank(rank_idx_fen));
            }
        }
        board.update_occupied_bb();

        // 2. Active Color
        board.turn = match fen.active_color {
            'w' => Color::White,
            'b' => Color::Black,
            _ => return Err(FenParseError::InvalidActiveColor(fen.active_color)),
        };

        // 3. Castling Availability
        board.castling_rights = CastlingRights::NONE;
        if fen.castling_availability != "-" {
            for char_code in fen.castling_availability.chars() {
                match char_code {
                    'K' => board.castling_rights.add_right(CastlingRights::WHITE_KINGSIDE),
                    'Q' => board.castling_rights.add_right(CastlingRights::WHITE_QUEENSIDE),
                    'k' => board.castling_rights.add_right(CastlingRights::BLACK_KINGSIDE),
                    'q' => board.castling_rights.add_right(CastlingRights::BLACK_QUEENSIDE),
                    _ => return Err(FenParseError::InvalidCastlingString(fen.castling_availability.clone())),
                }
            }
        }
        
        // 4. En Passant Target
        if fen.en_passant_target == "-" {
            board.en_passant_square = None;
        } else {
            board.en_passant_square = Square::from_algebraic(&fen.en_passant_target);
            if board.en_passant_square == None && fen.en_passant_target != "-" {
                return Err(FenParseError::InvalidEnPassantTarget(fen.en_passant_target.clone()));
            }
        }

        // 5. Halfmove Clock
        board.halfmove_clock = fen.halfmove_clock;

        // 6. Fullmove Number
        board.fullmove_number = fen.fullmove_number;
        if fen.fullmove_number == 0 { // Fullmove number must be at least 1
            return Err(FenParseError::InvalidFullmoveNumber("Fullmove number is 0".parse::<i32>().unwrap_err()));
        }

        Ok(board)
    }


    pub fn new_start_pos_from_fen() -> Self {
        let start_fen = Fen {
            piece_placement: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".to_string(),
            active_color: 'w',
            castling_availability: "KQkq".to_string(),
            en_passant_target: "-".to_string(),
            halfmove_clock: 0,
            fullmove_number: 1,
        };
        Self::from_fen(&start_fen).expect("Standard FEN should always parse correctly")
    }
}


impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() { // Iterate ranks from 8 down to 1
            write!(f, "{} ", rank + 1)?; // Print rank number

            for file in 0..8 { // Iterate files from 'a' to 'h'
                let square = Square::from_file_rank(file, rank);
                match self.piece_on_square(square) {
                    Some((piece_type, color)) => {
                        let piece_char = match (piece_type, color) {
                            (PieceType::Rook, Color::White) => 'R',
                            (PieceType::Knight, Color::White) => 'N',
                            (PieceType::Bishop, Color::White) => 'B',
                            (PieceType::Queen, Color::White) => 'Q',
                            (PieceType::King, Color::White) => 'K',
                            (PieceType::Pawn, Color::White) => 'P',
                            (PieceType::Rook, Color::Black) => 'r',
                            (PieceType::Knight, Color::Black) => 'n',
                            (PieceType::Bishop, Color::Black) => 'b',
                            (PieceType::Queen, Color::Black) => 'q',
                            (PieceType::King, Color::Black) => 'k',
                            (PieceType::Pawn, Color::Black) => 'p',
                        };
                        // write!(f, "{} ", piece_char)?;
                        match color {
                            Color::Black => write!(f, "\x1b[1;34m{}\x1b[0m ", piece_char)?,
                            Color::White => write!(f, "\x1b[1;31m{}\x1b[0m ", piece_char)?,
                        }
                    }
                    None => {
                        write!(f, ". ")?; // Represent empty squares
                    }
                }
            }
            writeln!(f)?; // Newline after each rank
        }
        writeln!(f, "  a b c d e f g h")?; // Print file letters

        // Optionally print other board state information
        writeln!(f, "Turn: {:?}", self.turn)?;
        writeln!(f, "Castling Rights: {}", self.castling_rights)?;
        writeln!(f, "En Passant: {:?}", self.en_passant_square)?;
        writeln!(f, "Halfmove Clock: {}", self.halfmove_clock)?;
        writeln!(f, "Fullmove Number: {}", self.fullmove_number)?;

        Ok(())
    }
}








use rand::{RngCore, SeedableRng};
use rand::rngs::StdRng;
use once_cell::sync::Lazy;

pub struct ZobristHashes {
    /// Hashes for each piece type, color, and square.
    /// Indexed as `piece_square_hashes[PieceType as usize][Color as usize][Square as usize]`.
    piece_square_hashes: [[[u64; 64]; 2]; 6],
    /// Hashes for the current turn (White or Black).
    /// Indexed as `turn_hash[Color as usize]`.
    turn_hash: [u64; 2],
    /// Hashes for all 16 possible castling rights combinations (since `CastlingRights` is a `u8` bitmask).
    /// Indexed by the `u8` value of `CastlingRights`.
    castling_hashes: [u64; 16],
    /// Hashes for each possible en passant square (0-63).
    /// Indexed as `en_passant_hashes[Square as usize]`.
    en_passant_hashes: [u64; 64],
}

impl ZobristHashes {
    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(27);

        let mut hashes = ZobristHashes {
            piece_square_hashes: [[[0; 64]; 2]; 6],
            turn_hash: [0; 2],
            castling_hashes: [0; 16],
            en_passant_hashes: [0; 64],
        };

        for pt_idx in 0..6 {
            for color_idx in 0..2 {
                for sq_idx in 0..64 {
                    hashes.piece_square_hashes[pt_idx][color_idx][sq_idx] = rng.next_u64();
                }
            }
        }

        hashes.turn_hash[Color::White as usize] = rng.next_u64();
        hashes.turn_hash[Color::Black as usize] = rng.next_u64();

        for i in 0..16 {
            hashes.castling_hashes[i] = rng.next_u64();
        }

        for i in 0..64 {
            hashes.en_passant_hashes[i] = rng.next_u64();
        }

        hashes
    }
}

static ZOBRIST_HASHES: Lazy<ZobristHashes> = Lazy::new(|| ZobristHashes::new());

impl Board {
    pub fn compute_zobrist_hash(&self) -> u64 {
        let zobrist_hashes = &ZOBRIST_HASHES;
        let mut hash = 0u64;

        // 1. Hash pieces on squares
        for pt_idx in 0..6 { // PieceType (e.g., Pawn, Knight, Bishop, Rook, Queen, King)
            for color_idx in 0..2 { // Color (White, Black)
                let piece_type_bb = self.piece_bbs[pt_idx];
                let color_bb = self.color_bbs[color_idx];
                let specific_piece_bb = piece_type_bb & color_bb; // Get bitboard for this specific piece type and color

                // Iterate over set bits in the combined bitboard
                for sq_idx in specific_piece_bb.iter() {
                    hash ^= zobrist_hashes.piece_square_hashes[pt_idx][color_idx][sq_idx as usize];
                }
            }
        }

        // 2. Hash the current turn
        hash ^= zobrist_hashes.turn_hash[self.turn as usize];

        // 3. Hash the castling rights
        // The `CastlingRights` struct's inner `u8` value directly represents the combination.
        hash ^= zobrist_hashes.castling_hashes[self.castling_rights.0 as usize];
        // NOTE: this might need per component hashes?

        // 4. Hash the en passant square, if one exists
        if let Some(sq) = self.en_passant_square {
            hash ^= zobrist_hashes.en_passant_hashes[sq as usize];
        }

        // EDGE CASE?????
        // if let Some(sq) = self.en_passant_square {
        //     if self.can_capture_en_passant(sq) {
        //         hash ^= zobrist_hashes.en_passant_hashes[sq as usize];
        //     }
        // }

        hash
    }

    // pub fn get_zobrist_hash(&mut self) -> u64 {
    //     if self.zobrist_hash == 0 {
    //         self.zobrist_hash = self.compute_zobrist_hash(); // TODO: just compute the hash for the initial startpos and then the rest are only updates
    //     }
    //     self.zobrist_hash
    // }
}



impl Board {
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();

        // Generate piece placement from rank 8 to 1
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                let sq_index = rank * 8 + file;
                let mut found_piece = false;

                for (i, bb) in self.piece_bbs.iter().enumerate() {
                    if (bb.0 >> sq_index) & 1 == 1 {
                        if empty > 0 {
                            fen.push_str(&empty.to_string());
                            empty = 0;
                        }

                        let piece_char = match i {
                            0 => 'P',
                            1 => 'N',
                            2 => 'B',
                            3 => 'R',
                            4 => 'Q',
                            5 => 'K',
                            _ => unreachable!(),
                        };

                        let is_black = (self.color_bbs[1].0 >> sq_index) & 1 == 1;
                        fen.push(if is_black { piece_char.to_ascii_lowercase() } else { piece_char });
                        found_piece = true;
                        break;
                    }
                }

                if !found_piece {
                    empty += 1;
                }
            }

            if empty > 0 {
                fen.push_str(&empty.to_string());
            }

            if rank != 0 {
                fen.push('/');
            }
        }

        // Active color
        fen.push(' ');
        fen.push(match self.turn {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling rights
        fen.push(' ');
        fen.push_str(&self.castling_rights.to_string());

        // En passant square
        fen.push(' ');
        fen.push_str(match self.en_passant_square {
            Some(ref sq) => sq.to_algebraic(),
            None => "-".to_string(),
        }
        .as_str());

        // Halfmove clock and fullmove number
        fen.push(' ');
        fen.push_str(&self.halfmove_clock.to_string());
        fen.push(' ');
        fen.push_str(&self.fullmove_number.to_string());

        fen
    }

    
    pub fn to_pgn(&self) -> String {
        let fen = self.to_fen();
        format!(
            "[FEN \"{}\"]\n[SetUp \"1\"]\n\n*",
            fen
        )
    }
}

use std::collections::HashMap;

impl Board {
    pub fn is_capture(&self, mv: &ChessMove) -> bool {
        let to_mask = 1u64 << mv.to() as usize;
        let opponent_color = self.turn.opponent() as usize;
        (self.color_bbs[opponent_color].0 & to_mask) != 0
    }

    pub fn is_check(&self) -> bool {
        self.find_king_square(self.turn).map_or(false, |sq| self.is_square_attacked(sq, self.turn.opponent()))
    }
}

impl Board {
    pub fn to_san(&self, mv: &ChessMove, pseudo_legal_moves: &mut Vec<ChessMove>, legal_moves: &mut Vec<ChessMove>, board_repetition_counts: &mut HashMap<u64, u8>) -> String {
        // Clone board to simulate the move
        let mut board = self.clone();
        let from = mv.from();
        let to = mv.to();
        let piece = board.piece_type_on_square(from).expect("No piece on from-square");

        // Handle castling
        if piece == PieceType::King {
            let from_file = from.file();
            let to_file = to.file();
            if from_file == 4 && (to_file == 6 || to_file == 2) {
                return if to_file == 6 { "O-O".to_string() } else { "O-O-O".to_string() };
            }
        }

        let is_capture = board.is_capture(mv);
        // let is_capture = mv.is_capture();
        let mut san = String::new();

        // Piece letter (empty for pawns)
        if piece != PieceType::Pawn {
            san.push(piece.to_char());
        }

        board.generate_legal_moves(pseudo_legal_moves, legal_moves);

        // Disambiguation
        let ambiguous = pseudo_legal_moves
            .into_iter()
            .filter(|m| m.to() == mv.to() && *m != mv && board.piece_type_on_square(m.from()) == Some(piece))
            .collect::<Vec<_>>();
        if !ambiguous.is_empty() {
            let from_file = from.file();
            let from_rank = from.rank();

            let file_needed = ambiguous.iter().any(|m| m.from().file() != from_file);
            let rank_needed = ambiguous.iter().any(|m| m.from().rank() != from_rank);

            if file_needed {
                san.push((b'a' + from_file) as char);
            }
            if rank_needed || !file_needed {
                san.push((b'1' + from_rank) as char);
            }
        }

        // Capture
        if is_capture {
            if piece == PieceType::Pawn {
                san.push((b'a' + from.file()) as char);
            }
            san.push('x');
        }

        // Destination
        san.push_str(&to.to_algebraic());

        // Promotion
        if let Some(promo) = mv.promotion() {
            san.push('=');
            san.push(promo.to_char());
        }

        // Simulate move to check for check or mate
        board = board.make_move(mv);

        let board_repetition_count = *board_repetition_counts.entry(board.compute_zobrist_hash()).or_insert(0) + 1;

        board.generate_legal_moves(pseudo_legal_moves, legal_moves);

        let game_state = board.check_game_state(legal_moves.is_empty(), board_repetition_count);

        if let GameState::Checkmate(_) = game_state {
            san.push('#');
        } else if board.is_check() {
            san.push('+');
        }

        san
    }
}
