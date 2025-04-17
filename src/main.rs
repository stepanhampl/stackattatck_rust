use ggez::event::{self, EventHandler};
use ggez::glam::Vec2;
use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::input::keyboard::{KeyInput, KeyCode};
use ggez::{Context, GameError, GameResult};
use std::time::{Duration, Instant};

struct GridGame {
    grid_size: usize,
    cell_size: f32,
    figure_position: (usize, usize), // Track only the lower box position
    last_update: Instant,
    pending_move: Option<KeyCode>,
}

impl GridGame {
    fn new(grid_size: usize, cell_size: f32) -> Self {
        Self {
            grid_size: grid_size,
            cell_size: cell_size,
            figure_position: (0, grid_size - 2), // upper box - head
            last_update: Instant::now(),
            pending_move: None,
        }
    }

    fn draw_player(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        // Draw lower box
        let (x, y) = self.figure_position;
        let lower_box = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                x as f32 * self.cell_size,
                y as f32 * self.cell_size,
                self.cell_size,
                self.cell_size * 2.0, // draws also the lower box - body
            ),
            Color::BLACK,
        )?;

        canvas.draw(&lower_box, DrawParam::default());

        Ok(())
    }
}

impl EventHandler for GridGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Check if one second has passed
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            // Process pending move
            if let Some(key) = self.pending_move {
                let (mut x, y) = self.figure_position;

                match key {
                    KeyCode::Left => {
                        if x > 0 {
                            x -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if x < self.grid_size - 1 {
                            x += 1;
                        }
                    }
                    _ => {}
                }

                self.figure_position = (x, y);
                self.pending_move = None;
            }

            // Reset the timer
            self.last_update = Instant::now();
        }

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

        // Draw the player using the dedicated method
        self.draw_player(ctx, &mut canvas)?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        key_input: KeyInput,
        _repeat: bool,
    ) -> Result<(), GameError> {
        if let Some(keycode) = key_input.keycode {
            match keycode {
                KeyCode::Left | KeyCode::Right => {
                    // Store the last key press before redraw
                    self.pending_move = Some(keycode);
                }
                _ => {}
            }
        }
        Ok(())
    }
}

fn main() -> GameResult {
    // Create an instance of the game state
    let game = GridGame::new(20, 30.0);
    let window_size = game.grid_size as f32 * game.cell_size;

    // Create a game context and event loop
    let cb = ggez::ContextBuilder::new("stackattack_rust", "stepanhampl")
        .window_setup(ggez::conf::WindowSetup::default().title("Stackattack"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(window_size, window_size));

    let (ctx, event_loop) = cb.build()?;

    // Run the main event loop
    event::run(ctx, event_loop, game)
}
