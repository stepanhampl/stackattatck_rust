use ggez::event::EventHandler;
use ggez::graphics::{self, Color};
use ggez::input::keyboard::{KeyInput, KeyCode};
use ggez::{Context, GameError, GameResult};
use std::time::{Duration, Instant};

use crate::block::{Block, spawn_random_block};
use crate::player::Player;
use crate::rendering::draw_grid;

pub struct GridGame {
    pub grid_size: usize,
    pub cell_size: f32,
    player: Player,
    last_update: Instant,
    pending_move: Option<KeyCode>,
    refresh_rate_milliseconds: u64,
    blocks: Vec<Block>,
    block_fall_speed: usize,
    block_spawn_rate: u64,  // How many refresh cycles between new blocks
    block_spawn_counter: u64, // Counter for block spawning
}

impl GridGame {
    pub fn new(grid_size: usize, cell_size: f32, block_fall_speed: usize, block_spawn_rate: u64) -> Self {
        let mut game = Self {
            grid_size,
            cell_size,
            player: Player::new(grid_size),
            last_update: Instant::now(),
            pending_move: None,
            refresh_rate_milliseconds: 500,
            blocks: Vec::new(),
            block_fall_speed,
            block_spawn_rate,
            block_spawn_counter: 0,
        };
        
        // Spawn the first block
        game.spawn_block();
        
        game
    }

    fn spawn_block(&mut self) {
        self.blocks.push(spawn_random_block(self.grid_size));
    }

    fn update_blocks(&mut self) {
        // Update existing blocks
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
        
        // Check if it's time to spawn a new block
        self.block_spawn_counter += 1;
        if self.block_spawn_counter >= self.block_spawn_rate {
            self.spawn_block();
            self.block_spawn_counter = 0;
        }
    }
}

impl EventHandler for GridGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Check if one second has passed
        if self.last_update.elapsed() >= Duration::from_millis(self.refresh_rate_milliseconds) {
            // Process pending move
            if let Some(key) = self.pending_move {
                match key {
                    KeyCode::Left => self.player.move_left(),
                    KeyCode::Right => self.player.move_right(self.grid_size),
                    _ => {}
                }
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

        // Draw the grid
        draw_grid(ctx, &mut canvas, self.grid_size, self.cell_size)?;

        // Draw the player
        self.player.draw(ctx, &mut canvas, self.cell_size)?;
        
        // Draw the blocks
        for block in &self.blocks {
            block.draw(ctx, &mut canvas, self.cell_size)?;
        }

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
