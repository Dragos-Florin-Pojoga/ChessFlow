mod api;
mod game_logic;

use api::{IncomingMessage, OutgoingMessage, GameOverReason, PlayerColor};
use game_logic::{Board, GameTimer, PieceType, Player};
use std::env;
use std::time::Duration;
use std::str::FromStr;

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::{ServerOptions, NamedPipeServer};
use tokio::time::sleep;
use shakmaty::{Position, san::San};

struct GameSession {
    board: Board,
    timer: GameTimer,
    turn: Player,
    shakmaty_helper: shakmaty::Chess,
    last_move_san: Option<String>,
    white_elo: u32,
    black_elo: u32,
    is_bot_game: bool,
    move_history: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let simple_pipe_name = env::args().nth(1).ok_or_else(|| anyhow::anyhow!("Pipe name not provided."))?;
    let full_pipe_path = format!(r"\\.\pipe\{}", simple_pipe_name);
    println!("GM Process: Creating server on pipe '{}'.", full_pipe_path);
    let server = ServerOptions::new().first_pipe_instance(true).create(&full_pipe_path)?;
    println!("GM Process: Waiting for client to connect...");
    server.connect().await?;
    println!("GM Process: Client connected.");
    handle_client(server).await?;
    println!("GM Process shutting down.");
    Ok(())
}

async fn handle_client(server: NamedPipeServer) -> anyhow::Result<()> {
    let (read_half, mut write_half) = io::split(server);
    let mut reader = BufReader::new(read_half);
    let mut line = String::new();
    let mut session: Option<GameSession> = None;

    loop {
        let sleep_future = if let Some(s) = &session {
            if let Some(time_left) = s.timer.time_left_on_current_turn(s.turn) {
                sleep(time_left)
            } else {
                sleep(Duration::from_secs(u64::MAX))
            }
        } else {
            sleep(Duration::from_secs(u64::MAX))
        };

        tokio::select! {
            result = reader.read_line(&mut line) => {
                match result {
                    Ok(0) => { println!("Client disconnected."); break; }
                    Ok(_) => {
                        println!("[SERVER] Received: {}", line.trim());

                        let response = handle_incoming_message(&mut session, &line);

                        if let Some(resp) = response {
                            let json = serde_json::to_string(&resp)? + "\n";
                            println!("[SERVER] Sending: {}", json.trim());
                            write_half.write_all(json.as_bytes()).await?;
                            write_half.flush().await?;

                            if let OutgoingMessage::GameOver {..} = resp {
                                break;
                            }
                        }
                        line.clear();
                    }
                    Err(e) => { eprintln!("Pipe read error: {}", e); break; }
                }
            }

            _ = sleep_future => {
                if let Some(s) = session.as_mut() {
                    println!("[SERVER] Timeout detected for player {:?}!", s.turn);
                    let winner = s.turn.opponent();
                    let game_result = if winner == Player::White { (1.0, 0.0) } else { (0.0, 1.0) };
                    let (white_elo_change, black_elo_change) = get_elo_changes_for_result(s, game_result);

                    let response = OutgoingMessage::GameOver {
                        reason: GameOverReason::Timeout,
                        winner: Some(convert_player(winner)),
                        white_elo_change,
                        black_elo_change,
                        fen: get_fen(s),
                        pgn: get_move_history_string(s),
                        move_count: s.move_history.len() as u32,
                    };

                    let json = serde_json::to_string(&response)? + "\n";
                    println!("[SERVER] Sending: {}", json.trim());
                    write_half.write_all(json.as_bytes()).await?;
                    write_half.flush().await?;

                    break;
                }
            }
        }
    }
    Ok(())
}

