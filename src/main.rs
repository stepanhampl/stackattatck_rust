// Main entry point for the application
use ggez::event;
use ggez::GameResult;

// Import our platform-specific adapter
mod core;
mod platform;

use platform::ggez::GameAdapter;

fn main() -> GameResult {
    // Game configuration
    let grid_size = 16;
    let cell_size = 30.0;
    let refresh_rate = 200;
    let block_fall_speed = 1;
    let block_spawn_rate = 10;
    
    // Create the game adapter with our configuration
    let game = GameAdapter::new(grid_size, cell_size, refresh_rate, block_fall_speed, block_spawn_rate);
    
    // Calculate window dimensions
    let grid_pixel_size = grid_size as f32 * cell_size;
    let window_width = grid_pixel_size;
    let window_height = grid_pixel_size + cell_size; // Grid size plus score bar height

    // Create a game context and event loop
    let cb = ggez::ContextBuilder::new("stackattack_rust", "stepanhampl")
        .window_setup(ggez::conf::WindowSetup::default().title("Stackattack"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(window_width, window_height));

    let (ctx, event_loop) = cb.build()?;

    // Run the main event loop
    event::run(ctx, event_loop, game)
}
