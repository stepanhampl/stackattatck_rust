mod game;
mod block;
mod player;
mod rendering;

use game::GridGame;
use ggez::event;
use ggez::GameResult;

fn main() -> GameResult {
    // Create an instance of the game state
    let game = GridGame::new(16, 30.0, 200, 1, 10); // Grid size, cell size, fall speed, spawn rate
    let grid_size = game.grid_size as f32 * game.cell_size;
    
    // Add height for score bar
    let window_width = grid_size;
    let window_height = grid_size + game.cell_size; // Grid size plus score bar height

    // Create a game context and event loop
    let cb = ggez::ContextBuilder::new("stackattack_rust", "stepanhampl")
        .window_setup(ggez::conf::WindowSetup::default().title("Stackattack"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(window_width, window_height));

    let (ctx, event_loop) = cb.build()?;

    // Run the main event loop
    event::run(ctx, event_loop, game)
}
