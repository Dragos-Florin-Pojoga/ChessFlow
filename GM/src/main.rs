use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc::{self, TryRecvError};
use std::fmt;

// Add rand crate for bot move selection
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    White,
    Black,
}

impl Player {
    fn opponent(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PieceType {
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Piece {
    ptype: PieceType,
    color: Player,
}

impl Piece {
    fn to_char(&self) -> char {
        match self.color {
            Player::White => match self.ptype {
                PieceType::King => 'K',
                PieceType::Queen => 'Q',
                PieceType::Rook => 'R',
                PieceType::Bishop => 'B',
                PieceType::Knight => 'N',
                PieceType::Pawn => 'P',
            },
            Player::Black => match self.ptype {
                PieceType::King => 'k',
                PieceType::Queen => 'q',
                PieceType::Rook => 'r',
                PieceType::Bishop => 'b',
                PieceType::Knight => 'n',
                PieceType::Pawn => 'p',
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CastlingRights {
    white_king_side: bool,
    white_queen_side: bool,
    black_king_side: bool,
    black_queen_side: bool,
}

impl CastlingRights {
    fn new() -> Self {
        CastlingRights {
            white_king_side: true,
            white_queen_side: true,
            black_king_side: true,
            black_queen_side: true,
        }
    }

    // Update rights based on piece movement or captures affecting king/rooks.
    fn update_on_move(&mut self, from: (usize, usize), to: (usize, usize), piece: Piece) {
        if piece.ptype == PieceType::King && piece.color == Player::White && from == (7, 4) {
            self.white_king_side = false;
            self.white_queen_side = false;
        }
        if piece.ptype == PieceType::King && piece.color == Player::Black && from == (0, 4) {
            self.black_king_side = false;
            self.black_queen_side = false;
        }
        if piece.ptype == PieceType::Rook && piece.color == Player::White && from == (7, 0) { self.white_queen_side = false; }
        if piece.ptype == PieceType::Rook && piece.color == Player::White && from == (7, 7) { self.white_king_side = false; }
        if piece.ptype == PieceType::Rook && piece.color == Player::Black && from == (0, 0) { self.black_queen_side = false; }
        if piece.ptype == PieceType::Rook && piece.color == Player::Black && from == (0, 7) { self.black_king_side = false; }

        // Rook capture also revokes rights
        if to == (0, 0) { self.black_queen_side = false; }
        if to == (0, 7) { self.black_king_side = false; }
        if to == (7, 0) { self.white_queen_side = false; }
        if to == (7, 7) { self.white_king_side = false; }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Normal { from: (usize, usize), to: (usize, usize) },
    Promotion { from: (usize, usize), to: (usize, usize), promote_to: PieceType },
    EnPassant { from: (usize, usize), to: (usize, usize), captured_pawn: (usize, usize) },
    CastleKingside { player: Player },
    CastleQueenside { player: Player },
}

fn coords_to_notation(coords: (usize, usize)) -> String {
    let file = (b'a' + coords.1 as u8) as char;
    let rank = (b'8' - coords.0 as u8) as char;
    format!("{}{}", file, rank)
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Move::Normal { from, to } => write!(f, "{}{}", coords_to_notation(*from), coords_to_notation(*to)),
            Move::Promotion { from, to, promote_to } => {
                let piece_char = Piece {ptype: *promote_to, color: Player::White }.to_char().to_ascii_lowercase();
                write!(f, "{}{}{}", coords_to_notation(*from), coords_to_notation(*to), piece_char)
            }
            Move::EnPassant { from, to, .. } => write!(f, "{}{} (en passant)", coords_to_notation(*from), coords_to_notation(*to)),
            Move::CastleKingside { .. } => write!(f, "O-O"),
            Move::CastleQueenside { .. } => write!(f, "O-O-O"),
        }
    }
}


#[derive(Clone)]
pub struct Board {
    squares: [[Option<Piece>; 8]; 8],
    // Stores the square *behind* a pawn that just moved two steps, if any.
    en_passant_target: Option<(usize, usize)>,
    castling_rights: CastlingRights,
}

impl Board {
    pub fn new() -> Self {
        let mut squares = [[None; 8]; 8];
        let mut place = |rank: usize, file: usize, ptype: PieceType, color: Player| {
            squares[rank][file] = Some(Piece { ptype, color });
        };

        for file in 0..8 {
            place(1, file, PieceType::Pawn, Player::Black);
            place(6, file, PieceType::Pawn, Player::White);
        }
        place(0, 0, PieceType::Rook, Player::Black); place(0, 7, PieceType::Rook, Player::Black);
        place(0, 1, PieceType::Knight, Player::Black); place(0, 6, PieceType::Knight, Player::Black);
        place(0, 2, PieceType::Bishop, Player::Black); place(0, 5, PieceType::Bishop, Player::Black);
        place(0, 3, PieceType::Queen, Player::Black); place(0, 4, PieceType::King, Player::Black);
        place(7, 0, PieceType::Rook, Player::White); place(7, 7, PieceType::Rook, Player::White);
        place(7, 1, PieceType::Knight, Player::White); place(7, 6, PieceType::Knight, Player::White);
        place(7, 2, PieceType::Bishop, Player::White); place(7, 5, PieceType::Bishop, Player::White);
        place(7, 3, PieceType::Queen, Player::White); place(7, 4, PieceType::King, Player::White);

        Board {
            squares,
            en_passant_target: None,
            castling_rights: CastlingRights::new(),
        }
    }

    pub fn print(&self) {
        println!("\n   a b c d e f g h");
        println!("  +-----------------+");
        for rank in 0..8 {
            print!("{} |", 8 - rank);
            for file in 0..8 {
                match self.squares[rank][file] {
                    Some(piece) => print!("{} ", piece.to_char()),
                    None => print!(". "),
                }
            }
            println!("| {}", 8 - rank);
        }
        println!("  +-----------------+");
        println!("   a b c d e f g h");
    }

    fn parse_square(notation: &str) -> Option<(usize, usize)> {
        if notation.len() != 2 { return None; }
        let mut chars = notation.chars();
        let file_char = chars.next()?.to_ascii_lowercase();
        let rank_char = chars.next()?;
        let file = match file_char { 'a'..='h' => Some(file_char as usize - 'a' as usize), _ => None }?;
        let rank_num = match rank_char { '1'..='8' => Some(rank_char.to_digit(10)? as usize), _ => None }?;
        let rank = 8 - rank_num;
        if rank > 7 || file > 7 { None } else { Some((rank, file)) }
    }

    // Parses user input like "e2e4" or "e7e8q".
    pub fn parse_move_input(notation: &str) -> Option<((usize, usize), (usize, usize), Option<char>)> {
        let cleaned = notation.trim().to_lowercase();
        if !(cleaned.len() == 4 || cleaned.len() == 5) { return None; }

        let from_str = &cleaned[0..2];
        let to_str = &cleaned[2..4];
        let promotion_char = if cleaned.len() == 5 { cleaned.chars().nth(4) } else { None };

        let from = Self::parse_square(from_str)?;
        let to = Self::parse_square(to_str)?;

        if let Some(p_char) = promotion_char {
            if !['q', 'r', 'b', 'n'].contains(&p_char) {
                return None; // Invalid promotion character
            }
        }
        Some((from, to, promotion_char))
    }

    fn find_king(&self, player: Player) -> Option<(usize, usize)> {
        for r in 0..8 {
            for f in 0..8 {
                if let Some(piece) = self.squares[r][f] {
                    if piece.ptype == PieceType::King && piece.color == player {
                        return Some((r, f));
                    }
                }
            }
        }
        None
    }

    // Checks if a square is attacked by the opponent.
    fn is_square_attacked_by(&self, square: (usize, usize), attacker_color: Player) -> bool {
        for r_att in 0..8 {
            for f_att in 0..8 {
                if let Some(attacker_piece) = self.squares[r_att][f_att] {
                    if attacker_piece.color == attacker_color {
                        if self.can_piece_attack(attacker_piece, (r_att, f_att), square) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    // Checks if a piece at 'from' can attack 'target' (ignoring occupancy, path clear required).
    fn can_piece_attack(&self, piece: Piece, from: (usize, usize), target: (usize, usize)) -> bool {
        let (from_r, from_f) = (from.0 as isize, from.1 as isize);
        let (to_r, to_f) = (target.0 as isize, target.1 as isize);
        let dr = to_r - from_r;
        let df = to_f - from_f;
        let abs_dr = dr.abs();
        let abs_df = df.abs();

        if from == target { return false; }

        match piece.ptype {
            PieceType::Pawn => {
                let direction = if piece.color == Player::White { -1 } else { 1 };
                dr == direction && abs_df == 1
            }
            PieceType::Rook => {
                ((dr == 0 && df != 0) || (df == 0 && dr != 0)) && self.is_path_clear(from, target)
            }
            PieceType::Knight => {
                (abs_dr == 2 && abs_df == 1) || (abs_dr == 1 && abs_df == 2)
            }
            PieceType::Bishop => {
                (abs_dr == abs_df && abs_dr != 0) && self.is_path_clear(from, target)
            }
            PieceType::Queen => {
                let is_rook_path = (dr == 0 && df != 0) || (df == 0 && dr != 0);
                let is_bishop_path = abs_dr == abs_df && abs_dr != 0;
                (is_rook_path || is_bishop_path) && self.is_path_clear(from, target)
            }
            PieceType::King => {
                abs_dr <= 1 && abs_df <= 1
            }
        }
    }

    pub fn is_in_check(&self, player: Player) -> bool {
        if let Some(king_pos) = self.find_king(player) {
            self.is_square_attacked_by(king_pos, player.opponent())
        } else {
            false // Should not happen in a valid game
        }
    }

    // Checks if the path between two squares (exclusive) is empty.
    fn is_path_clear(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let (from_r, from_f) = (from.0 as isize, from.1 as isize);
        let (to_r, to_f) = (to.0 as isize, to.1 as isize);
        let dr = to_r - from_r;
        let df = to_f - from_f;
        let step_r = dr.signum();
        let step_f = df.signum();

        let mut curr_r = from_r + step_r;
        let mut curr_f = from_f + step_f;

        while curr_r != to_r || curr_f != to_f {
            if !(0..8).contains(&curr_r) || !(0..8).contains(&curr_f) {
                return false; // Should not happen with valid moves
            }
            if self.squares[curr_r as usize][curr_f as usize].is_some() {
                return false; // Blocked
            }
            curr_r += step_r;
            curr_f += step_f;
        }
        true
    }

    /// Generates all *pseudo-legal* moves for a player.
    /// Pseudo-legal means the move follows piece rules, but might leave the king in check.
    fn generate_pseudo_legal_moves(&self, player: Player) -> Vec<Move> {
        let mut moves = Vec::new();

        for r_from in 0..8 {
            for f_from in 0..8 {
                if let Some(piece) = self.squares[r_from][f_from] {
                    if piece.color == player {
                        let from = (r_from, f_from);
                        // Generate basic moves and promotions
                        for r_to in 0..8 {
                            for f_to in 0..8 {
                                let to = (r_to, f_to);
                                if self.is_pseudo_legal_basic_move(piece, from, to) {
                                    let promotion_rank = if player == Player::White { 0 } else { 7 };
                                    if piece.ptype == PieceType::Pawn && r_to == promotion_rank {
                                        for promote_to in [PieceType::Queen, PieceType::Rook, PieceType::Bishop, PieceType::Knight] {
                                            moves.push(Move::Promotion { from, to, promote_to });
                                        }
                                    } else {
                                        moves.push(Move::Normal { from, to });
                                    }
                                }
                            }
                        }

                        // Generate En Passant
                        if piece.ptype == PieceType::Pawn {
                            if let Some(ep_target) = self.en_passant_target {
                                if self.is_pseudo_legal_en_passant(piece, from, ep_target) {
                                    let captured_pawn_rank = from.0;
                                    let captured_pawn_file = ep_target.1;
                                    moves.push(Move::EnPassant { from, to: ep_target, captured_pawn: (captured_pawn_rank, captured_pawn_file) });
                                }
                            }
                        }
                    }
                }
            }
        }

        // Generate Castling
        self.generate_castling_moves(player, &mut moves);

        moves
    }

    // Simplified check for non-special moves (no EP, no castling).
    fn is_pseudo_legal_basic_move(&self, piece: Piece, from: (usize, usize), to: (usize, usize)) -> bool {
        let (from_r, from_f) = (from.0 as isize, from.1 as isize);
        let (to_r, to_f) = (to.0 as isize, to.1 as isize);
        let dr = to_r - from_r;
        let df = to_f - from_f;
        let abs_dr = dr.abs();
        let abs_df = df.abs();

        if from == to { return false; }

        if let Some(target_piece) = self.squares[to.0][to.1] {
            if target_piece.color == piece.color { return false; }
        }

        match piece.ptype {
            PieceType::Pawn => {
                let direction = if piece.color == Player::White { -1 } else { 1 };
                let start_rank = if piece.color == Player::White { 6 } else { 1 };

                // Move 1 square forward
                if df == 0 && dr == direction && self.squares[to.0][to.1].is_none() { return true; }
                // Move 2 squares forward
                if df == 0 && dr == 2 * direction && from_r == start_rank as isize && self.squares[to.0][to.1].is_none() {
                    let intermediate_rank = (from_r + direction) as usize;
                    return self.squares[intermediate_rank][from_f as usize].is_none();
                }
                // Capture diagonally
                if abs_df == 1 && dr == direction {
                    if let Some(target_piece) = self.squares[to.0][to.1] {
                        return target_piece.color != piece.color;
                    }
                }
                false
            }
            PieceType::Rook => {
                ((dr == 0 && df != 0) || (df == 0 && dr != 0)) && self.is_path_clear(from, to)
            }
            PieceType::Knight => {
                (abs_dr == 2 && abs_df == 1) || (abs_dr == 1 && abs_df == 2)
            }
            PieceType::Bishop => {
                (abs_dr == abs_df && abs_dr != 0) && self.is_path_clear(from, to)
            }
            PieceType::Queen => {
                let is_rook_move = (dr == 0 && df != 0) || (df == 0 && dr != 0);
                let is_bishop_move = abs_dr == abs_df && abs_dr != 0;
                (is_rook_move || is_bishop_move) && self.is_path_clear(from, to)
            }
            PieceType::King => {
                // Castling handled separately
                abs_dr <= 1 && abs_df <= 1
            }
        }
    }

    fn is_pseudo_legal_en_passant(&self, piece: Piece, from: (usize, usize), ep_target: (usize, usize)) -> bool {
        if piece.ptype != PieceType::Pawn { return false; }
        let (from_r, from_f) = (from.0 as isize, from.1 as isize);
        let (to_r, to_f) = (ep_target.0 as isize, ep_target.1 as isize);
        let dr = to_r - from_r;
        let df = to_f - from_f;
        let direction = if piece.color == Player::White { -1 } else { 1 };

        dr == direction && df.abs() == 1
    }

    // Adds pseudo-legal castling moves to the list.
    fn generate_castling_moves(&self, player: Player, moves: &mut Vec<Move>) {
        if self.is_in_check(player) { return; } // Cannot castle out of check

        match player {
            Player::White => {
                // Kingside
                if self.castling_rights.white_king_side
                    && self.squares[7][5].is_none() && self.squares[7][6].is_none()
                    && self.squares[7][7] == Some(Piece { ptype: PieceType::Rook, color: Player::White })
                    && !self.is_square_attacked_by((7, 5), Player::Black)
                    && !self.is_square_attacked_by((7, 6), Player::Black)
                {
                    moves.push(Move::CastleKingside { player });
                }
                // Queenside
                if self.castling_rights.white_queen_side
                    && self.squares[7][1].is_none() && self.squares[7][2].is_none() && self.squares[7][3].is_none()
                    && self.squares[7][0] == Some(Piece { ptype: PieceType::Rook, color: Player::White })
                    && !self.is_square_attacked_by((7, 2), Player::Black)
                    && !self.is_square_attacked_by((7, 3), Player::Black)
                {
                    moves.push(Move::CastleQueenside { player });
                }
            }
            Player::Black => {
                // Kingside
                if self.castling_rights.black_king_side
                    && self.squares[0][5].is_none() && self.squares[0][6].is_none()
                    && self.squares[0][7] == Some(Piece { ptype: PieceType::Rook, color: Player::Black })
                    && !self.is_square_attacked_by((0, 5), Player::White)
                    && !self.is_square_attacked_by((0, 6), Player::White)
                {
                    moves.push(Move::CastleKingside { player });
                }
                // Queenside
                if self.castling_rights.black_queen_side
                    && self.squares[0][1].is_none() && self.squares[0][2].is_none() && self.squares[0][3].is_none()
                    && self.squares[0][0] == Some(Piece { ptype: PieceType::Rook, color: Player::Black })
                    && !self.is_square_attacked_by((0, 2), Player::White)
                    && !self.is_square_attacked_by((0, 3), Player::White)
                {
                    moves.push(Move::CastleQueenside { player });
                }
            }
        }
    }

    // Generates all strictly legal moves (pseudo-legal moves that don't leave the king in check).
    pub fn get_all_legal_moves(&self, player: Player) -> Vec<Move> {
        let pseudo_legal_moves = self.generate_pseudo_legal_moves(player);
        let mut legal_moves = Vec::new();

        for game_move in pseudo_legal_moves {
            let mut temp_board = self.clone();
            temp_board.apply_move_internal(&game_move); // Apply the move on a temporary board

            if !temp_board.is_in_check(player) { // Check if the player's king is safe
                legal_moves.push(game_move);
            }
        }
        legal_moves
    }

    /// Applies any type of move to the board *without* checking legality.
    /// Updates castling rights and en passant target.
    fn apply_move_internal(&mut self, game_move: &Move) {
        let mut new_ep_target = None; // Will be set if a pawn moves two squares

        // Update castling rights based on the piece being moved and its destination
        let piece_to_move = match game_move {
            Move::Normal { from, ..} | Move::Promotion { from, ..} | Move::EnPassant { from, ..} => self.squares[from.0][from.1],
            Move::CastleKingside { player } | Move::CastleQueenside { player } => {
                let rank = if *player == Player::White { 7 } else { 0 };
                self.squares[rank][4] // The King
            }
        };

        if let Some(piece) = piece_to_move {
            let from_coord = match game_move {
                Move::Normal { from, .. } | Move::Promotion { from, .. } | Move::EnPassant { from, .. } => *from,
                Move::CastleKingside { player } | Move::CastleQueenside { player } => if *player == Player::White { (7, 4) } else { (0, 4) },
            };
            let to_coord = match game_move {
                Move::Normal { to, .. } | Move::Promotion { to, .. } | Move::EnPassant { to, .. } => *to,
                Move::CastleKingside { player } => if *player == Player::White { (7, 6) } else { (0, 6) },
                Move::CastleQueenside { player } => if *player == Player::White { (7, 2) } else { (0, 2) },
            };
            self.castling_rights.update_on_move(from_coord, to_coord, piece);
        }

        // Reset the en passant target from the previous turn before applying the move
        self.en_passant_target = None;

        // Execute the specific move logic
        match game_move {
            Move::Normal { from, to } => {
                if let Some(piece) = self.squares[from.0][from.1].take() {
                    if piece.ptype == PieceType::Pawn {
                        let dr = (to.0 as isize - from.0 as isize).abs();
                        if dr == 2 { // Pawn moved two squares, set new EP target
                            let ep_rank = (from.0 + to.0) / 2;
                            new_ep_target = Some((ep_rank, from.1));
                        }
                    }
                    self.squares[to.0][to.1] = Some(piece);
                }
            }
            Move::Promotion { from, to, promote_to } => {
                if let Some(pawn) = self.squares[from.0][from.1].take() {
                    self.squares[to.0][to.1] = Some(Piece { ptype: *promote_to, color: pawn.color });
                }
            }
            Move::EnPassant { from, to, captured_pawn } => {
                if let Some(piece) = self.squares[from.0][from.1].take() {
                    self.squares[to.0][to.1] = Some(piece); // Move attacking pawn
                    self.squares[captured_pawn.0][captured_pawn.1] = None; // Remove captured pawn
                }
            }
            Move::CastleKingside { player } => {
                let rank = if *player == Player::White { 7 } else { 0 };
                let king = self.squares[rank][4].take();
                let rook = self.squares[rank][7].take();
                self.squares[rank][6] = king; // King to g1/g8
                self.squares[rank][5] = rook; // Rook to f1/f8
            }
            Move::CastleQueenside { player } => {
                let rank = if *player == Player::White { 7 } else { 0 };
                let king = self.squares[rank][4].take();
                let rook = self.squares[rank][0].take();
                self.squares[rank][2] = king; // King to c1/c8
                self.squares[rank][3] = rook; // Rook to d1/d8
            }
        }
        // Set the en passant target for the *next* turn, if applicable
        self.en_passant_target = new_ep_target;
    }

    pub fn is_checkmate(&self, player: Player) -> bool {
        self.is_in_check(player) && self.get_all_legal_moves(player).is_empty()
    }

    pub fn is_stalemate(&self, player: Player) -> bool {
        !self.is_in_check(player) && self.get_all_legal_moves(player).is_empty()
    }

    /// Attempts to make a legal move based on user input coordinates.
    /// Handles promotion prompting if necessary.
    pub fn make_legal_move(
        &mut self,
        from: (usize, usize),
        to: (usize, usize),
        promotion_char_opt: Option<char>,
        current_player: Player,
    ) -> Result<Move, &'static str> {

        let piece_to_move = match self.squares[from.0][from.1] {
            Some(p) if p.color == current_player => p,
            Some(_) => return Err("Cannot move opponent's piece."),
            None => return Err("No piece at start coordinate."),
        };

        // Determine the *intended* move type based on input
        let intended_move: Move;
        if piece_to_move.ptype == PieceType::King {
            let df = to.1 as isize - from.1 as isize;
            if df == 2 { intended_move = Move::CastleKingside { player: current_player }; }
            else if df == -2 { intended_move = Move::CastleQueenside { player: current_player }; }
            else { intended_move = Move::Normal { from, to }; }
        }
        else if piece_to_move.ptype == PieceType::Pawn && Some(to) == self.en_passant_target && self.is_pseudo_legal_en_passant(piece_to_move, from, to) {
            let captured_pawn_rank = from.0;
            let captured_pawn_file = to.1;
            intended_move = Move::EnPassant { from, to, captured_pawn: (captured_pawn_rank, captured_pawn_file) };
        }
        else if piece_to_move.ptype == PieceType::Pawn {
            let promotion_rank = if current_player == Player::White { 0 } else { 7 };
            if to.0 == promotion_rank {
                let promotion_piece_type = match promotion_char_opt {
                    Some('q') => PieceType::Queen,
                    Some('r') => PieceType::Rook,
                    Some('b') => PieceType::Bishop,
                    Some('n') => PieceType::Knight,
                    Some(_) => return Err("Invalid promotion character (use q, r, b, or n)."),
                    None => { // Prompt user if no character provided
                        match self.prompt_for_promotion() {
                            Ok(ptype) => ptype,
                            Err(e) => return Err(e),
                        }
                    }
                };
                intended_move = Move::Promotion { from, to, promote_to: promotion_piece_type };
            } else { intended_move = Move::Normal { from, to }; }
        }
        else { intended_move = Move::Normal { from, to }; }

        // Check if the intended move is in the list of *strictly* legal moves
        let legal_moves = self.get_all_legal_moves(current_player);
        if legal_moves.contains(&intended_move) {
            self.apply_move_internal(&intended_move); // Apply the move to the actual board
            Ok(intended_move)
        } else {
            // Provide more specific error messages
            if let Move::CastleKingside { .. } | Move::CastleQueenside { .. } = intended_move {
                Err("Castling is not possible (check rights, path, or attacks).")
            } else if self.is_pseudo_legal_basic_move(piece_to_move, from, to) || matches!(intended_move, Move::EnPassant{..}) || matches!(intended_move, Move::Promotion{..}) {
                Err("Move would put/leave the king in check.")
            } else {
                Err("Invalid move (piece rules, destination, blocked path, or special rules).")
            }
        }
    }

    // Prompts the user for a piece type during pawn promotion.
    fn prompt_for_promotion(&self) -> Result<PieceType, &'static str> {
        loop {
            print!("Promote pawn to (Q, R, B, N): ");
            io::stdout().flush().map_err(|_| "I/O Error flushing.")?;
            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|_| "I/O Error reading.")?;
            match input.trim().to_uppercase().as_str() {
                "Q" => return Ok(PieceType::Queen),
                "R" => return Ok(PieceType::Rook),
                "B" => return Ok(PieceType::Bishop),
                "N" => return Ok(PieceType::Knight),
                _ => println!("Invalid choice. Try again."),
            }
        }
    }
}

#[derive(Debug)]
pub struct GameTimer {
    white_time: Duration,
    black_time: Duration,
    current_turn: Player,
    turn_start_time: Option<Instant>,
    initial_time: Duration,
}

impl GameTimer {
    pub fn new(initial_time: Duration) -> Self {
        GameTimer {
            white_time: initial_time,
            black_time: initial_time,
            current_turn: Player::White,
            turn_start_time: None,
            initial_time,
        }
    }

    pub fn start_turn(&mut self) {
        if self.turn_start_time.is_none() {
            self.turn_start_time = Some(Instant::now());
        }
    }

    pub fn stop_turn_timing(&mut self) -> Duration {
        let mut elapsed = Duration::ZERO;
        if let Some(start_time) = self.turn_start_time.take() {
            elapsed = start_time.elapsed();
            match self.current_turn {
                Player::White => self.white_time = self.white_time.saturating_sub(elapsed),
                Player::Black => self.black_time = self.black_time.saturating_sub(elapsed),
            }
        }
        elapsed
    }

    pub fn switch_player(&mut self) {
        self.current_turn = self.current_turn.opponent();
    }

    pub fn get_remaining_time(&self, player: Player) -> Duration {
        let base_time = if player == Player::White { self.white_time } else { self.black_time };
        if self.current_turn == player && self.turn_start_time.is_some() {
            base_time.saturating_sub(self.turn_start_time.unwrap().elapsed())
        } else {
            base_time
        }
    }

    pub fn current_player(&self) -> Player { self.current_turn }
    pub fn initial_time(&self) -> Duration { self.initial_time }
}

// --- Game Mode Enum ---
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    PlayerVsPlayer,
    PlayerVsBot { human_player: Player }, // Store which color the human is
}

pub struct Game {
    board: Board,
    timer: GameTimer,
    turn_count: u32, // Number of half-moves
    max_turns: u32, // Max full turns
    game_mode: GameMode, // Store the selected game mode
    game_over: bool,
    winner: Option<Player>,
    game_over_reason: String,
    last_move: Option<Move>,
}

impl Game {
    pub fn new(initial_time: Duration, max_turns: u32, game_mode: GameMode) -> Self {
        Game {
            board: Board::new(),
            timer: GameTimer::new(initial_time),
            turn_count: 0,
            max_turns,
            game_mode, // Initialize game mode
            game_over: false,
            winner: None,
            game_over_reason: String::new(),
            last_move: None,
        }
    }

    /// Simple Bot: Chooses a random legal move.
    fn get_bot_move(&self, bot_player: Player) -> Option<Move> {
        let legal_moves = self.board.get_all_legal_moves(bot_player);
        if legal_moves.is_empty() {
            None // No legal moves (checkmate or stalemate)
        } else {
            // Choose a random move from the list
            let mut rng = thread_rng();
            legal_moves.choose(&mut rng).copied() // .copied() because choose returns &Move
        }
    }


    pub fn run(&mut self) {
        println!("=== Chess Game 1===");
        println!("Enter moves in 'e2e4' format. For promotion: 'e7e8q'.");
        println!("For castling, move the king 2 squares (e.g., 'e1g1' for White Kingside).");
        println!("'exit' to quit.");
        println!("Game Mode: {:?}", self.game_mode);
        println!("Initial time per player: {:?}", self.timer.initial_time());
        println!("Game ends after {} full turns, timeout, checkmate, or stalemate.", self.max_turns);
        println!("==========================================================");

        while !self.game_over {
            let current_player = self.timer.current_player();
            let full_turn_number = (self.turn_count / 2) + 1;

            // --- Check Game End Conditions ---
            if self.board.is_checkmate(current_player) {
                self.game_over = true;
                self.winner = Some(current_player.opponent());
                self.game_over_reason = format!("Checkmate! {:?} wins.", current_player.opponent());
                self.board.print();
                println!("\n--- {} ---", self.game_over_reason);
                break;
            } else if self.board.is_stalemate(current_player) {
                self.game_over = true;
                self.winner = None;
                self.game_over_reason = "Stalemate! Draw.".to_string();
                self.board.print();
                println!("\n--- {} ---", self.game_over_reason);
                break;
            }

            // --- Print Board and Status ---
            println!("\n--- Turn {} ({:?}) ---", full_turn_number , current_player);
            self.board.print();
            if let Some(ref mv) = self.last_move {
                println!("Last move ({:?}): {}", current_player.opponent(), mv);
            }
            if self.board.is_in_check(current_player) {
                println!("*** CHECK! ***");
            }
            let white_remaining = self.timer.get_remaining_time(Player::White);
            let black_remaining = self.timer.get_remaining_time(Player::Black);
            println!(
                "Time White: {:.1}s | Time Black: {:.1}s",
                white_remaining.as_secs_f32(),
                black_remaining.as_secs_f32()
            );

            // --- Check Max Turns ---
            if full_turn_number > self.max_turns {
                self.game_over = true;
                self.winner = None;
                self.game_over_reason = format!("{} turn limit reached!", self.max_turns);
                println!("\n--- {} ---", self.game_over_reason);
                break;
            }

            // --- Determine if Human or Bot Turn ---
            let is_human_turn = match self.game_mode {
                GameMode::PlayerVsPlayer => true,
                GameMode::PlayerVsBot { human_player } => current_player == human_player,
            };

            // --- Handle Turn ---
            if is_human_turn {
                // --- Human Player's Turn ---
                self.timer.start_turn();
                let (tx, rx) = mpsc::channel::<String>();
                let input_player = current_player;
                let input_thread_handle = thread::spawn(move || {
                    print!("Move {:?} (or 'exit'): ", input_player);
                    io::stdout().flush().expect("Error flushing stdout.");
                    let mut input = String::new();
                    match io::stdin().read_line(&mut input) {
                        Ok(_) => { tx.send(input.trim().to_string()).ok(); }
                        Err(e) => { eprintln!("\nError reading input: {}", e); }
                    }
                });

                let mut move_input_received: Option<String> = None;
                let timeout_occurred = false;

                // Input/Timeout Loop
                loop {
                    match rx.try_recv() {
                        Ok(received_input) => { move_input_received = Some(received_input); break; }
                        Err(TryRecvError::Empty) => {} // Input not ready yet
                        Err(TryRecvError::Disconnected) => {
                            if !timeout_occurred {
                                self.game_over = true;
                                self.game_over_reason = "Input thread error".to_string();
                                eprintln!("\nError: Input channel disconnected.");
                            }
                            break;
                        }
                    }

                    if self.timer.get_remaining_time(current_player).is_zero() {
                        self.game_over = true;
                        self.winner = Some(current_player.opponent());
                        self.game_over_reason = format!("Time expired for {:?}! {:?} wins.", current_player, current_player.opponent());
                        println!("\n!!! {} !!!", self.game_over_reason);
                        break;
                    }
                    thread::sleep(Duration::from_millis(100)); // Prevent busy-waiting
                }

                let _ = self.timer.stop_turn_timing();
                input_thread_handle.join().expect("Input thread panicked");

                if self.game_over { break; } // Exit if timeout or thread error occurred

                // Process Human Input
                if let Some(input) = move_input_received {
                    if input.eq_ignore_ascii_case("exit") {
                        self.game_over = true;
                        self.winner = None;
                        self.game_over_reason = format!("Player {:?} chose to exit.", current_player);
                        println!("\n--- {} ---", self.game_over_reason);
                        break;
                    }

                    match Board::parse_move_input(&input) {
                        Some((from_coords, to_coords, promotion_char)) => {
                            match self.board.make_legal_move(from_coords, to_coords, promotion_char, current_player) {
                                Ok(executed_move) => {
                                    self.last_move = Some(executed_move);
                                    self.timer.switch_player();
                                    self.turn_count += 1;
                                }
                                Err(e) => {
                                    println!("Invalid move: '{}'. Reason: {}. Try again.", input, e);
                                }
                            }
                        }
                        None => {
                            println!("Invalid move format: '{}'. Use 'e2e4', 'e7e8q', 'e1g1', etc. Try again.", input);
                        }
                    }
                } else {
                    if !self.game_over {
                        eprintln!("Error: No input received from user.");
                        self.game_over = true;
                        self.game_over_reason = "Unexpected input error".to_string();
                    }
                }

            } else {
                // --- Bot Player's Turn ---
                println!("Bot ({:?}) is thinking...", current_player);
                thread::sleep(Duration::from_millis(500)); // Simulate thinking time

                if let Some(bot_move) = self.get_bot_move(current_player) {
                    println!("Bot move: {}", bot_move);
                    self.board.apply_move_internal(&bot_move); // Apply the chosen legal move
                    self.last_move = Some(bot_move);
                    self.timer.switch_player();
                    self.turn_count += 1;
                } else {
                    // This should only happen if the bot has no moves, which means
                    // the game should have ended in checkmate/stalemate already.
                    // We'll print an error just in case.
                    eprintln!("Error: Bot has no legal moves, but game end not detected earlier.");
                    self.game_over = true; // Force game over
                    self.game_over_reason = "Internal error: Bot stuck?".to_string();
                }
            }
        } // End of main game loop

        // --- Final Game State Output ---
        println!("\n--- GAME OVER ---");
        if !self.game_over_reason.is_empty() {
            println!("Reason: {}", self.game_over_reason);
        }
        self.board.print();
        if let Some(ref mv) = self.last_move {
            if !(self.game_over_reason.contains("expired") || self.game_over_reason.contains("chose to exit")) {
                println!("Last move ({:?}): {}", self.timer.current_player(), mv);
            }
        }
        let white_final = self.timer.get_remaining_time(Player::White);
        let black_final = self.timer.get_remaining_time(Player::Black);
        println!(
            "Final Time - White: {:.1}s | Black: {:.1}s",
            white_final.as_secs_f32(),
            black_final.as_secs_f32()
        );
        if let Some(winner) = self.winner {
            println!("Winner: {:?}", winner);
        } else {
            println!("Result: Draw or undetermined.");
        }
        println!("===================");
    }
}

// --- Function to get user input with prompt ---
fn get_user_input(prompt: &str) -> String {
    loop {
        print!("{}", prompt);
        io::stdout().flush().expect("Failed to flush stdout.");
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            return input.trim().to_string();
        } else {
            println!("Error reading input. Please try again.");
        }
    }
}

// --- Program Entry Point ---
fn main() {
    println!("Select Game Mode:");
    println!("  1. Player vs Player");
    println!("  2. Player vs Bot");

    let game_mode: GameMode;
    loop {
        match get_user_input("Enter choice (1-2): ").parse::<u32>() {
            Ok(1) => {
                game_mode = GameMode::PlayerVsPlayer;
                break;
            }
            Ok(2) => {
                println!("Play as:");
                println!("  1. White");
                println!("  2. Black");
                loop {
                    match get_user_input("Enter choice (1-2): ").parse::<u32>() {
                        Ok(1) => {
                            game_mode = GameMode::PlayerVsBot { human_player: Player::White };
                            break;
                        }
                        Ok(2) => {
                            game_mode = GameMode::PlayerVsBot { human_player: Player::Black };
                            break;
                        }
                        Ok(_) => println!("Invalid choice. Please enter 1 or 2."),
                        Err(_) => println!("Invalid input. Please enter a number."),
                    }
                }
                break; // Break outer loop once color is chosen
            }
            Ok(_) => {
                println!("Invalid choice. Please enter 1 or 2.");
            }
            Err(_) => {
                println!("Invalid input. Please enter a number.");
            }
        }
    }


    println!("\nSelect Time Control:");
    println!("  1. Standard (60 minutes)");
    println!("  2. Rapid    (10 minutes)");
    println!("  3. Blitz    ( 5 minutes)");
    println!("  4. Bullet   ( 1 minute)");

    let initial_time: Duration;
    loop {
        match get_user_input("Enter choice (1-4): ").parse::<u32>() {
            Ok(1) => { initial_time = Duration::from_secs(60 * 60); break; }
            Ok(2) => { initial_time = Duration::from_secs(10 * 60); break; }
            Ok(3) => { initial_time = Duration::from_secs(5 * 60); break; }
            Ok(4) => { initial_time = Duration::from_secs(1 * 60); break; }
            Ok(_) => {
                println!("Invalid choice. Please enter a number between 1 and 4.");
            }
            Err(_) => {
                println!("Invalid input. Please enter a number.");
            }
        }
    }

    let max_turns = 100; // Limit to 100 full turns

    let mut game = Game::new(initial_time, max_turns, game_mode);
    game.run();
}

// Might need some work.