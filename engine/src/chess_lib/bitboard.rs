use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr};


/// Represents a square on an 8x8 chessboard.
/// The squares are numbered 0 to 63, starting from A1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Square {
    A1 = 0, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8
}

//      ^
// File |
//     a1 -->
//          Rank
// "a file rank 1"

impl Square {
    #[inline]
    pub fn from_u8(val: u8) -> Self {
        unsafe { std::mem::transmute(val) }
    }

    #[inline]
    pub fn from_file_rank(file: u8, rank: u8) -> Self {
        Self::from_u8(rank * 8 + file)
    }

    #[inline]
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Gets the file (column) of the square (0-7, where 0 is 'a'-file).
    #[inline]
    pub fn file(self) -> u8 {
        self.to_u8() % 8
    }

    /// Gets the rank (row) of the square (0-7, where 0 is 1st rank).
    #[inline]
    pub fn rank(self) -> u8 {
        self.to_u8() / 8
    }

    /// Optionally creates a square by safely shifting.
    /// Returns None if the shift goes off board.
    pub fn try_offset(self, file_offset: i8, rank_offset: i8) -> Option<Square> {
        let current_file = self.file() as i8;
        let current_rank = self.rank() as i8;

        let new_file = current_file + file_offset;
        let new_rank = current_rank + rank_offset;

        if (0..8).contains(&new_file) && (0..8).contains(&new_rank) {
            Some(Square::from_file_rank(new_file as u8, new_rank as u8))
        } else {
            None
        }
    }

    /// Converts algebraic notation (e.g., "e4") to a Square.
    /// Returns None if the notation is invalid.
    pub fn from_algebraic(notation: &str) -> Option<Square> {
        if notation.len() != 2 {
            return None;
        }
        let mut chars = notation.chars();
        let file_char = chars.next()?;
        let rank_char = chars.next()?;

        let file = match file_char {
            'a'..='h' => file_char as u8 - b'a',
            _ => return None,
        };
        let rank = match rank_char {
            '1'..='8' => rank_char as u8 - b'1',
            _ => return None,
        };
        Some(Square::from_file_rank(file, rank))
    }

    pub fn to_algebraic(&self) -> String {
        let file_char = (b'a' + self.file()) as char;
        let rank_char = (b'1' + self.rank()) as char;
        format!("{}{}", file_char, rank_char)
    }

    pub fn is_light(&self) -> bool {
        // (self.file() + self.rank()) % 2 != 0
        ((self.to_u8() >> 3) & 1) ^ (self.to_u8() & 1) == 1
    }

    pub fn surrounding_squares(self) -> impl Iterator<Item = Square> {
        [
            (-1, -1), (0, -1), (1, -1), // Rank below
            (-1,  0),          (1,  0), // Same rank
            (-1,  1), (0,  1), (1,  1), // Rank above
        ]
        .into_iter()
        .filter_map(move |(file_offset, rank_offset)| {
            self.try_offset(file_offset, rank_offset)
        })
    }
}

/// Represents an 8x8 bitboard using a u64.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Bitboard(pub u64);

impl Bitboard {
    /// An empty bitboard.
    pub const EMPTY: Self = Bitboard(0);
    /// A bitboard with all bits set (full board).
    pub const FULL: Self = Bitboard(u64::MAX);

    #[inline]
    pub fn new() -> Self {
        Bitboard::EMPTY
    }

    #[inline]
    pub fn from_u64(value: u64) -> Self {
        Bitboard(value)
    }

    #[inline]
    pub fn from_square(square: Square) -> Self {
        Bitboard(1u64 << square.to_u8())
    }
    
    #[inline]
    pub fn as_u64(self) -> u64 {
        self.0
    }

    #[inline]
    pub fn set(&mut self, square: Square) {
        self.0 |= 1u64 << square.to_u8();
    }

    #[inline]
    pub fn clear(&mut self, square: Square) {
        self.0 &= !(1u64 << square.to_u8());
    }

    #[inline]
    pub fn toggle(&mut self, square: Square) {
        self.0 ^= 1u64 << square.to_u8();
    }

    #[inline]
    pub fn is_set(self, square: Square) -> bool {
        (self.0 & (1u64 << square.to_u8())) != 0
    }


    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn is_not_empty(self) -> bool {
        self.0 != 0
    }
    
    
    #[inline]
    pub fn popcount(self) -> u32 {
        self.0.count_ones()
    }


    /// Finds the index of the least significant bit (LSB).
    #[inline]
    pub fn lsb(self) -> Option<Square> {
        if self.is_empty() {
            None
        } else {
            Some(Square::from_u8(self.0.trailing_zeros() as u8))
        }
    }

    /// Finds the index of the most significant bit (MSB).
    #[inline]
    pub fn msb(self) -> Option<Square> {
        if self.is_empty() {
            None
        } else {
            Some(Square::from_u8((63 - self.0.leading_zeros()) as u8))
        }
    }

    /// Finds and clears the least significant bit (LSB), returning its square.
    #[inline]
    pub fn pop_lsb(&mut self) -> Option<Square> {
        let lsb_square = self.lsb();
        if let Some(lsb_square) = lsb_square {
            self.clear(lsb_square);
        }
        lsb_square
    }


    #[inline]
    pub fn iter(self) -> BitboardIterator {
        BitboardIterator { bitboard: self }
    }
}




// Implementations for bitwise operators

impl BitAnd for Bitboard {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for Bitboard {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitXor for Bitboard {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        Bitboard(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bitboard {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0;
    }
}

impl Not for Bitboard {
    type Output = Self;
    #[inline]
    fn not(self) -> Self::Output {
        Bitboard(!self.0)
    }
}



// Implementations for shift operators
// Note: Shifting a bitboard by N means shifting all pieces N squares.
// TODO: implement specific directional shifts (North, South, etc.) if needed

impl Shl<u8> for Bitboard {
    type Output = Self;
    /// Shifts all bits left by `rhs` positions.
    /// Bits shifted off the "left" (MSB side) are lost.
    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl Shr<u8> for Bitboard {
    type Output = Self;
    /// Shifts all bits right by `rhs` positions.
    /// Bits shifted off the "right" (LSB side) are lost.
    #[inline]
    fn shr(self, rhs: u8) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}




/// Iterator for efficiently iterating over set squares in a bitboard.
pub struct BitboardIterator {
    bitboard: Bitboard,
}

impl Iterator for BitboardIterator {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.bitboard.is_empty() {
            None
        } else {
            self.bitboard.pop_lsb()
        }
    }
}


/// Pretty-prints the bitboard in an 8x8 grid.
impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  a b c d e f g h")?; // File labels
        for rank in (0..8).rev() { // Iterate ranks from 7 (8th rank) down to 0 (1st rank)
            write!(f, "{} ", rank + 1)?; // Rank label
            for file in 0..8 { // Iterate files from 0 ('a') to 7 ('h')
                let square = Square::from_file_rank(file, rank);
                if self.is_set(square) {
                    write!(f, "X ")?;
                } else {
                    write!(f, ". ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}
