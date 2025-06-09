mod chess_lib;
use chess_lib::*;

use wasm_bindgen::prelude::*;
use web_sys::console;
use std::{
    cell::OnceCell, sync::{atomic::AtomicUsize, Arc, Mutex}
};
use js_sys::Function;
use wasm_bindgen_spawn::ThreadCreator;

use std::sync::{atomic::Ordering};
use once_cell::sync::Lazy;

fn main() {}



#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &JsValue);
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_str(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &JsValue);
    #[wasm_bindgen(js_namespace = console, js_name = error)]
    fn error_str(s: &str);
}




thread_local! {
    static THREAD_CREATOR: OnceCell<Arc<ThreadCreator>> = OnceCell::new();
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init().unwrap();

    console::log_1(&"WASM start begin".into());

    let thread_creator = match ThreadCreator::new("/public/pkg/wasm_engine_bg.wasm", "/public/pkg/wasm_engine.js") {
        Ok(v) => v,
        Err(e) => {
            error_str("Failed to create thread creator");
            error(&e);
            return;
        }
    };
    THREAD_CREATOR.with(|cell| {
        let _ = cell.set(Arc::new(thread_creator));
    });

    console::log_1(&"WASM start completed".into());
}

fn thread_creator() -> Arc<ThreadCreator> {
    THREAD_CREATOR.with(|cell| Arc::clone(cell.get().unwrap()))
}


// USE FOR TESTING ONLY
// THIS WILL PANICK
#[wasm_bindgen]
pub fn process_string(line: &str) -> String {
    format!("{:?}", parse_command(line).unwrap())
}

// USE FOR TESTING ONLY
#[wasm_bindgen]
pub fn parse_and_execute_line(line: &str) -> String {
    execute_command(parse_command(line).unwrap())
}


#[wasm_bindgen]
pub fn debug() {
    console::error_1(&"DEBUG".into());
}



static COUNTER: Lazy<Arc<AtomicUsize>> = Lazy::new(|| Arc::new(AtomicUsize::new(0)));


#[wasm_bindgen]
pub fn multithreading_test_stage_one() {
    let tc = thread_creator();
    let mut handles = vec![];

    let counter = COUNTER.clone();
    let handle = tc
        .spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            for i in 1..=10000 {
                std::thread::sleep(std::time::Duration::from_millis(1));
                counter.fetch_add(1, Ordering::Relaxed);
            }
            log_str("counter thread finished");
            0
        })
        .unwrap();
    handles.push(handle);
}

#[wasm_bindgen]
pub fn multithreading_test_stage_two() {
    let tc = thread_creator();
    let mut handles = vec![];

    let counter = COUNTER.clone();
    let handle = tc
        .spawn(move || {
            // Load and print the counter value from within the thread
            let current_value = counter.load(Ordering::Relaxed);
            log_str(&format!("Counter value inside thread: {}", current_value));

            0
        })
        .unwrap();
    handles.push(handle);

    for handle in handles {
        match handle.join() {
            Ok(value) => log_str(&format!("Worker thread returned: {}", value)),
            Err(e) => log_str(&format!("Worker thread failed: {:?}", e)),
        }
    }
}



use wasm_bindgen::JsValue;
use futures::{channel::{mpsc, oneshot}, SinkExt, StreamExt};
use std::panic;
use std::panic::AssertUnwindSafe; 


#[wasm_bindgen]
pub fn trigger_callback_from_rust(msg: &str) {
    let global = js_sys::global()
                    .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
                    .expect("Failed to get DedicatedWorkerGlobalScope");
    let message_obj = js_sys::Object::new();
    js_sys::Reflect::set(&message_obj, &JsValue::from_str("type"), &JsValue::from_str("callback_trigger")).unwrap();
    js_sys::Reflect::set(&message_obj, &JsValue::from_str("message"), &JsValue::from_str(msg)).unwrap();
    global.post_message(&message_obj).expect("Failed to post message from worker");
}

// Global static for the MPSC sender to send messages to the persistent thread.
static mut UCI_COMMAND_SENDER: Option<mpsc::Sender<String>> = None;

// New static to send results *from* the computation thread *to* the main worker thread
// where `post_message` can be safely called.
// We no longer need to store result_tx here, as the engine thread will receive its own clone.
// The main thread will directly use the 'result_rx'.
// static mut RESULT_SENDER: Option<mpsc::Sender<String>> = None;


