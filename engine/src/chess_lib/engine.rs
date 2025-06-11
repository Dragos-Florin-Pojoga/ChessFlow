use crate::game::*;
use crate::board::*;

use futures::channel::mpsc as futures_mpsc;
use futures::SinkExt;
use futures::StreamExt;


use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use crossbeam::channel::{unbounded, Sender as CrossbeamSender, Receiver as CrossbeamReceiver};

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

use std::cell::OnceCell;
use wasm_bindgen_spawn::ThreadCreator;
use std::sync::OnceLock;
use std::panic::AssertUnwindSafe;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);

        #[cfg(target_arch = "wasm32")]
        web_sys::console::info_1(&format!("{}\n", msg).into());

        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("info: {}", msg);
    }};
}

thread_local! {
    pub static THREAD_CREATOR: OnceCell<Arc<ThreadCreator>> = OnceCell::new();
}
pub fn thread_creator() -> Arc<ThreadCreator> {
    THREAD_CREATOR.with(|cell| Arc::clone(cell.get().unwrap()))
}



// THIS IS THE KEY CHANGE: Use std::sync::OnceLock for thread-safe static initialization.
pub static GLOBAL_RESPONSE_SENDER: OnceLock<futures::channel::mpsc::Sender<String>> = OnceLock::new();


#[macro_export]
macro_rules! send_response {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(tx) = GLOBAL_RESPONSE_SENDER.get() {
                let mut tx_clone = tx.clone();
                let msg_clone = msg.clone();
                tx_clone.try_send(msg_clone.clone());
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        println!("{}", msg);
    }};
}

enum SearchTask {
    Go { game: Game, depth: u8, is_eval: bool },
    Quit,
}

pub struct Engine {
    game: Game,
    stop_signal: Arc<AtomicBool>,
    max_depth: u8,
    max_q_depth: u8,
    search_sender: CrossbeamSender<SearchTask>,
    is_evaluation_mode: bool,
}


impl Engine {
    pub fn new() -> Self {
        let (search_sender, search_receiver) = unbounded::<SearchTask>();
        
        let search_thread_loop = move || {
            log!("Spawned search thread: '{}' started. Waiting for tasks.", thread::current().name().unwrap_or("unnamed"));
            loop {
                match search_receiver.recv() {
                    Ok(SearchTask::Go { mut game, depth, is_eval }) => {
                        if is_eval {
                            let pseudo_legal_moves : &mut Vec<ChessMove> = &mut Vec::with_capacity(200);
                            let legal_moves : &mut Vec<ChessMove> = &mut Vec::with_capacity(100);
                            for i in 1..=depth {
                                let best_move = game.find_best_move(i);
                                let gamestate = game.get_game_state();
                                let eval = game.board.evaluate(0, pseudo_legal_moves, legal_moves, gamestate);
                                if let Some(mv) = best_move {
                                    send_response!("info depth {} score cp {} pv {}", i + game.q_search_max_ply, eval as f32 / 10.0, mv.to_uci());
                                } else {
                                    send_response!("info depth {} score cp {} pv", i + game.q_search_max_ply, eval as f32 / 10.0);
                                }
                            }
                        } else {
                            let best_move = game.find_best_move(depth);
                            if let Some(mv) = best_move {
                                send_response!("bestmove {}", mv.to_uci());
                            } else {
                                send_response!("bestmove (none)");
                            }
                        }
                    },
                    Ok(SearchTask::Quit) => {
                        break; // For wasm32, this WILL panic
                    },
                    Err(_) => {
                        break; // Sender disconnected or other error, exit the thread
                    }
                }
            }
        };

        #[cfg(target_arch = "wasm32")]
        thread_creator().spawn(AssertUnwindSafe(search_thread_loop)).unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        thread::spawn(search_thread_loop);

        let max_depth = 5; // default search depth
        let max_q_depth = 3; // default quiescence search depth
        let stop_signal = Arc::new(AtomicBool::new(false));

        Engine {
            game: Game::new(Board::new_start_pos(), max_depth, max_q_depth, Arc::clone(&stop_signal)),
            stop_signal,
            max_depth,
            max_q_depth,
            search_sender,
            is_evaluation_mode: false,
        }
    }

    pub fn process_command(&mut self, command: UciCommand) {
        match command {
            UciCommand::Uci => {
                send_response!(
                    "id name ChessFlow\n\
                    id author ChessFlow\n\
                    option name max_depth type spin default 7 min 1 max 20\n\
                    option name max_q_depth type spin default 3 min 1 max 10\n\
                    option name is_evaluation_mode type check default false\n\
                    uciok");
            }
            UciCommand::SetOption { name, value } => {
                log!("SetOption: {} = {}", name, value);
                match name.to_ascii_lowercase().as_str() {
                    "max_depth" => {
                        let val = value.parse().unwrap_or(7);
                        if val >= 1 && val <= 20 {
                            self.max_depth = val;
                        }
                    },
                    "max_q_depth" => {
                        let val = value.parse().unwrap_or(3);
                        if val >= 1 && val <= 10 {
                            self.max_q_depth = val;
                        }
                    },
                    "is_evaluation_mode" => {
                        let val = value.parse().unwrap_or(false);
                        self.is_evaluation_mode = val;
                    }
                    _ => {}
                }
                self.game = Game::new(Board::new_start_pos(), self.max_depth, self.max_q_depth, Arc::clone(&self.stop_signal));
            }
            UciCommand::IsReady => {
                send_response!("readyok");
            }
            UciCommand::NewGame => {
                self.game = Game::new(Board::new_start_pos(), self.max_depth, self.max_q_depth, Arc::clone(&self.stop_signal));
            }
            UciCommand::Position { fen, moves } => {
                let board = if let Some(fen) = fen {
                    Board::from_fen(&fen).expect("Invalid FEN")
                } else {
                    Board::new_start_pos()
                };
                self.game = Game::new(board, self.max_depth, self.max_q_depth, Arc::clone(&self.stop_signal));
                for mv_str in moves {
                    if let Ok(mv) = parse_move_string(&mv_str) {
                        self.game.make_move(&mv);
                    }
                }
            }
            UciCommand::Go { depth } => {
                self.stop_signal.store(false, Ordering::Relaxed);
                let game_clone = self.game.clone();
                let depth = depth.unwrap_or(self.max_depth);
                
                self.search_sender.send(SearchTask::Go { 
                    game: game_clone,
                    depth,
                    is_eval: self.is_evaluation_mode,
                }).expect("Failed to send search task");
            }
            UciCommand::Stop => {
                self.stop_signal.store(true, Ordering::Relaxed);
            }
            UciCommand::Quit => {
                self.search_sender.send(SearchTask::Quit).expect("Failed to send quit task");
                std::process::exit(0);
            }
        }
    }
}