fn handle_incoming_message(session: &mut Option<GameSession>, line: &str) -> Option<OutgoingMessage> {
    match serde_json::from_str::<IncomingMessage>(line) {
        Ok(msg) => {
            match msg {
                IncomingMessage::StartGame { initial_time_ms, white_elo, black_elo, is_bot_game, .. } => {
                    let mut new_session = GameSession {
                        board: Board::new(),
                        timer: GameTimer::new(Duration::from_millis(initial_time_ms as u64)),
                        turn: Player::White,
                        shakmaty_helper: shakmaty::Chess::default(),
                        last_move_san: None,
                        white_elo,
                        black_elo,
                        is_bot_game,
                        move_history: Vec::new(),
                    };
                    new_session.timer.start_turn();
                    *session = Some(new_session);
                    Some(create_move_result(session.as_ref().unwrap(), true, Some("Game started successfully.".to_string()), None))
                }
                IncomingMessage::MakeMove { san_move } => {
                    if let Some(s) = session.as_mut() {
                        s.timer.stop_turn_timing(s.turn);
                        match San::from_str(&san_move) {
                            Ok(san) => match san.to_move(&s.shakmaty_helper) {
                                Ok(m) => {
                                    s.shakmaty_helper.play_unchecked(&m);
                                    s.turn = s.turn.opponent();
                                    s.timer.start_turn();
                                    s.last_move_san = Some(san_move.clone());
                                    s.move_history.push(san_move.clone());

                                    if s.shakmaty_helper.is_checkmate() {
                                        let winner = s.turn.opponent();
                                        let game_result = if winner == Player::White { (1.0, 0.0) } else { (0.0, 1.0) };
                                        let (white_elo_change, black_elo_change) = get_elo_changes_for_result(s, game_result);
                                        Some(OutgoingMessage::GameOver {
                                            reason: GameOverReason::Checkmate,
                                            winner: Some(convert_player(winner)),
                                            white_elo_change,
                                            black_elo_change,
                                            fen: get_fen(s),
                                            pgn: get_move_history_string(s),
                                            move_count: s.move_history.len() as u32,
                                        })
                                    } else if s.shakmaty_helper.is_stalemate() {
                                        let (white_elo_change, black_elo_change) = get_elo_changes_for_result(s, (0.5, 0.5));
                                        Some(OutgoingMessage::GameOver {
                                            reason: GameOverReason::Stalemate,
                                            winner: None,
                                            white_elo_change,
                                            black_elo_change,
                                            fen: get_fen(s),
                                            pgn: get_move_history_string(s),
                                            move_count: s.move_history.len() as u32,
                                        })
                                    } else {
                                        Some(create_move_result(s, true, Some("Move successful.".to_string()), None))
                                    }
                                }
                                Err(e) => Some(create_move_result(s, false, Some(format!("Move '{}' is ambiguous or illegal: {:?}", san_move, e)), None)),
                            }
                            Err(e) => Some(create_move_result(s, false, Some(format!("Move '{}' is not valid SAN: {:?}", san_move, e)), None)),
                        }
                    } else { Some(OutgoingMessage::Error { message: "Game not started.".into() }) }
                }
                IncomingMessage::Resign { player_color } => {
                    if let Some(s) = session.as_ref() {
                        let winner = match player_color { PlayerColor::White => Some(PlayerColor::Black), PlayerColor::Black => Some(PlayerColor::White) };
                        let game_result = if player_color == PlayerColor::White { (0.0, 1.0) } else { (1.0, 0.0) };
                        let (white_elo_change, black_elo_change) = get_elo_changes_for_result(s, game_result);
                        Some(OutgoingMessage::GameOver {
                            reason: GameOverReason::Resignation,
                            winner,
                            white_elo_change,
                            black_elo_change,
                            fen: get_fen(s),
                            pgn: get_move_history_string(s),
                            move_count: s.move_history.len() as u32,
                        })
                    } else { Some(OutgoingMessage::Error { message: "Cannot resign, game not started.".into() }) }
                }
                IncomingMessage::ClaimDraw => {
                    if let Some(s) = session.as_ref() {
                        let (white_elo_change, black_elo_change) = get_elo_changes_for_result(s, (0.5, 0.5));
                        Some(OutgoingMessage::GameOver {
                            reason: GameOverReason::AgreedDraw,
                            winner: None,
                            white_elo_change,
                            black_elo_change,
                            fen: get_fen(s),
                            pgn: get_move_history_string(s),
                            move_count: s.move_history.len() as u32,
                        })
                    } else { Some(OutgoingMessage::Error { message: "Cannot claim draw, game not started.".into() }) }
                }
                IncomingMessage::RequestBoard => {
                    if let Some(s) = session.as_ref() {
                        Some(create_move_result(s, true, Some("Current game state retrieved.".to_string()), Some(get_move_history_string(s))))
                    } else {
                        Some(OutgoingMessage::Error { message: "Cannot request board state, no active game.".into() })
                    }
                }
                IncomingMessage::Abort => {
                    if let Some(s) = session.as_ref() {
                        Some(OutgoingMessage::GameOver {
                            reason: GameOverReason::Aborted, winner: None, white_elo_change: 0, black_elo_change: 0,
                            fen: get_fen(s), pgn: get_move_history_string(s), move_count: s.move_history.len() as u32,
                        })
                    } else {
                        Some(OutgoingMessage::GameOver {
                            reason: GameOverReason::Aborted, winner: None, white_elo_change: 0, black_elo_change: 0,
                            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(), pgn: "".to_string(), move_count: 0,
                        })
                    }
                }
            }
        }
        Err(e) => Some(OutgoingMessage::Error { message: format!("JSON parse error: {}", e) })
    }
}

