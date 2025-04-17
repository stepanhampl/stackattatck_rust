use ggez::event::EventHandler;
use ggez::glam::Vec2;
use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::input::keyboard::{KeyInput, KeyCode};
use ggez::{Context, GameError, GameResult};
use std::time::{Duration, Instant};
use rand::Rng;

struct Block {
    position: (usize, usize),
    falling: bool,
}

pub struct GridGame {
    pub grid_size: usize,
    pub cell_size: f32,
    figure_position: (usize, usize), // Track only the lower box position
    last_update: Instant,
    pending_move: Option<KeyCode>,
    refresh_rate_milliseconds: u64,
    blocks: Vec<Block>,
    block_fall_speed: usize,
}

impl GridGame {
    pub fn new(grid_size: usize, cell_size: f32, block_fall_speed: usize) -> Self {
        let mut game = Self {
            grid_size,
            cell_size,
            figure_position: (0, grid_size - 2),
            last_update: Instant::now(),
            pending_move: None,
            refresh_rate_milliseconds: 500,
            blocks: Vec::new(),
            block_fall_speed,
        };
        
        // Spawn the first block
        game.spawn_block();
        
        game
    }

    fn spawn_block(&mut self) {
        let mut rng = rand::rng();
        let x = rng.random_range(0..self.grid_size);
        
        self.blocks.push(Block {
            position: (x, 0),
            falling: true,
        });
    }

    fn update_blocks(&mut self) {
        for block in &mut self.blocks {
            if block.falling {
                let (x, y) = block.position;
                let new_y = y + self.block_fall_speed;
                
                if new_y < self.grid_size {
                    block.position = (x, new_y);
                } else {
                    block.position = (x, self.grid_size - 1);
                    block.falling = false;
                }
            }
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

    fn draw_blocks(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        for block in &self.blocks {
            let (x, y) = block.position;
            let block_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    x as f32 * self.cell_size,
                    y as f32 * self.cell_size,
                    self.cell_size,
                    self.cell_size,
                ),
                Color::BLACK,
            )?;
            canvas.draw(&block_mesh, DrawParam::default());
        }
        Ok(())
    }
}

impl EventHandler for GridGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Check if one second has passed
        if self.last_update.elapsed() >= Duration::from_millis(self.refresh_rate_milliseconds) {
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

            // Update falling blocks
            self.update_blocks();

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
        
        // Draw the blocks
        self.draw_blocks(ctx, &mut canvas)?;

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