#[wasm_bindgen]
pub fn start_engine_thread() {
    let (tx, mut rx) = mpsc::channel::<String>(300); // For UCI commands TO the engine thread
    let (result_tx, mut result_rx) = mpsc::channel::<String>(100); // For results FROM the engine thread

    // Store the UCI command sender globally
    unsafe {
        UCI_COMMAND_SENDER = Some(tx);
        // RESULT_SENDER = Some(result_tx); // Removed: result_tx is now handled by cloning
    }

    let tc = thread_creator();

    // Clone result_tx BEFORE moving it into the spawned thread's closure
    let result_tx_for_engine_thread = result_tx.clone(); // <--- CLONE HERE

    // The persistent engine thread
    tc.spawn(AssertUnwindSafe(move || {
        use futures::executor::block_on;

        // Use the cloned sender here
        let mut result_tx_clone = result_tx_for_engine_thread; // No need to clone again, just use the moved one

        block_on(async move {
            let mut game = Game::new(Board::new_start_pos(), 7, 3);
            while let Some(owned_msg) = rx.next().await {
                let mut result_message = String::new();

                if let Result::Ok(command) = parse_command(&owned_msg) {
                    match command {
                        UciCommand::Uci => result_message = "id name ChessFlow\n\
                                                            id author ChessFlow\n\
                                                            uciok".to_string(),
                        UciCommand::IsReady => result_message = "readyok".to_string(),
                        UciCommand::Position { fen, moves } => {
                            if let Some(fen) = fen {
                                game = Game::new(Board::from_fen(&fen).unwrap(), 7, 3);
                                if let Some(cpu) = game.find_best_move(3) {
                                    let pseudo_legal_moves : &mut Vec<ChessMove> = &mut Vec::with_capacity(200);
                                    let legal_moves : &mut Vec<ChessMove> = &mut Vec::with_capacity(100);
                                    let gamestate = game.get_game_state();
                                    let eval = game.board.evaluate(0, pseudo_legal_moves, legal_moves, gamestate);
                                    result_message = format!("info depth {} score cp {} pv {}", game.max_search_depth + game.q_search_max_ply, eval as f32 / 10.0, cpu.to_uci());
                                } else {
                                    result_message = "NO MOVES".to_string();
                                }
                            } else {
                                result_message = "NOT FEN".to_string();
                            }
                        },
                        UciCommand::Stop => {
                            // Stop search and print best
                            continue;
                        }
                        _ => {
                            continue;
                        }
                    }
                } else {
                    result_message = "PARSE FAIL".to_string();
                }

                // Send the result message back to the main worker thread
                if let Err(e) = result_tx_clone.send(result_message).await {
                    console::error_1(&format!("Failed to send result message to main worker: {:?}", e).into());
                }
            }
            console::warn_1(&"UCI worker channel closed, worker terminating.".into());
        });
    })).unwrap();

    // This block runs on the *main Web Worker thread*
    // Listen for results from the engine thread and post them to JS
    wasm_bindgen_futures::spawn_local(async move {
        let global_scope = js_sys::global()
            .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
            .expect("Failed to get DedicatedWorkerGlobalScope for posting messages");

        while let Some(message) = result_rx.next().await { // result_rx is still available here
            console::warn_1(&message.clone().into()); // Still useful for debugging

            let message_obj = js_sys::Object::new();
            js_sys::Reflect::set(&message_obj, &JsValue::from_str("type"), &JsValue::from_str("callback_trigger")).unwrap();
            js_sys::Reflect::set(&message_obj, &JsValue::from_str("message"), &JsValue::from_str(&message)).unwrap();

            // Post the message from the Web Worker's main thread
            global_scope.post_message(&message_obj).expect("Failed to post message from worker");
        }
        console::warn_1(&"Result channel closed, main worker listener terminating.".into());
    });
}


#[wasm_bindgen]
pub fn send_uci_message(msg: &str) {
    let tx_option = unsafe { UCI_COMMAND_SENDER.as_ref() };

    if let Some(tx) = tx_option {
        let owned_msg = msg.to_string();
        let mut tx_clone = tx.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = tx_clone.send(owned_msg).await {
                console::error_1(&format!("Failed to send UCI message to worker: {:?}", e).into());
            }
        });
    } else {
        console::error_1(&"UCI worker not started. Call `start_engine_thread()` first.".into());
    }
}