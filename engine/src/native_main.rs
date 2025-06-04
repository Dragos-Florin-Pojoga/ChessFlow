mod chess_lib;
use chess_lib::*;



#[cfg(feature = "tracy")]
use tracing_subscriber::layer::SubscriberExt;

#[cfg(feature = "tracy")]
#[global_allocator]
static GLOBAL: tracy_client::ProfiledAllocator<std::alloc::System> = tracy_client::ProfiledAllocator::new(std::alloc::System, 100);

fn main() {
    #[cfg(feature = "tracy")]
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(tracing_tracy::TracyLayer::default())
    ).expect("setup tracy layer");
    #[cfg(feature = "tracy")]
    tracing::event!(tracing::Level::INFO, "STARTING PROFILING");

    let mut game = Game::new(Board::new_start_pos(), 7);
    let mut depth = 5;

    loop {
        #[cfg(feature = "tracy")]
        tracy_client::frame_mark();

        if let Some(cpu) = game.find_best_move(depth) {
            game.make_move(&cpu);
        } else {
            break;
        }
        if game.get_game_state() != GameState::Ongoing {
            break;
        }

        game.print();
    }

    game.print_end();

    #[cfg(feature = "tracy")]
    tracing::event!(tracing::Level::INFO, "END PROFILING");
}