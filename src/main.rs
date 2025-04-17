mod game;

use game::GridGame;
use ggez::event;
use ggez::GameResult;

fn main() -> GameResult {
    // Create an instance of the game state
    let game = GridGame::new(20, 30.0, 1, 10); // Grid size, cell size, fall speed, spawn rate
    let window_size = game.grid_size as f32 * game.cell_size;

    // Create a game context and event loop
    let cb = ggez::ContextBuilder::new("stackattack_rust", "stepanhampl")
        .window_setup(ggez::conf::WindowSetup::default().title("Stackattack"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(window_size, window_size));

    let (ctx, event_loop) = cb.build()?;

    // Run the main event loop
    event::run(ctx, event_loop, game)
}
