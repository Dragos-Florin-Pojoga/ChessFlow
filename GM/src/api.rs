use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlayerColor {
    White,
    Black,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum IncomingMessage {
    StartGame {
        game_id: String,
        white_elo: u32,
        black_elo: u32,
        initial_time_ms: u32,
        increment_ms: u32,
        is_bot_game: bool,
    },
    MakeMove {
        san_move: String,
    },
    Resign {
        player_color: PlayerColor,
    },
    ClaimDraw,
    RequestBoard,
    Abort,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OutgoingMessage {
    MoveResult {
        fen: String,
        turn: PlayerColor,
        white_ms: u32,
        black_ms: u32,
        last_move: Option<String>,
        is_valid: bool,
        message: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        move_history: Option<String>,
    },
    GameOver {
        reason: GameOverReason,
        winner: Option<PlayerColor>,
        white_elo_change: i32,
        black_elo_change: i32,
        fen: String,
        pgn: String,
        move_count: u32,
    },
    Error {
        message: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum GameOverReason {
    Checkmate,
    Resignation,
    Stalemate,
    Timeout,
    AgreedDraw,
    Aborted,
}