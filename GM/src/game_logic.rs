use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    White,
    Black,
}

impl Player {
    pub fn opponent(&self) -> Player {
        match self {
            Player::White => Player::Black,
            Player::Black => Player::White,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PieceType {
    Pawn, King, Queen, Rook, Bishop, Knight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Player,
    pub has_moved: bool,
}

pub struct Board {
    pub squares: [[Option<Piece>; 8]; 8],
}

impl Board {
    pub fn new() -> Self {
        Board { squares: [[None; 8]; 8] }
    }

    pub fn make_legal_move(&mut self, _from: (usize, usize), _to: (usize, usize), _promotion: Option<PieceType>, _turn: Player) -> Result<(), &'static str> {
        Ok(())
    }

    pub fn is_checkmate(&self, _turn: Player) -> bool {
        false
    }

    pub fn is_stalemate(&self, _turn: Player) -> bool {
        false
    }
}

pub struct GameTimer {
    white_time: Duration,
    black_time: Duration,
    turn_start_time: Option<Instant>,
}

impl GameTimer {
    pub fn new(initial_time: Duration) -> Self {
        GameTimer {
            white_time: initial_time,
            black_time: initial_time,
            turn_start_time: None,
        }
    }

    pub fn start_turn(&mut self) {
        self.turn_start_time = Some(Instant::now());
    }

    pub fn stop_turn_timing(&mut self, player_who_moved: Player) {
        if let Some(start_time) = self.turn_start_time.take() {
            let elapsed = start_time.elapsed();
            match player_who_moved {
                Player::White => {
                    self.white_time = self.white_time.saturating_sub(elapsed);
                }
                Player::Black => {
                    self.black_time = self.black_time.saturating_sub(elapsed);
                }
            }
        }
    }

    pub fn get_remaining_time(&self, player: Player) -> Duration {
        match player {
            Player::White => self.white_time,
            Player::Black => self.black_time,
        }
    }

    pub fn time_left_on_current_turn(&self, current_player: Player) -> Option<Duration> {
        self.turn_start_time.map(|start_time| {
            let elapsed = start_time.elapsed();
            let remaining_base = self.get_remaining_time(current_player);
            remaining_base.saturating_sub(elapsed)
        })
    }
}