use ggez::event::EventHandler;
use ggez::graphics::{self, Color, Text};
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
    game_over: bool,
}

impl GridGame {
    pub fn new(grid_size: usize, cell_size: f32, refresh_rate_milliseconds: u64, block_fall_speed: usize, block_spawn_rate: u64) -> Self {
        let mut game = Self {
            grid_size,
            cell_size,
            player: Player::new(grid_size),
            last_update: Instant::now(),
            pending_move: None,
            refresh_rate_milliseconds,
            blocks: Vec::new(),
            block_fall_speed,
            block_spawn_rate,
            block_spawn_counter: 0,
            game_over: false,
        };
        
        // Spawn the first block
        game.spawn_block();
        
        game
    }

    fn spawn_block(&mut self) {
        self.blocks.push(spawn_random_block(self.grid_size));
    }

    fn check_for_levitating_blocks(&mut self) {
        let mut blocks_changed = false;
        
        for i in 0..self.blocks.len() {
            // Skip blocks that are already falling
            if self.blocks[i].falling {
                continue;
            }
            
            let (x, y) = self.blocks[i].position;
            
            // Skip blocks on the bottom row
            if y >= self.grid_size - 1 {
                continue;
            }
            
            // Check if there's a block or ground beneath this one
            let has_support = self.blocks.iter().any(|b| 
                !b.falling && 
                b.position.0 == x && 
                b.position.1 == y + 1
            );
            
            // If no support is found, make it start falling
            if !has_support {
                self.blocks[i].falling = true;
                blocks_changed = true;
            }
        }
        
        // If blocks started falling, check again for chain reactions
        if blocks_changed {
            self.check_for_levitating_blocks();
        }
    }

    fn update_blocks(&mut self) {
        for i in 0..self.blocks.len() {
            if !self.blocks[i].falling {
                continue;
            }
            
            let (x, y) = self.blocks[i].position;
            let new_y = y + self.block_fall_speed;
            
            // Check if the block will hit the player's head
            let (player_x, player_y) = self.player.position;
            if x == player_x && new_y == player_y {
                self.game_over = true;
                self.blocks[i].falling = false;
                return;
            }
            
            // Check if the block will hit the bottom of the grid
            if new_y >= self.grid_size {
                self.blocks[i].position.1 = self.grid_size - 1;
                self.blocks[i].falling = false;
                continue;
            }
            
            // Check for collision with other blocks
            let mut will_collide = false;
            for j in 0..self.blocks.len() {
                if i != j && !self.blocks[j].falling && 
                   self.blocks[j].position.0 == x && 
                   self.blocks[j].position.1 == new_y {
                    will_collide = true;
                    break;
                }
            }
            
            if will_collide {
                self.blocks[i].falling = false;
            } else {
                self.blocks[i].position.1 = new_y;
            }
        }
        
        // Check if it's time to spawn a new block
        self.block_spawn_counter += 1;
        if self.block_spawn_counter >= self.block_spawn_rate {
            self.spawn_block();
            self.block_spawn_counter = 0;
        }
        
        // Check for levitating blocks after updating
        self.check_for_levitating_blocks();
    }

    fn update_player(&mut self) {
        // Update jump counter first
        self.player.update_jump();
        
        // Then check if player should land, passing blocks for collision detection
        self.player.land(&self.blocks);
    }
}

impl EventHandler for GridGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Skip updates if the game is over
        if self.game_over {
            return Ok(());
        }

        // Check if one second has passed
        if self.last_update.elapsed() >= Duration::from_millis(self.refresh_rate_milliseconds) {
            // Process pending move
            if let Some(key) = self.pending_move {
                match key {
                    KeyCode::Left => self.player.move_left(&mut self.blocks),
                    KeyCode::Right => self.player.move_right(self.grid_size, &mut self.blocks),
                    KeyCode::Up => {
                        println!("Processing Up key press!");
                        self.player.jump();
                    },
                    _ => {}
                }
                self.pending_move = None;
                
                // Check for levitating blocks after player moves a block
                self.check_for_levitating_blocks();
            }

            // Update player (handles landing after jump)
            self.update_player();
            
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

        // Draw "Game Over" text if the game is over
        if self.game_over {
            let window_size = self.grid_size as f32 * self.cell_size;
            let game_over_text = Text::new("Game Over");
            
            // Position text in the center of the screen
            let text_x = window_size / 2.0;
            let text_y = window_size / 2.0;
            
            canvas.draw(
                &game_over_text, 
                graphics::DrawParam::default()
                    .dest([text_x, text_y])
                    .color(Color::RED)
                    .scale([2.0, 2.0])
                    .offset([0.5, 0.5])  // Center the text at the destination point
            );
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
        // Ignore input if game is over
        if self.game_over {
            return Ok(());
        }

        if let Some(keycode) = key_input.keycode {
            match keycode {
                KeyCode::Left | KeyCode::Right | KeyCode::Up => {
                    println!("Key pressed: {:?}", keycode);
                    // Store the last key press before redraw
                    self.pending_move = Some(keycode);
                },
                _ => {}
            }
        }
        Ok(())
    }
}
