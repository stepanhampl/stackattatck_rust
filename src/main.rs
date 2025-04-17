use ggez::{Context, GameResult};
use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;

struct GridGame {
    grid_size: usize,
    cell_size: f32,
}

impl GridGame {
    fn new() -> Self {
        Self {
            grid_size: 20,
            cell_size: 30.0,
        }
    }
}

impl EventHandler for GridGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        // Clear screen with white background (for white squares)
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);
        
        // Draw the grid lines
        for i in 0..=self.grid_size {
            let position = i as f32 * self.cell_size;
            
            // Horizontal line
            let h_line = Mesh::new_line(
                ctx,
                &[
                    Vec2::new(0.0, position),
                    Vec2::new(self.cell_size * self.grid_size as f32, position),
                ],
                1.0,
                Color::BLACK,
            )?;
            
            // Vertical line
            let v_line = Mesh::new_line(
                ctx,
                &[
                    Vec2::new(position, 0.0),
                    Vec2::new(position, self.cell_size * self.grid_size as f32),
                ],
                1.0,
                Color::BLACK,
            )?;
            
            canvas.draw(&h_line, DrawParam::default());
            canvas.draw(&v_line, DrawParam::default());
        }
        
        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    // Create an instance of the game state
    let game = GridGame::new();
    let window_size = game.grid_size as f32 * game.cell_size;
    
    // Create a game context and event loop
    let cb = ggez::ContextBuilder::new("stackattack_rust", "stepanhampl")
        .window_setup(ggez::conf::WindowSetup::default().title("Stackattack"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(window_size, window_size));

    let (ctx, event_loop) = cb.build()?;
    
    // Run the main event loop
    event::run(ctx, event_loop, game)
}
