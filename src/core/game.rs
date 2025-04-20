// Core game implementation - platform-independent
use std::time::{Duration, Instant};

use crate::core::block::{Block, spawn_random_block};
use crate::core::player::Player;
use crate::core::types::{InputAction, Direction, GameConfig, GameUpdateResult};

pub struct GameState {
    pub grid_size: usize,
    pub cell_size: f32,
    pub player: Player,
    pub last_update: Instant,
    pub refresh_rate_milliseconds: u64,
    pub blocks: Vec<Block>,
    pub block_fall_speed: usize,
    pub block_spawn_rate: u64,
    pub block_spawn_counter: u64,
    pub game_over: bool,
    pub score: u32,
    pub last_move_direction: Option<Direction>,
    last_move_time: Instant,
}

impl GameState {
    pub fn new(config: GameConfig) -> Self {
        let mut game = Self {
            grid_size: config.grid_size,
            cell_size: config.cell_size,
            player: Player::new(config.grid_size),
            last_update: Instant::now(),
            refresh_rate_milliseconds: config.refresh_rate_milliseconds,
            blocks: Vec::new(),
            block_fall_speed: config.block_fall_speed,
            block_spawn_rate: config.block_spawn_rate,
            block_spawn_counter: 0,
            game_over: false,
            score: 0,
            last_move_direction: None,
            last_move_time: Instant::now(),
        };
        
        // Spawn the first block
        game.spawn_block();
        
        game
    }

    // Reset game state
    pub fn restart(&mut self) {
        self.player = Player::new(self.grid_size);
        self.blocks.clear();
        self.last_update = Instant::now();
        self.block_spawn_counter = 0;
        self.game_over = false;
        self.score = 0;
        self.last_move_direction = None;
        self.last_move_time = Instant::now();
        
        // Spawn the first block for the new game
        self.spawn_block();
    }

    pub fn spawn_block(&mut self) {
        self.blocks.push(spawn_random_block(self.grid_size));
    }

    pub fn check_for_levitating_blocks(&mut self) {
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

    pub fn check_full_rows(&mut self) {
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

    pub fn update_blocks(&mut self) {
        self.update_falling_blocks();
        self.handle_block_spawning();
        self.check_for_levitating_blocks();
        self.check_full_rows();
    }

    pub fn update_falling_blocks(&mut self) {
        for i in 0..self.blocks.len() {
            // Skip blocks that are currently being carried
            if self.blocks[i].carried {
                continue;
            }
            
            if !self.blocks[i].falling {
                continue;
            }
            
            let (x, y) = self.blocks[i].position;
            let new_y = y + self.block_fall_speed;
            
            if self.check_block_player_collision(x, new_y) {
                return; // Game over detected, exit early
            }
            
            if self.check_block_bottom_collision(i, new_y) {
                continue;
            }
            
            if self.check_block_block_collision(i, x, new_y) {
                self.blocks[i].falling = false;
            } else {
                self.blocks[i].position.1 = new_y;
            }
        }
    }

    pub fn check_block_player_collision(&mut self, x: usize, new_y: usize) -> bool {
        let (player_x, player_y) = self.player.position;
        if x == player_x && new_y == player_y {
            self.game_over = true;
            return true;
        }
        false
    }

    pub fn check_block_bottom_collision(&mut self, block_idx: usize, new_y: usize) -> bool {
        if new_y >= self.grid_size {
            self.blocks[block_idx].position.1 = self.grid_size - 1;
            self.blocks[block_idx].falling = false;
            return true;
        }
        false
    }

    pub fn check_block_block_collision(&self, block_idx: usize, x: usize, new_y: usize) -> bool {
        for j in 0..self.blocks.len() {
            if block_idx != j && !self.blocks[j].falling && 
               self.blocks[j].position.0 == x && 
               self.blocks[j].position.1 == new_y {
                return true;
            }
        }
        false
    }

    pub fn handle_block_spawning(&mut self) {
        self.block_spawn_counter += 1;
        if self.block_spawn_counter >= self.block_spawn_rate {
            self.spawn_block();
            self.block_spawn_counter = 0;
        }
    }

    pub fn update_player(&mut self) {
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

    // Process an input action and update the game state
    pub fn process_input(&mut self, action: InputAction) -> GameUpdateResult {
        // Early exit if game is over
        if self.game_over {
            if action == InputAction::Restart {
                self.restart();
                return GameUpdateResult::Restart;
            }
            return GameUpdateResult::GameOver;
        }

        // Process player movement
        match action {
            InputAction::Left => {
                if self.last_move_time.elapsed() >= Duration::from_millis(self.refresh_rate_milliseconds) {
                    self.last_move_direction = Some(-1);
                    self.player.move_left(&mut self.blocks);
                    self.last_move_time = Instant::now();
                }
            },
            InputAction::Right => {
                if self.last_move_time.elapsed() >= Duration::from_millis(self.refresh_rate_milliseconds) {
                    self.last_move_direction = Some(1);
                    self.player.move_right(&mut self.blocks);
                    self.last_move_time = Instant::now();
                }
            },
            InputAction::Up => {
                self.player.jump();
            },
            InputAction::Restart => {
                self.restart();
                return GameUpdateResult::Restart;
            },
            InputAction::None => {
                // No directional input, release carried blocks
                self.player.release_carried_blocks(&mut self.blocks, None);
                self.last_move_direction = None;
            },
        }

        // Release blocks if direction changed
        self.player.release_carried_blocks(&mut self.blocks, self.last_move_direction);
        
        // Check for levitating blocks that might have been moved
        self.check_for_levitating_blocks();

        GameUpdateResult::Continue
    }

    // Update game state with time progression
    pub fn update(&mut self) -> GameUpdateResult {
        // Skip updates if the game is over
        if self.game_over {
            return GameUpdateResult::GameOver;
        }

        // Check if it's time to update based on refresh rate
        if self.last_update.elapsed() >= Duration::from_millis(self.refresh_rate_milliseconds) {
            // Update player
            self.update_player();
            
            // Update falling blocks
            self.update_blocks();

            // Reset the timer
            self.last_update = Instant::now();
        }

        if self.game_over {
            GameUpdateResult::GameOver
        } else {
            GameUpdateResult::Continue
        }
    }
}
