use ggez::event::EventHandler;
use ggez::graphics::{self, Color, Text};
use ggez::input::keyboard::{KeyInput, KeyCode};
use ggez::input::mouse::MouseButton;
use ggez::{Context, GameError, GameResult};
use std::time::{Duration, Instant};
use std::collections::HashSet; // Add this import
use std::collections::VecDeque; // Add this import for tracking key order

use crate::block::{Block, spawn_random_block};
use crate::player::Player;
use crate::rendering::draw_grid;

pub struct GridGame {
    pub grid_size: usize,
    pub cell_size: f32,
    player: Player,
    last_update: Instant,
    pending_move: Option<KeyCode>,
    held_keys: HashSet<KeyCode>, // Track keys that are currently being held down
    keys_pressed_since_update: Vec<KeyCode>, // Track keys pressed since last update
    direction_press_order: VecDeque<KeyCode>, // Track order of direction key presses
    refresh_rate_milliseconds: u64,
    blocks: Vec<Block>,
    block_fall_speed: usize,
    block_spawn_rate: u64,  // How many refresh cycles between new blocks
    block_spawn_counter: u64, // Counter for block spawning
    game_over: bool,
    score: u32, // Add a score counter
    restart_button: graphics::Rect, // Store the restart button area
    last_move_direction: Option<isize>, // Track the last movement direction
}

