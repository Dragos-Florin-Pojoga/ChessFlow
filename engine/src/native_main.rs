mod chess_lib;
use chess_lib::*;

use std::io;
use std::io::{BufRead};

#[cfg(feature = "tracy")]
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "tracy")]
#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> = tracy_client::ProfiledAllocator::new(std::alloc::System, 100);

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();
    
    #[cfg(feature = "tracy")]
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default())
    ).expect("setup tracy layer");
    #[cfg(feature = "tracy")]
    tracing::event!(tracing::Level::INFO, "STARTING PROFILING");

    let mut engine = Engine::new();
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or_default();
        if let Ok(command) = parse_command(&line) {
            engine.process_command(command);
        }
    }

    #[cfg(feature = "tracy")]
    tracing::event!(tracing::Level::INFO, "END PROFILING");
}