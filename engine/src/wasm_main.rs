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




use wasm_bindgen::prelude::*;
use js_sys::{ Reflect};
use web_sys::window;
use std::cell::RefCell;
use web_sys::WorkerGlobalScope;
use std::rc::Rc;


#[wasm_bindgen]
pub fn trigger_callback_from_rust(msg: &str) {
    // Get the global scope (the WorkerGlobalScope in a Web Worker)
    let global = js_sys::global()
        .dyn_into::<web_sys::DedicatedWorkerGlobalScope>()
        .expect("Failed to get DedicatedWorkerGlobalScope");

    // Create a JavaScript object to send as a message
    let message_obj = js_sys::Object::new();
    js_sys::Reflect::set(&message_obj, &JsValue::from_str("type"), &JsValue::from_str("callback_trigger")).unwrap();
    js_sys::Reflect::set(&message_obj, &JsValue::from_str("message"), &JsValue::from_str(msg)).unwrap();

    // Post the message back to the main thread
    global.post_message(&message_obj).expect("Failed to post message from worker");
}