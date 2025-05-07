use crate::uci_parser::*;

#[cfg(target_arch = "wasm32")]
use web_sys::console;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {{
        let msg = format!($($arg)*);

        #[cfg(target_arch = "wasm32")]
        console::info_1(&format!("{}\n", msg).into());

        #[cfg(not(target_arch = "wasm32"))]
        eprintln!("info: {}", msg);
    }};
}



pub fn execute_command(command: UciCommand) -> String {
    match command {
        UciCommand::Uci => {
            "id name ChessFlow\n\
            id author ChessFlow\n\
            uciok".to_string()
        }
        UciCommand::SetOption { name, value } => {
            log!("Setting option: {} = {}", name, value);
            // WIP/TODO
            "".to_string()
        }
        UciCommand::IsReady => {
            // TODO: Check if is actually ready
            "readyok".to_string()
        }
        UciCommand::NewGame => {
            log!("Starting a new game.");
            // TODO: Reset engine state
            "".to_string()
        }
        UciCommand::Position { fen, moves } => {
            match fen {
                Some(fen) => {
                    log!("Setting position from FEN: {:?}", fen);
                }
                None => {
                    log!("Setting position to startpos.");
                }
            }
            if !moves.is_empty() {
                log!("Applying moves: {:?}", moves);
            }
            // TODO: Update current position here.
            "".to_string()
        }
        UciCommand::Go => {
            log!("Starting search.");
            // TODO: Start search on worker thread
            "".to_string()
        }
        UciCommand::Stop => {
            log!("Stopping search.");
            // TODO: Stop search
            "".to_string()
        }
        UciCommand::Quit => {
            log!("Quitting engine.");
            // TODO: Perform cleanup if needed.
            "".to_string()
        }
    }
}