fn get_fen(session: &GameSession) -> String {
    shakmaty::fen::Fen::from_position(session.shakmaty_helper.clone(), shakmaty::EnPassantMode::Legal).to_string()
}

fn get_move_history_string(session: &GameSession) -> String {
    session.move_history.join(" ")
}

fn get_elo_changes_for_result(session: &GameSession, game_result: (f64, f64)) -> (i32, i32) {
    if session.is_bot_game {
        (0, 0)
    } else {
        calculate_elo_change(session.white_elo, session.black_elo, game_result)
    }
}

fn calculate_elo_change(white_elo: u32, black_elo: u32, game_result: (f64, f64)) -> (i32, i32) {
    const K_FACTOR: f64 = 32.0;
    let (white_actual_score, black_actual_score) = game_result;
    let r_white = white_elo as f64;
    let r_black = black_elo as f64;
    let expected_white = 1.0 / (1.0 + 10.0f64.powf((r_black - r_white) / 400.0));
    let expected_black = 1.0 / (1.0 + 10.0f64.powf((r_white - r_black) / 400.0));
    let white_change = K_FACTOR * (white_actual_score - expected_white);
    let black_change = K_FACTOR * (black_actual_score - expected_black);
    (white_change.round() as i32, black_change.round() as i32)
}

fn create_move_result(session: &GameSession, is_valid: bool, message: Option<String>, move_history: Option<String>) -> OutgoingMessage {
    OutgoingMessage::MoveResult {
        fen: get_fen(session),
        turn: convert_player(session.turn),
        white_ms: session.timer.get_remaining_time(Player::White).as_millis() as u32,
        black_ms: session.timer.get_remaining_time(Player::Black).as_millis() as u32,
        last_move: session.last_move_san.clone(),
        is_valid,
        message,
        move_history,
    }
}

fn convert_player(player: Player) -> PlayerColor {
    match player {
        Player::White => PlayerColor::White,
        Player::Black => PlayerColor::Black,
    }
}

fn convert_shakmaty_piece(piece: shakmaty::Role) -> PieceType {
    match piece {
        shakmaty::Role::Queen => PieceType::Queen,
        shakmaty::Role::Rook => PieceType::Rook,
        shakmaty::Role::Bishop => PieceType::Bishop,
        shakmaty::Role::Knight => PieceType::Knight,
        _ => PieceType::Queen,
    }
}