impl GridGame {
    pub fn new(grid_size: usize, cell_size: f32, refresh_rate_milliseconds: u64, block_fall_speed: usize, block_spawn_rate: u64) -> Self {
        let mut game = Self {
            grid_size,
            cell_size,
            player: Player::new(grid_size),
            last_update: Instant::now(),
            pending_move: None,
            held_keys: HashSet::new(), // Initialize the set of held keys
            keys_pressed_since_update: Vec::new(), // Initialize the keys pressed since update
            direction_press_order: VecDeque::new(), // Initialize order tracking
            refresh_rate_milliseconds,
            blocks: Vec::new(),
            block_fall_speed,
            block_spawn_rate,
            block_spawn_counter: 0,
            game_over: false,
            score: 0, // Initialize score to 0
            restart_button: graphics::Rect::new(0.0, 0.0, 0.0, 0.0), // Will be set in draw
            last_move_direction: None, // Initialize with no direction
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
        self.held_keys.clear(); // Clear held keys on restart
        self.keys_pressed_since_update.clear(); // Clear pressed keys on restart
        self.direction_press_order.clear(); // Clear direction order on restart
        self.block_spawn_counter = 0;
        self.game_over = false;
        self.score = 0;
        self.last_move_direction = None;
        
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
        self.update_falling_blocks();
        self.handle_block_spawning();
        self.check_for_levitating_blocks();
        self.check_full_rows();
    }

    fn update_falling_blocks(&mut self) {
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

    fn check_block_player_collision(&mut self, x: usize, new_y: usize) -> bool {
        let (player_x, player_y) = self.player.position;
        if x == player_x && new_y == player_y {
            self.game_over = true;
            return true;
        }
        false
    }

    fn check_block_bottom_collision(&mut self, block_idx: usize, new_y: usize) -> bool {
        if new_y >= self.grid_size {
            self.blocks[block_idx].position.1 = self.grid_size - 1;
            self.blocks[block_idx].falling = false;
            return true;
        }
        false
    }

    fn check_block_block_collision(&self, block_idx: usize, x: usize, new_y: usize) -> bool {
        for j in 0..self.blocks.len() {
            if block_idx != j && !self.blocks[j].falling && 
               self.blocks[j].position.0 == x && 
               self.blocks[j].position.1 == new_y {
                return true;
            }
        }
        false
    }

    fn handle_block_spawning(&mut self) {
        self.block_spawn_counter += 1;
        if self.block_spawn_counter >= self.block_spawn_rate {
            self.spawn_block();
            self.block_spawn_counter = 0;
        }
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

    // Determine the current movement direction based on held keys
    fn get_current_movement_direction(&self) -> Option<isize> {
        if self.held_keys.contains(&KeyCode::Left) {
            Some(-1)
        } else if self.held_keys.contains(&KeyCode::Right) {
            Some(1)
        } else {
            None
        }
    }
    
    // New method to determine which movement to process based on key press priority
    fn determine_movement(&mut self) -> Option<KeyCode> {
        // If no keys were pressed, return None
        if self.keys_pressed_since_update.is_empty() {
            return None;
        }
        
        // Check if "Up" was pressed, prioritize jump
        if self.keys_pressed_since_update.contains(&KeyCode::Up) {
            return Some(KeyCode::Up);
        }
        
        // If we have direction keys in the order queue, return the last one
        if !self.direction_press_order.is_empty() {
            return Some(self.direction_press_order.back().cloned().unwrap());
        }
        
        None
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
            // Determine the current movement direction from held keys
            let current_direction = self.get_current_movement_direction();
            self.last_move_direction = current_direction;
            
            // Use the current direction for releasing/maintaining carried blocks
            self.player.release_carried_blocks(&mut self.blocks, current_direction);
            
            // Add held direction keys to the keys_pressed_since_update for continuous movement
            // This ensures continuous movement as long as keys are held down
            if self.held_keys.contains(&KeyCode::Left) {
                self.keys_pressed_since_update.push(KeyCode::Left);
                if !self.direction_press_order.contains(&KeyCode::Left) {
                    self.direction_press_order.push_back(KeyCode::Left);
                }
            }
            if self.held_keys.contains(&KeyCode::Right) {
                self.keys_pressed_since_update.push(KeyCode::Right);
                if !self.direction_press_order.contains(&KeyCode::Right) {
                    self.direction_press_order.push_back(KeyCode::Right);
                }
            }
            
            // Process the key presses according to priority rules
            if let Some(key) = self.determine_movement() {
                match key {
                    KeyCode::Left => self.player.move_left(&mut self.blocks),
                    KeyCode::Right => self.player.move_right(self.grid_size, &mut self.blocks),
                    KeyCode::Up => {
                        // Only jump if we haven't already jumped
                        if !self.player.in_air {
                            self.player.jump();
                        }
                    },
                    _ => {}
                }
                
                // Check for levitating blocks after player moves a block
                self.check_for_levitating_blocks();
            }
            
            // Clear keys pressed since update and direction order
            self.keys_pressed_since_update.clear();
            self.direction_press_order.clear();

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
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::WHITE);

        self.draw_score_bar(ctx, &mut canvas)?;
        self.draw_restart_button(ctx, &mut canvas)?;

        // Define the offset for all game elements
        let y_offset = self.cell_size;

        draw_grid(ctx, &mut canvas, self.grid_size, self.cell_size, y_offset)?;
        self.draw_player(ctx, &mut canvas, y_offset)?;
        self.draw_blocks(ctx, &mut canvas, y_offset)?;
        self.draw_game_over(&mut canvas)?;

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
                    // Add to held keys
                    self.held_keys.insert(keycode);
                    
                    // Add to keys pressed since update
                    self.keys_pressed_since_update.push(keycode);
                    
                    // Update direction order for left/right keys
                    if keycode == KeyCode::Left || keycode == KeyCode::Right {
                        // Remove the key if it's already in the queue (to update its position)
                        if let Some(pos) = self.direction_press_order.iter().position(|&k| k == keycode) {
                            self.direction_press_order.remove(pos);
                        }
                        // Add it to the back (most recent)
                        self.direction_press_order.push_back(keycode);
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }
    
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

    // Add key up event handler to clear direction when keys are released
    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        key_input: KeyInput,
    ) -> Result<(), GameError> {
        if let Some(keycode) = key_input.keycode {
            // Remove from held keys when released
            self.held_keys.remove(&keycode);
            
            // If up arrow is released and a direction key is still held,
            // add that direction key to keys_pressed_since_update to continue movement
            if keycode == KeyCode::Up {
                if self.held_keys.contains(&KeyCode::Left) {
                    self.keys_pressed_since_update.push(KeyCode::Left);
                    // Make sure it's also in the direction queue
                    if !self.direction_press_order.contains(&KeyCode::Left) {
                        self.direction_press_order.push_back(KeyCode::Left);
                    }
                } else if self.held_keys.contains(&KeyCode::Right) {
                    self.keys_pressed_since_update.push(KeyCode::Right);
                    // Make sure it's also in the direction queue
                    if !self.direction_press_order.contains(&KeyCode::Right) {
                        self.direction_press_order.push_back(KeyCode::Right);
                    }
                }
            }
            
            // Clear last_move_direction if releasing a movement key
            match keycode {
                KeyCode::Left => {
                    if self.last_move_direction == Some(-1) {
                        self.last_move_direction = None;
                    }
                },
                KeyCode::Right => {
                    if self.last_move_direction == Some(1) {
                        self.last_move_direction = None;
                    }
                },
                _ => {}
            }
        }
        Ok(())
    }
}

// Add new drawing methods to GridGame
impl GridGame {
    fn draw_score_bar(&self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
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
        
        Ok(())
    }

    fn draw_restart_button(&mut self, ctx: &mut Context, canvas: &mut graphics::Canvas) -> GameResult {
        let button_width = 80.0;
        let button_height = self.cell_size * 0.8;
        let button_x = self.grid_size as f32 * self.cell_size - button_width - 10.0;
        let button_y = (self.cell_size - button_height) / 2.0;
        
        self.restart_button = graphics::Rect::new(button_x, button_y, button_width, button_height);
        
        let restart_button_mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            self.restart_button,
            Color::GREEN,
        )?;
        canvas.draw(&restart_button_mesh, graphics::DrawParam::default());
        
        let restart_text = Text::new("Restart");
        let restart_text_x = button_x + button_width / 2.0;
        let restart_text_y = button_y + button_height / 2.0;
        
        canvas.draw(
            &restart_text, 
            graphics::DrawParam::default()
                .dest([restart_text_x, restart_text_y])
                .color(Color::BLACK)
                .offset([0.5, 0.5])
        );
        
        Ok(())
    }

    fn draw_player(&self, ctx: &mut Context, canvas: &mut graphics::Canvas, y_offset: f32) -> GameResult {
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
        
        Ok(())
    }

    fn draw_blocks(&self, ctx: &mut Context, canvas: &mut graphics::Canvas, y_offset: f32) -> GameResult {
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
        
        Ok(())
    }

    fn draw_game_over(&self, canvas: &mut graphics::Canvas) -> GameResult {
        if !self.game_over {
            return Ok(());
        }
        
        let window_width = self.grid_size as f32 * self.cell_size;
        let window_height = window_width + self.cell_size;
        let game_over_text = Text::new("Game Over");
        
        let text_x = window_width / 2.0;
        let text_y = window_height / 2.0;
        
        canvas.draw(
            &game_over_text, 
            graphics::DrawParam::default()
                .dest([text_x, text_y])
                .color(Color::RED)
                .scale([2.0, 2.0])
                .offset([0.5, 0.5])
        );
        
        Ok(())
    }
}
