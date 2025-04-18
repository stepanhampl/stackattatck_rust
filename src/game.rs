use ggez::event::EventHandler;
use ggez::graphics::{self, Color, Text};
use ggez::input::keyboard::{KeyInput, KeyCode};
use ggez::input::mouse::MouseButton;
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
    score: u32, // Add a score counter
    restart_button: graphics::Rect, // Store the restart button area
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
            score: 0, // Initialize score to 0
            restart_button: graphics::Rect::new(0.0, 0.0, 0.0, 0.0), // Will be set in draw
        };
        
        // Spawn the first block
        game.spawn_block();
        
        game
    }

    // Add restart game method to reset game state
    fn restart_game(&mut self) {
        self.player = Player::new(self.grid_size);
        self.blocks.clear();
        self.last_update = Instant::now();
        self.pending_move = None;
        self.block_spawn_counter = 0;
        self.game_over = false;
        self.score = 0;
        
        // Spawn the first block for the new game
        self.spawn_block();
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

    fn check_full_rows(&mut self) {
        // Check each row from the bottom up
        for row in (0..self.grid_size).rev() {
            // Count non-falling blocks in this row
            let blocks_in_row = self.blocks.iter()
                .filter(|block| !block.falling && block.position.1 == row)
                .count();
            
            // If the row is full
            if blocks_in_row == self.grid_size {
                // Remove all blocks in this row
                self.blocks.retain(|block| block.position.1 != row);
                
                // Increment the score
                self.score += 1;
                
                // Check for blocks that are now levitating after removing the row
                self.check_for_levitating_blocks();
                
                // We'll check one row at a time to keep it simple
                // The next full row (if any) will be caught in the next update
                break;
            }
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
        
        // Add this new line: Check if any rows are full
        self.check_full_rows();
    }

    fn update_player(&mut self) {
        // Update jump counter first
        self.player.update_jump();
        
        // Update fall delay counter
        self.player.update_fall_delay();
        
        // Check if player should start falling
        self.player.update_falling_state(&self.blocks, self.grid_size);
        
        // Apply gravity if player is falling
        if self.player.is_falling {
            self.player.apply_gravity();
        }
        
        // Check if player should land, passing blocks for collision detection
        self.player.land(&self.blocks, self.grid_size);
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

        // Draw the score bar
        let score_bar = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                0.0,
                0.0,
                self.grid_size as f32 * self.cell_size,
                self.cell_size,
            ),
            Color::BLUE,
        )?;
        canvas.draw(&score_bar, graphics::DrawParam::default());
        
        // Draw the score text
        let score_text = Text::new(format!("Score: {}", self.score));
        let text_x = 10.0; // Left padding
        let text_y = self.cell_size / 2.0;
        
        canvas.draw(
            &score_text, 
            graphics::DrawParam::default()
                .dest([text_x, text_y])
                .color(Color::WHITE)
                .offset([0.0, 0.5]) // Center vertically
        );
        
        // Create and draw restart button
        let button_width = 80.0;
        let button_height = self.cell_size * 0.8;
        let button_x = self.grid_size as f32 * self.cell_size - button_width - 10.0; // Right aligned with padding
        let button_y = (self.cell_size - button_height) / 2.0; // Centered vertically
        
        // Store the button area for hit detection
        self.restart_button = graphics::Rect::new(button_x, button_y, button_width, button_height);
        
        let restart_button_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            self.restart_button,
            Color::GREEN, // Use a different color to make it stand out
        )?;
        canvas.draw(&restart_button_mesh, graphics::DrawParam::default());
        
        // Draw restart text
        let restart_text = Text::new("Restart");
        let restart_text_x = button_x + button_width / 2.0;
        let restart_text_y = button_y + button_height / 2.0;
        
        canvas.draw(
            &restart_text, 
            graphics::DrawParam::default()
                .dest([restart_text_x, restart_text_y])
                .color(Color::BLACK)
                .offset([0.5, 0.5]) // Center the text
        );

        // Define the offset for all game elements
        let y_offset = self.cell_size;

        // Draw the grid with offset
        draw_grid(ctx, &mut canvas, self.grid_size, self.cell_size, y_offset)?;

        // Draw the player with offset
        let player_pos = self.player.position;
        let player_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(
                player_pos.0 as f32 * self.cell_size,
                player_pos.1 as f32 * self.cell_size,
                self.cell_size,
                self.cell_size * self.player.body_size as f32,
            ),
            Color::RED,
        )?;
        canvas.draw(&player_mesh, graphics::DrawParam::default().dest([0.0, y_offset]));
        
        // Draw the blocks with offset
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
            canvas.draw(&block_mesh, graphics::DrawParam::default().dest([0.0, y_offset]));
        }
        
        // Draw "Game Over" text if the game is over
        if self.game_over {
            let window_width = self.grid_size as f32 * self.cell_size;
            let window_height = window_width + self.cell_size; // Include score bar height
            let game_over_text = Text::new("Game Over");
            
            // Position text in the center of the screen
            let text_x = window_width / 2.0;
            let text_y = window_height / 2.0;
            
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
                    // Store the last key press before redraw
                    self.pending_move = Some(keycode);
                },
                _ => {}
            }
        }
        Ok(())
    }
    
    // Add mouse button event handler
    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
        if button == MouseButton::Left {
            // Check if click was inside the restart button
            if self.restart_button.contains([x, y]) {
                self.restart_game();
            }
        }
        Ok(())
    }
}
